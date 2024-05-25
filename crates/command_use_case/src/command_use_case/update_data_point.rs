use std::{str::FromStr as _, sync::Arc};

use write_model::value_object::{DataPointId, YValue};

#[cfg(any(test, feature = "test-util"))]
use crate::command_use_case::port::DataPointRepository;
use crate::port::HasDataPointRepository;

pub struct Input {
    pub data_point_id: String,
    pub y_value: u32,
}

pub struct Output;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("data point find")]
    DataPointFind(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("data point id")]
    DataPointId(#[source] write_model::value_object::data_point_id::Error),
    #[error("data point not found")]
    DataPointNotFound(write_model::value_object::DataPointId),
    #[error("data point store")]
    DataPointStore(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("data point update")]
    DataPointUpdate(#[source] write_model::aggregate::data_point::Error),
}

#[async_trait::async_trait]
pub trait UpdateDataPoint: HasDataPointRepository {
    async fn execute(
        &self,
        Input {
            data_point_id,
            y_value,
        }: Input,
    ) -> Result<Output, Error> {
        let data_point_repository = self.data_point_repository();

        let data_point_id = DataPointId::from_str(&data_point_id).map_err(Error::DataPointId)?;
        let y_value = YValue::from(y_value);

        let data_point = data_point_repository
            .find(data_point_id)
            .await
            .map_err(Error::DataPointFind)?
            .ok_or(Error::DataPointNotFound(data_point_id))?;
        let (_, events) = data_point.update(y_value).map_err(Error::DataPointUpdate)?;
        data_point_repository
            .store(Some(data_point.version()), &events)
            .await
            .map_err(Error::DataPointStore)?;
        Ok(Output)
    }
}

#[cfg(any(test, feature = "test-util"))]
mockall::mock! {
    pub UpdateDataPoint {}

    impl HasDataPointRepository for UpdateDataPoint {
        fn data_point_repository(&self) -> Arc<dyn DataPointRepository + Send + Sync>;
    }

    #[async_trait::async_trait]
    impl UpdateDataPoint for UpdateDataPoint {
        async fn execute(&self, input: Input) -> Result<Output, Error>;
    }
}

pub trait HasUpdateDataPoint {
    fn update_data_point(&self) -> Arc<dyn UpdateDataPoint + Send + Sync>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock() {
        let mut mock = MockUpdateDataPoint::new();
        mock.expect_execute().return_once(|_| Ok(Output));
    }

    // TODO: test execute
}
