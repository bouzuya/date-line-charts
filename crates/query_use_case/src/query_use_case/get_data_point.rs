use std::{str::FromStr as _, sync::Arc};

use write_model::value_object::DataPointId;

use crate::port::{DataPointQueryData, HasDataPointReader};
#[cfg(any(test, feature = "test-util"))]
use crate::query_use_case::port::DataPointReader;

pub struct Input {
    pub data_point_id: String,
}

pub struct Output {
    pub chart_id: String,
    pub created_at: String,
    pub x_value: String,
    pub y_value: u32,
}

impl From<DataPointQueryData> for Output {
    fn from(
        DataPointQueryData {
            chart_id,
            created_at,
            x_value,
            y_value,
        }: DataPointQueryData,
    ) -> Self {
        Self {
            chart_id: chart_id.to_string(),
            created_at: created_at.to_string(),
            x_value: x_value.to_string(),
            y_value: u32::from(y_value),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("data point get")]
    DataPointGet(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("data point id")]
    DataPointId(#[source] write_model::value_object::data_point_id::Error),
}

#[async_trait::async_trait]
pub trait GetDataPoint: HasDataPointReader {
    async fn execute(&self, Input { data_point_id }: Input) -> Result<Output, Error> {
        let data_point_reader = self.data_point_reader();
        let data_point_id = DataPointId::from_str(&data_point_id).map_err(Error::DataPointId)?;
        data_point_reader
            .get(data_point_id)
            .await
            .map(Output::from)
            .map_err(Error::DataPointGet)
    }
}

#[cfg(any(test, feature = "test-util"))]
mockall::mock! {
    pub GetDataPoint {}

    impl HasDataPointReader for GetDataPoint {
        fn data_point_reader(&self) -> Arc<dyn DataPointReader + Send + Sync>;
    }

    #[async_trait::async_trait]
    impl GetDataPoint for GetDataPoint {
        async fn execute(&self, input: Input) -> Result<Output, Error>;
    }
}

pub trait HasGetDataPoint {
    fn data_point_chart(&self) -> Arc<dyn GetDataPoint + Send + Sync>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut mock = MockGetDataPoint::new();
        mock.expect_execute().return_once(|_| {
            Ok(Output {
                chart_id: "chart_id".to_string(),
                created_at: "2021-08-21T00:00:00Z".to_string(),
                x_value: "2020-01-02".to_string(),
                y_value: 2,
            })
        });
    }

    // TODO: test execute
}
