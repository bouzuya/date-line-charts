use std::{str::FromStr as _, sync::Arc};

use write_model::value_object::ChartId;

use crate::port::{DataPointQueryData, HasDataPointReader};
#[cfg(any(test, feature = "test-util"))]
use crate::query_use_case::port::DataPointReader;

pub struct Input {
    pub chart_id: String,
}

pub struct Output(pub Vec<DataPoint>);

#[derive(Clone)]
pub struct DataPoint {
    pub chart_id: String,
    pub created_at: String,
    pub x_value: String,
    pub y_value: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("chart id")]
    ChartId(#[source] write_model::value_object::chart_id::Error),
    #[error("data point list")]
    DataPointList(#[source] Box<dyn std::error::Error + Send + Sync>),
}

#[async_trait::async_trait]
pub trait ListDataPoints: HasDataPointReader {
    async fn execute(&self, Input { chart_id }: Input) -> Result<Output, Error> {
        let data_point_reader = self.data_point_reader();
        let chart_id = ChartId::from_str(&chart_id).map_err(Error::ChartId)?;
        data_point_reader
            .list(chart_id)
            .await
            .map(|data_points| {
                data_points
                    .into_iter()
                    .map(
                        |DataPointQueryData {
                             chart_id,
                             created_at,
                             x_value,
                             y_value,
                         }| DataPoint {
                            chart_id: chart_id.to_string(),
                            created_at: created_at.to_string(),
                            x_value: x_value.to_string(),
                            y_value: u32::from(y_value),
                        },
                    )
                    .collect()
            })
            .map(Output)
            .map_err(Error::DataPointList)
    }
}

#[cfg(any(test, feature = "test-util"))]
mockall::mock! {
    pub ListDataPoints {}

    impl HasDataPointReader for ListDataPoints {
        fn data_point_reader(&self) -> Arc<dyn DataPointReader + Send + Sync>;
    }

    #[async_trait::async_trait]
    impl ListDataPoints for ListDataPoints {
        async fn execute(&self, input: Input) -> Result<Output, Error>;
    }
}

pub trait HasListDataPoints {
    fn list_data_points(&self) -> Arc<dyn ListDataPoints + Send + Sync>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock() {
        let mut mock = MockListDataPoints::new();
        mock.expect_execute().return_once(|_| {
            Ok(Output(vec![DataPoint {
                chart_id: "chart_id".to_string(),
                created_at: "2021-08-21T00:00:00Z".to_string(),
                x_value: "2020-01-02".to_string(),
                y_value: 2,
            }]))
        });
    }
}
