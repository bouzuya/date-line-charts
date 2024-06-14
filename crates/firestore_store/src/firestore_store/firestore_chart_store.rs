use std::str::FromStr;

use firestore_client::{CollectionPath, Document, DocumentPath, FirestoreClient, Timestamp};
use write_model::value_object::{ChartId, DateTime};

pub struct FirestoreChartStore(FirestoreClient);

impl FirestoreChartStore {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let project_id = "demo-project1";
        Ok(Self(
            FirestoreClient::new(
                project_id,
                ["https://www.googleapis.com/auth/cloud-platform"],
            )
            .await?,
        ))
    }

    async fn get_impl(
        &self,
        id: ChartId,
    ) -> Result<
        Option<query_use_case::port::ChartQueryData>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let document = self
            .0
            .get_document::<ChartDocumentData>(
                DocumentPath::from_str(&format!("charts/{}", id)).unwrap(),
            )
            .await?;
        let document = query_data_from_document(document)?;
        // FIXME: not found => None
        Ok(Some(document))
    }

    async fn list_impl(
        &self,
    ) -> Result<Vec<query_use_case::port::ChartQueryData>, Box<dyn std::error::Error + Send + Sync>>
    {
        let documents = self
            .0
            .list_all_documents::<ChartDocumentData>(&CollectionPath::from_str("charts").unwrap())
            .await?;
        let documents = documents
            .into_iter()
            .map(query_data_from_document)
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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct ChartDocumentData {
    created_at: Timestamp,
    title: String,
}

fn query_data_from_document(
    document: Document<ChartDocumentData>,
) -> Result<query_use_case::port::ChartQueryData, Box<dyn std::error::Error + Send + Sync>> {
    Ok(query_use_case::port::ChartQueryData {
        // FIXME: date time format
        created_at: DateTime::now(), // document.fields.created_at
        id: ChartId::from_str(document.name.document_id().as_ref())?,
        title: document.fields.title,
    })
}
