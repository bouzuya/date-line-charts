use std::{str::FromStr as _, sync::Arc};

use write_model::value_object::DataPointId;

#[cfg(any(test, feature = "test-util"))]
use crate::command_use_case::port::DataPointRepository;
use crate::port::HasDataPointRepository;

#[derive(Debug)]
pub struct Input {
    pub data_point_id: String,
}

#[derive(Debug)]
pub struct Output;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("data point delete")]
    DataPointDelete(#[source] write_model::aggregate::data_point::Error),
    #[error("data point find")]
    DataPointFind(#[source] crate::command_use_case::port::data_point_repository::Error),
    #[error("data point id")]
    DataPointId(#[source] write_model::value_object::data_point_id::Error),
    #[error("data point not found (id = {0})")]
    DataPointNotFound(DataPointId),
    #[error("data point store")]
    DataPointStore(#[source] crate::command_use_case::port::data_point_repository::Error),
}

#[async_trait::async_trait]
pub trait DeleteDataPoint: HasDataPointRepository {
    #[tracing::instrument(level = tracing::Level::INFO, err(Debug), ret, skip(self))]
    async fn execute(&self, Input { data_point_id }: Input) -> Result<Output, Error> {
        let data_point_repository = self.data_point_repository();
        let data_point_id = DataPointId::from_str(&data_point_id).map_err(Error::DataPointId)?;
        let data_point = data_point_repository
            .find(data_point_id)
            .await
            .map_err(Error::DataPointFind)?
            .ok_or(Error::DataPointNotFound(data_point_id))?;
        let (_, events) = data_point.delete().map_err(Error::DataPointDelete)?;
        data_point_repository
            .store(Some(data_point.version()), &events)
            .await
            .map_err(Error::DataPointStore)?;
        Ok(Output)
    }
}

#[cfg(any(test, feature = "test-util"))]
mockall::mock! {
    pub DeleteDataPoint {}

    impl HasDataPointRepository for DeleteDataPoint {
        fn data_point_repository(&self) -> Arc<dyn DataPointRepository + Send + Sync>;
    }

    #[async_trait::async_trait]
    impl DeleteDataPoint for DeleteDataPoint {
        async fn execute(&self, input: Input) -> Result<Output, Error>;
    }
}

pub trait HasDeleteDataPoint {
    fn delete_data_point(&self) -> Arc<dyn DeleteDataPoint + Send + Sync>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock() {
        let mut mock = MockDeleteDataPoint::new();
        mock.expect_execute().return_once(|_| Ok(Output));
    }

    // TODO: test execute
}
