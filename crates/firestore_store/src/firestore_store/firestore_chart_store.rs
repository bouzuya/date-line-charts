use firestore_client::FirestoreClient;
use schema::ChartDocumentData;
use write_model::value_object::ChartId;

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
    use write_model::value_object::ChartId;

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
}

mod schema {
    use firestore_client::Timestamp;

    #[derive(Debug, serde::Deserialize, serde::Serialize)]
    pub(crate) struct ChartDocumentData {
        pub(crate) created_at: Timestamp,
        pub(crate) title: String,
    }
}
