use std::{future::Future, pin::Pin, str::FromStr};

use converter::document_data_from_chart_event_data;
use firestore_client::{
    DocumentPath, FieldPath, Filter, FirestoreClient, Precondition, Transaction,
};
use schema::{
    ChartDocumentData, ChartEventDataDocumentData, EventDocumentData, EventStreamDocumentData,
};
use write_model::{
    aggregate::{chart::Event, Chart},
    value_object::{ChartId, EventStreamId, Version},
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
                .run_collection_query::<EventDocumentData<ChartEventDataDocumentData>>(
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
            .collect::<Result<Vec<Event>, Box<dyn std::error::Error + Send + Sync>>>()?;
        Ok(Some(Chart::from_events(&events)?))
    }

    async fn repository_store_impl(
        &self,
        current: Option<Version>,
        events: &[Event],
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
        let updater_metadata_document_path = DocumentPath::from_str("query/updater")?;
        #[derive(serde::Deserialize, serde::Serialize)]
        struct UpdaterMetadataDocumentData {
            last_processed_event_at: String,
        }
        #[derive(serde::Deserialize, serde::Serialize)]
        struct UpdaterMetadataProcessedEventDocumentData {}
        let last_processed_event_at = self
            .0
            .get_document::<UpdaterMetadataDocumentData>(&updater_metadata_document_path)
            .await?
            .map(|document| document.fields.last_processed_event_at)
            .unwrap_or_else(|| "1970-01-01T00:00:00.000Z".to_owned());
        let events = self
            .0
            .run_collection_query::<EventDocumentData<ChartEventDataDocumentData>>(
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
                        let updater_metadata_document_path =
                            DocumentPath::from_str("query/updater")?;
                        // lock updater_metadata_document
                        let updater_metadata_document = transaction
                            .get::<UpdaterMetadataDocumentData>(&updater_metadata_document_path)
                            .await?;
                        transaction.create(
                            &updater_metadata_document_path
                                .collection("processed_events")?
                                .doc(event.name.document_id().as_ref())?,
                            &UpdaterMetadataProcessedEventDocumentData {},
                        )?;

                        let chart_id = ChartId::from_str(&event.fields.stream_id)?;
                        let chart_document_path = path::chart_document(chart_id);
                        match event.fields.data {
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

        result
    }

    async fn repository_store_impl_transaction(
        transaction: &mut Transaction,
        current: Option<Version>,
        events: Vec<Event>,
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
                &EventDocumentData {
                    at: event.at.to_string(),
                    data: document_data_from_chart_event_data(&event.data),
                    id: event.id.to_string(),
                    stream_id: event.stream_id.to_string(),
                    version: i64::from(event.version),
                },
            )?;
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
        events: &[Event],
    ) -> Result<(), command_use_case::port::chart_repository::Error> {
        self.repository_store_impl(current, events)
            .await
            .map_err(command_use_case::port::chart_repository::Error::from)
    }
}

mod converter {
    use std::str::FromStr as _;

    use firestore_client::Document;
    use write_model::{
        aggregate::chart::{
            event::{Created, Deleted, Updated},
            Event, EventData,
        },
        value_object::{ChartId, DateTime},
    };

    use super::schema::{ChartDocumentData, ChartEventDataDocumentData, EventDocumentData};

    pub(crate) fn query_data_from_document(
        document: Document<ChartDocumentData>,
    ) -> Result<query_use_case::port::ChartQueryData, Box<dyn std::error::Error + Send + Sync>>
    {
        Ok(query_use_case::port::ChartQueryData {
            created_at: DateTime::from_str(&document.fields.created_at)?,
            id: ChartId::from_str(document.name.document_id().as_ref())?,
            title: document.fields.title,
        })
    }

    pub(crate) fn chart_event_from_document(
        document: Document<EventDocumentData<ChartEventDataDocumentData>>,
    ) -> Result<Event, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Event {
            at: DateTime::from_str(&document.fields.at)?,
            data: match document.fields.data {
                ChartEventDataDocumentData::Created(data) => {
                    EventData::Created(Created { title: data.title })
                }
                ChartEventDataDocumentData::Deleted(_) => EventData::Deleted(Deleted {}),
                ChartEventDataDocumentData::Updated(data) => {
                    EventData::Updated(Updated { title: data.title })
                }
            },
            id: write_model::value_object::EventId::from_str(document.name.document_id().as_ref())?,
            stream_id: write_model::value_object::ChartId::from_str(&document.fields.stream_id)?,
            version: write_model::value_object::Version::try_from(document.fields.version)?,
        })
    }

    pub(crate) fn document_data_from_chart_event_data(
        event_data: &write_model::aggregate::chart::EventData,
    ) -> ChartEventDataDocumentData {
        match event_data {
            write_model::aggregate::chart::EventData::Created(data) => {
                ChartEventDataDocumentData::Created(super::schema::Created {
                    title: data.title.to_owned(),
                })
            }
            write_model::aggregate::chart::EventData::Deleted(_) => {
                ChartEventDataDocumentData::Deleted(super::schema::Deleted {})
            }
            write_model::aggregate::chart::EventData::Updated(data) => {
                ChartEventDataDocumentData::Updated(super::schema::Updated {
                    title: data.title.to_owned(),
                })
            }
        }
    }
}

mod path {
    use std::str::FromStr as _;

    use firestore_client::{
        path::{CollectionId, DocumentId},
        CollectionPath, DocumentPath,
    };
    use write_model::value_object::{ChartId, EventId};

    pub(crate) fn chart_collection() -> CollectionPath {
        CollectionPath::new(None, chart_collection_id())
    }

    pub(crate) fn chart_collection_id() -> CollectionId {
        CollectionId::from_str("charts").expect("chart collection id to be valid collection id")
    }

    pub(crate) fn chart_document(chart_id: ChartId) -> DocumentPath {
        chart_collection()
            .doc(
                DocumentId::from_str(&chart_id.to_string())
                    .expect("chart id to be valid document id"),
            )
            .expect("chart document path to be valid document path")
    }

    // events
    // - event_id (pk)
    // - event_stream_id + version (uk)
    //
    // event_streams
    // - event_stream_id (pk)

    pub(crate) fn event_collection_id() -> CollectionId {
        CollectionId::from_str("events").expect("event collection id to be valid collection id")
    }

    pub(crate) fn event_collection() -> CollectionPath {
        CollectionPath::new(None, event_collection_id())
    }

    #[allow(dead_code)]
    pub(crate) fn event_document(event_id: EventId) -> DocumentPath {
        event_collection()
            .doc(
                DocumentId::from_str(&event_id.to_string())
                    .expect("event id to be valid document id"),
            )
            .expect("event document path to be valid document path")
    }

    pub(crate) fn event_stream_collection_id() -> CollectionId {
        CollectionId::from_str("event_streams")
            .expect("event_stream collection id to be valid collection id")
    }

    pub(crate) fn event_stream_collection() -> CollectionPath {
        CollectionPath::new(None, event_stream_collection_id())
    }

    pub(crate) fn event_stream_document(event_stream_id: &str) -> DocumentPath {
        event_stream_collection()
            .doc(DocumentId::from_str(event_stream_id).expect("chart id to be valid document id"))
            .expect("chart event stream document path to be valid document path")
    }
}

mod schema {
    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub(crate) struct ChartDocumentData {
        pub(crate) created_at: String,
        pub(crate) title: String,
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub(crate) struct EventStreamDocumentData {
        pub(crate) id: String,
        pub(crate) last_event_at: String,
        pub(crate) version: i64,
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub(crate) struct EventDocumentData<T> {
        pub(crate) at: String,
        pub(crate) data: T,
        pub(crate) id: String,
        pub(crate) stream_id: String,
        pub(crate) version: i64,
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "snake_case", tag = "type")]
    pub(crate) enum ChartEventDataDocumentData {
        Created(Created),
        Deleted(Deleted),
        Updated(Updated),
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct Created {
        pub(crate) title: String,
    }

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct Deleted {}

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub struct Updated {
        pub(crate) title: String,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_created() -> anyhow::Result<()> {
            assert_eq!(
                serde_json::to_value(EventDocumentData {
                    at: "2020-01-02T03:04:05.678Z".to_owned(),
                    data: ChartEventDataDocumentData::Created(Created {
                        title: "title".to_owned(),
                    }),
                    id: "id".to_owned(),
                    stream_id: "stream_id".to_owned(),
                    version: 1,
                })?,
                serde_json::json!({
                    "at": "2020-01-02T03:04:05.678Z",
                    "data": {
                        "title": "title",
                        "type": "created",
                    },
                    "id": "id",
                    "stream_id": "stream_id",
                    "version": 1,
                }),
            );
            Ok(())
        }

        #[test]
        fn test_deleted() -> anyhow::Result<()> {
            assert_eq!(
                serde_json::to_value(EventDocumentData {
                    at: "2020-01-02T03:04:05.678Z".to_owned(),
                    data: ChartEventDataDocumentData::Deleted(Deleted {}),
                    id: "id".to_owned(),
                    stream_id: "stream_id".to_owned(),
                    version: 1,
                })?,
                serde_json::json!({
                    "at": "2020-01-02T03:04:05.678Z",
                    "data": {
                        "type": "deleted",
                    },
                    "id": "id",
                    "stream_id": "stream_id",
                    "version": 1,
                }),
            );
            Ok(())
        }

        #[test]
        fn test_updated() -> anyhow::Result<()> {
            assert_eq!(
                serde_json::to_value(EventDocumentData {
                    at: "2020-01-02T03:04:05.678Z".to_owned(),
                    data: ChartEventDataDocumentData::Updated(Updated {
                        title: "title".to_owned(),
                    }),
                    id: "id".to_owned(),
                    stream_id: "stream_id".to_owned(),
                    version: 1,
                })?,
                serde_json::json!({
                    "at": "2020-01-02T03:04:05.678Z",
                    "data": {
                        "title": "title",
                        "type": "updated",
                    },
                    "id": "id",
                    "stream_id": "stream_id",
                    "version": 1,
                }),
            );
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use command_use_case::port::ChartRepository as _;

    use super::*;

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
