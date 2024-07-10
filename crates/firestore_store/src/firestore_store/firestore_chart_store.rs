use std::{future::Future, pin::Pin, str::FromStr};

use crate::{
    converter, path,
    schema::{
        self, ChartDocumentData, ChartEventDataDocumentData, EventDocumentData,
        EventStreamDocumentData, UpdaterMetadataDocumentData,
        UpdaterMetadataProcessedEventDocumentData,
    },
};
use firestore_client::{FieldPath, Filter, FirestoreClient, Precondition, Transaction};
use write_model::{
    aggregate::Chart,
    event::ChartEvent,
    value_object::{ChartId, EventId, EventStreamId, Version},
};

pub struct FirestoreChartStore(FirestoreClient);

impl FirestoreChartStore {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self(FirestoreClient::new().await?))
    }

    async fn reader_get_impl(
        &self,
        id: ChartId,
    ) -> Result<
        Option<query_use_case::port::ChartQueryData>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        self.0
            .get_document::<ChartDocumentData>(&path::chart_document(id))
            .await?
            .map(converter::query_data_from_document)
            .transpose()
    }

    async fn reader_list_impl(
        &self,
    ) -> Result<Vec<query_use_case::port::ChartQueryData>, Box<dyn std::error::Error + Send + Sync>>
    {
        let documents = self
            .0
            .list_all_documents::<ChartDocumentData>(&path::chart_collection())
            .await?;
        let documents = documents
            .into_iter()
            .map(converter::query_data_from_document)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(documents)
    }

    async fn repository_find_impl(
        &self,
        id: ChartId,
    ) -> Result<Option<Chart>, Box<dyn std::error::Error + Send + Sync>> {
        let event_stream = self
            .0
            .get_document::<EventStreamDocumentData>(&path::event_stream_document(&id.to_string()))
            .await?;
        if event_stream.is_none() {
            return Ok(None);
        }
        let collection_path = path::event_collection();
        let mut start_after = None;
        let mut all_documents = vec![];
        loop {
            let documents = self
                .0
                .run_collection_query::<EventDocumentData>(
                    &collection_path,
                    Some(Filter::and([FieldPath::raw("stream_id")
                        .equal(firestore_client::to_value(&id.to_string())?)?])),
                    Some([FieldPath::raw("version").ascending()]),
                    start_after.clone(),
                    Some(100),
                )
                .await?;
            let is_end = documents.is_empty() || documents.len() < 100;
            start_after = Some([firestore_client::to_value(
                &documents
                    .last()
                    .expect("documents to have at least one element")
                    .fields
                    .version,
            )?]);
            all_documents.extend(documents);
            if is_end {
                break;
            }
        }
        let events = all_documents
            .into_iter()
            .map(converter::chart_event_from_document)
            .collect::<Result<Vec<ChartEvent>, Box<dyn std::error::Error + Send + Sync>>>()?;
        Ok(Some(Chart::from_events(&events)?))
    }

    async fn repository_store_impl(
        &self,
        current: Option<Version>,
        events: &[ChartEvent],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if events.is_empty() {
            return Ok(());
        }

        let events = events.to_vec();
        let result = self
            .run_transaction(move |transaction| {
                Box::pin(async move {
                    Self::repository_store_impl_transaction(transaction, current, events).await
                })
            })
            .await;

        // To simplify the structure, update the query data at this timing (not supported for failure).
        self.repository_store_impl_update_query_data().await?;

        result
    }

    async fn repository_store_impl_transaction(
        transaction: &mut Transaction,
        current: Option<Version>,
        events: Vec<ChartEvent>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event_stream_id = EventStreamId::from_str(events[0].stream_id.to_string().as_str())?;
        let last_event_version = events
            .last()
            .expect("events to have at least one element")
            .version;
        let last_event_at = events
            .last()
            .expect("events to have at least one element")
            .at;
        match current {
            None => {
                // create event_stream
                transaction.create(
                    &path::event_stream_document(event_stream_id.to_string().as_str()),
                    &EventStreamDocumentData {
                        id: event_stream_id.to_string(),
                        last_event_at: last_event_at.to_string(),
                        version: i64::from(last_event_version),
                    },
                )?;
            }
            Some(current) => {
                // get event_stream with lock
                let event_stream = transaction
                    .get::<EventStreamDocumentData>(&path::event_stream_document(
                        event_stream_id.to_string().as_str(),
                    ))
                    .await?
                    .ok_or("event stream not found")?;

                // check version
                if event_stream.fields.version != i64::from(current) {
                    return Err("version mismatch".into());
                }

                // update event_stream
                transaction.update(
                    &path::event_stream_document(event_stream_id.to_string().as_str()),
                    &EventStreamDocumentData {
                        last_event_at: last_event_at.to_string(),
                        version: i64::from(last_event_version),
                        ..event_stream.fields
                    },
                )?;
            }
        }
        // create events
        for event in events {
            transaction.create(
                &path::event_document(event.id),
                &converter::event_document_data_from_event(&event),
            )?;
        }
        Ok(())
    }

    async fn repository_store_impl_update_query_data(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let updater_metadata_document_path = path::query_updater_document();
        let last_processed_event_at = self
            .0
            .get_document::<UpdaterMetadataDocumentData>(&updater_metadata_document_path)
            .await?
            .map(|document| document.fields.last_processed_event_at)
            .unwrap_or_else(|| "1970-01-01T00:00:00.000Z".to_owned());
        let events = self
            .0
            .run_collection_query::<EventDocumentData>(
                &path::event_collection(),
                Some(Filter::and([FieldPath::raw("at").greater_than_or_equal(
                    // FIXME: last_processed_event_at - 10s
                    firestore_client::to_value(&last_processed_event_at.to_string())?,
                )?])),
                Some([FieldPath::raw("at").ascending()]),
                None::<Vec<_>>,
                None,
            )
            .await?;
        let mut filtered_events = vec![];
        for event in events {
            let document_path = updater_metadata_document_path
                .collection("processed_events")?
                .doc(event.name.document_id().as_ref())?;
            let processed_event = self
                .0
                .get_document::<UpdaterMetadataProcessedEventDocumentData>(&document_path)
                .await?;
            if processed_event.is_none() {
                filtered_events.push(event);
            }
        }
        for event in filtered_events {
            match self
                .run_transaction(move |transaction| {
                    Box::pin(async move {
                        let updater_metadata_document_path = path::query_updater_document();
                        // lock updater_metadata_document
                        let updater_metadata_document = transaction
                            .get::<UpdaterMetadataDocumentData>(&updater_metadata_document_path)
                            .await?;
                        transaction.create(
                            &path::query_updater_processed_event_document(EventId::from_str(
                                &event.fields.id,
                            )?),
                            &UpdaterMetadataProcessedEventDocumentData {},
                        )?;

                        let chart_id = ChartId::from_str(&event.fields.stream_id)?;
                        let chart_document_path = path::chart_document(chart_id);
                        match serde_json::from_str::<ChartEventDataDocumentData>(
                            &event.fields.data,
                        )? {
                            ChartEventDataDocumentData::Created(schema::Created { title }) => {
                                transaction.create(
                                    &chart_document_path,
                                    &ChartDocumentData {
                                        created_at: event.fields.at.clone(),
                                        title,
                                    },
                                )?;
                            }
                            ChartEventDataDocumentData::Deleted(schema::Deleted {}) => {
                                transaction.delete(&chart_document_path)?
                            }
                            ChartEventDataDocumentData::Updated(schema::Updated { title }) => {
                                let document = transaction
                                    .get::<ChartDocumentData>(&chart_document_path)
                                    .await?
                                    .ok_or("not found")?;
                                transaction.update(
                                    &chart_document_path,
                                    &ChartDocumentData {
                                        created_at: document.fields.created_at,
                                        title,
                                    },
                                )?
                            }
                        }

                        match updater_metadata_document {
                            None => {
                                transaction.create(
                                    &updater_metadata_document_path,
                                    &UpdaterMetadataDocumentData {
                                        last_processed_event_at: event.fields.at.clone(),
                                    },
                                )?;
                            }
                            Some(updater_metadata_document) => {
                                transaction.update_with_precondition(
                                    &updater_metadata_document_path,
                                    &UpdaterMetadataDocumentData {
                                        last_processed_event_at: event.fields.at.clone(),
                                    },
                                    Precondition::UpdateTime(updater_metadata_document.update_time),
                                )?;
                            }
                        }
                        Ok(())
                    })
                })
                .await
            {
                Err(_) => {
                    // ignore error
                    // stop event processing
                    break;
                }
                Ok(_) => {
                    // next event
                }
            }
        }

        Ok(())
    }

    async fn run_transaction<F>(
        &self,
        callback: F,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce(
            &mut Transaction,
        ) -> Pin<
            Box<
                dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
                    + Send
                    + '_,
            >,
        >,
    {
        let mut transaction = self.0.begin_transaction().await?;
        let result = match callback(&mut transaction).await {
            Ok(()) => transaction.commit().await.map_err(Into::into),
            Err(e) => Err(e),
        };
        match result {
            Ok(()) => Ok(()),
            Err(e) => {
                // ignore rollback error
                let _ = transaction.rollback().await;
                Err(e)
            }
        }
    }
}

#[async_trait::async_trait]
impl query_use_case::port::ChartReader for FirestoreChartStore {
    async fn get(
        &self,
        id: ChartId,
    ) -> Result<
        Option<query_use_case::port::ChartQueryData>,
        query_use_case::port::chart_reader::Error,
    > {
        self.reader_get_impl(id)
            .await
            .map_err(query_use_case::port::chart_reader::Error::from)
    }

    async fn list(
        &self,
    ) -> Result<Vec<query_use_case::port::ChartQueryData>, query_use_case::port::chart_reader::Error>
    {
        self.reader_list_impl()
            .await
            .map_err(query_use_case::port::chart_reader::Error::from)
    }
}

#[async_trait::async_trait]
impl command_use_case::port::ChartRepository for FirestoreChartStore {
    async fn find(
        &self,
        id: ChartId,
    ) -> Result<Option<Chart>, command_use_case::port::chart_repository::Error> {
        self.repository_find_impl(id)
            .await
            .map_err(command_use_case::port::chart_repository::Error::from)
    }

    async fn store(
        &self,
        current: Option<Version>,
        events: &[ChartEvent],
    ) -> Result<(), command_use_case::port::chart_repository::Error> {
        self.repository_store_impl(current, events)
            .await
            .map_err(command_use_case::port::chart_repository::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use command_use_case::port::ChartRepository as _;

    use super::*;

    #[ignore = "requires Firestore"]
    #[tokio::test]
    async fn test() -> anyhow::Result<()> {
        let store = FirestoreChartStore::new()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        let (chart, events) = Chart::create("title1".to_owned())?;
        assert_eq!(store.find(chart.id()).await?, None);
        store.store(None, &events).await?;
        assert_eq!(store.find(chart.id()).await?, None);
        Ok(())
    }
}
