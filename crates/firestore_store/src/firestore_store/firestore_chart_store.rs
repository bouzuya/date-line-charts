use std::str::FromStr;

use converter::document_data_from_chart_event_data;
use firestore_client::{FieldPath, Filter, FirestoreClient};
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

        let event_stream_id = EventStreamId::from_str(events[0].stream_id.to_string().as_str())?;
        let last_event_version = events
            .last()
            .expect("events to have at least one element")
            .version;
        let last_event_at = events
            .last()
            .expect("events to have at least one element")
            .at;
        let mut transaction = self.0.begin_transaction().await?;
        match current {
            None => {
                // create event_stream
                // TODO: rollback
                transaction.create(
                    &path::event_stream_document(event_stream_id.to_string().as_str()),
                    &EventStreamDocumentData {
                        id: event_stream_id.to_string(),
                        last_event_at: last_event_at.to_string(),
                        version: i64::from(last_event_version),
                    },
                )?;
                // create events
                for event in events {
                    // TODO: rollback
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
            }
            Some(_current) => {
                // get event_stream with lock
                // check version
                // update event_stream
                // create events
                todo!()
            }
        }
        transaction.commit().await?;
        Ok(())
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
            created_at: DateTime::from_unix_timestamp_millis(
                document.fields.created_at.seconds * 1_000
                    + i64::from(document.fields.created_at.nanos / 1_000_000),
            )?,
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
    use firestore_client::Timestamp;

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub(crate) struct ChartDocumentData {
        pub(crate) created_at: Timestamp,
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
