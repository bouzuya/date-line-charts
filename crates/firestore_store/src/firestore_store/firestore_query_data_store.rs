use std::{future::Future, pin::Pin, str::FromStr};

use crate::{
    converter, path,
    schema::{
        self, ChartDocumentData, ChartEventDataDocumentData, EventDocumentData,
        UpdaterMetadataDocumentData, UpdaterMetadataProcessedEventDocumentData,
    },
};
use firestore_client::{FieldPath, Filter, FirestoreClient, Precondition, Transaction};
use write_model::value_object::{ChartId, EventId};

pub(crate) struct FirestoreQueryDataStore {
    client: FirestoreClient,
}

impl FirestoreQueryDataStore {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            client: FirestoreClient::new().await?,
        })
    }

    pub(crate) async fn get_chart(
        &self,
        id: ChartId,
    ) -> Result<
        Option<query_use_case::port::ChartQueryData>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        self.client
            .get_document::<ChartDocumentData>(&path::chart_document(id))
            .await?
            .map(converter::query_data_from_document)
            .transpose()
    }

    pub(crate) async fn list_charts(
        &self,
    ) -> Result<Vec<query_use_case::port::ChartQueryData>, Box<dyn std::error::Error + Send + Sync>>
    {
        let documents = self
            .client
            .list_all_documents::<ChartDocumentData>(&path::chart_collection())
            .await?;
        let documents = documents
            .into_iter()
            .map(converter::query_data_from_document)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(documents)
    }

    pub(crate) async fn update(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let updater_metadata_document_path = path::query_updater_document();
        let last_processed_event_at = self
            .client
            .get_document::<UpdaterMetadataDocumentData>(&updater_metadata_document_path)
            .await?
            .map(|document| document.fields.last_processed_event_at)
            .unwrap_or_else(|| "1970-01-01T00:00:00.000Z".to_owned());
        let events = self
            .client
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
                .client
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
                        match event.fields.data {
                            schema::EventDataDocumentData::Chart(event_data) => match event_data {
                                ChartEventDataDocumentData::Created(
                                    schema::chart_event_data_document_data::Created { title },
                                ) => {
                                    transaction.create(
                                        &chart_document_path,
                                        &ChartDocumentData {
                                            created_at: event.fields.at.clone(),
                                            title,
                                        },
                                    )?;
                                }
                                ChartEventDataDocumentData::Deleted(
                                    schema::chart_event_data_document_data::Deleted {},
                                ) => transaction.delete(&chart_document_path)?,
                                ChartEventDataDocumentData::Updated(
                                    schema::chart_event_data_document_data::Updated { title },
                                ) => {
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
                            },
                            schema::EventDataDocumentData::DataPoint(_) => unreachable!(),
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
        let mut transaction = self.client.begin_transaction().await?;
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
