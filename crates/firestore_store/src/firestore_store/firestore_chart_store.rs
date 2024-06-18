use firestore_client::FirestoreClient;
use schema::ChartDocumentData;
use write_model::{
    aggregate::{chart::Event, Chart},
    value_object::{ChartId, Version},
};

pub struct FirestoreChartStore(FirestoreClient);

impl FirestoreChartStore {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self(FirestoreClient::new().await?))
    }

    async fn get_impl(
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

    async fn list_impl(
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
        self.get_impl(id)
            .await
            .map_err(query_use_case::port::chart_reader::Error::from)
    }

    async fn list(
        &self,
    ) -> Result<Vec<query_use_case::port::ChartQueryData>, query_use_case::port::chart_reader::Error>
    {
        self.list_impl()
            .await
            .map_err(query_use_case::port::chart_reader::Error::from)
    }
}

#[async_trait::async_trait]
impl command_use_case::port::ChartRepository for FirestoreChartStore {
    async fn find(
        &self,
        _id: ChartId,
    ) -> Result<Option<Chart>, command_use_case::port::chart_repository::Error> {
        todo!()
    }

    async fn store(
        &self,
        _current: Option<Version>,
        _events: &[Event],
    ) -> Result<(), command_use_case::port::chart_repository::Error> {
        todo!()
    }
}

mod converter {
    use std::str::FromStr as _;

    use firestore_client::Document;
    use write_model::value_object::{ChartId, DateTime};

    use super::schema::ChartDocumentData;

    pub(crate) fn query_data_from_document(
        document: Document<ChartDocumentData>,
    ) -> Result<query_use_case::port::ChartQueryData, Box<dyn std::error::Error + Send + Sync>>
    {
        Ok(query_use_case::port::ChartQueryData {
            // FIXME: date time format
            created_at: DateTime::now(), // document.fields.created_at
            id: ChartId::from_str(document.name.document_id().as_ref())?,
            title: document.fields.title,
        })
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

    pub(crate) fn chart_event_collection_id() -> CollectionId {
        CollectionId::from_str("events")
            .expect("chart event collection id to be valid collection id")
    }

    pub(crate) fn chart_event_collection(chart_id: ChartId) -> CollectionPath {
        chart_event_stream_document(chart_id)
            .collection(chart_event_collection_id())
            .expect("chart event collection path to be valid collection path")
    }

    pub(crate) fn chart_event_document(chart_id: ChartId, event_id: EventId) -> DocumentPath {
        chart_event_collection(chart_id)
            .doc(
                DocumentId::from_str(&event_id.to_string())
                    .expect("event id to be valid document id"),
            )
            .expect("chart event document path to be valid document path")
    }

    pub(crate) fn chart_event_stream_collection_id() -> CollectionId {
        CollectionId::from_str("chart_event_streams")
            .expect("chart event stream collection id to be valid collection id")
    }

    pub(crate) fn chart_event_stream_collection() -> CollectionPath {
        CollectionPath::new(None, chart_event_stream_collection_id())
    }

    pub(crate) fn chart_event_stream_document(chart_id: ChartId) -> DocumentPath {
        chart_event_stream_collection()
            .doc(
                DocumentId::from_str(&chart_id.to_string())
                    .expect("chart id to be valid document id"),
            )
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
}
