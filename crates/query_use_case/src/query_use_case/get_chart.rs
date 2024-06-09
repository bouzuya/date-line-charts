use std::{str::FromStr as _, sync::Arc};

use write_model::value_object::ChartId;

use crate::port::{ChartQueryData, HasChartReader};
#[cfg(any(test, feature = "test-util"))]
use crate::query_use_case::port::ChartReader;

pub struct Input {
    pub chart_id: String,
}

pub struct Output(pub Option<OutputItem>);

pub struct OutputItem {
    pub created_at: String,
    pub id: String,
    pub title: String,
}

impl From<ChartQueryData> for OutputItem {
    fn from(
        ChartQueryData {
            created_at,
            id,
            title,
        }: ChartQueryData,
    ) -> Self {
        Self {
            created_at: created_at.to_string(),
            id: id.to_string(),
            title,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("chart get")]
    ChartGet(#[source] crate::port::chart_reader::Error),
    #[error("chart id")]
    ChartId(#[source] write_model::value_object::chart_id::Error),
}

#[async_trait::async_trait]
pub trait GetChart: HasChartReader {
    async fn execute(&self, input: Input) -> Result<Output, Error> {
        let chart_reader = self.chart_reader();
        let chart_id = ChartId::from_str(&input.chart_id).map_err(Error::ChartId)?;
        Ok(Output(
            chart_reader
                .get(chart_id)
                .await
                .map_err(Error::ChartGet)?
                .map(OutputItem::from),
        ))
    }
}

#[cfg(any(test, feature = "test-util"))]
mockall::mock! {
    pub GetChart {}

    impl HasChartReader for GetChart {
        fn chart_reader(&self) -> Arc<dyn ChartReader + Send + Sync>;
    }

    #[async_trait::async_trait]
    impl GetChart for GetChart {
        async fn execute(&self, input: Input) -> Result<Output, Error>;
    }
}

pub trait HasGetChart {
    fn get_chart(&self) -> Arc<dyn GetChart + Send + Sync>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut mock = MockGetChart::new();
        mock.expect_execute().return_once(|_| {
            Ok(Output(Some(OutputItem {
                created_at: "created_at".to_string(),
                id: "id".to_string(),
                title: "title".to_string(),
            })))
        });
    }

    // TODO: test execute
}
