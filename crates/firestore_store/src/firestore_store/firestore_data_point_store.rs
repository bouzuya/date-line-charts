use crate::{converter, path, schema::DataPointDocumentData};
use firestore_client::FirestoreClient;
use write_model::value_object::{ChartId, DataPointId};

pub struct FirestoreDataPointStore(FirestoreClient);

impl FirestoreDataPointStore {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self(FirestoreClient::new().await?))
    }

    async fn reader_get_impl(
        &self,
        id: DataPointId,
    ) -> Result<
        Option<query_use_case::port::DataPointQueryData>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        self.0
            .get_document::<DataPointDocumentData>(&path::data_point_document(id))
            .await?
            .map(converter::data_point_query_data_from_document)
            .transpose()
    }

    async fn reader_list_impl(
        &self,
        chart_id: ChartId,
    ) -> Result<
        Vec<query_use_case::port::DataPointQueryData>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let documents = self
            .0
            .list_all_documents::<DataPointDocumentData>(&path::data_point_collection(chart_id))
            .await?;
        let documents = documents
            .into_iter()
            .map(converter::data_point_query_data_from_document)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(documents)
    }
}

#[async_trait::async_trait]
impl query_use_case::port::DataPointReader for FirestoreDataPointStore {
    async fn get(
        &self,
        id: DataPointId,
    ) -> Result<
        Option<query_use_case::port::DataPointQueryData>,
        query_use_case::port::data_point_reader::Error,
    > {
        self.reader_get_impl(id)
            .await
            .map_err(query_use_case::port::data_point_reader::Error::from)
    }

    async fn list(
        &self,
        chart_id: ChartId,
    ) -> Result<
        Vec<query_use_case::port::DataPointQueryData>,
        query_use_case::port::data_point_reader::Error,
    > {
        self.reader_list_impl(chart_id)
            .await
            .map_err(query_use_case::port::data_point_reader::Error::from)
    }
}
