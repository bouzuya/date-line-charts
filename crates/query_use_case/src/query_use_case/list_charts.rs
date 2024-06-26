use std::sync::Arc;

use crate::port::{ChartQueryData, HasChartReader};
#[cfg(any(test, feature = "test-util"))]
use crate::query_use_case::port::ChartReader;

pub struct Input;

pub struct Output(pub Vec<Chart>);

#[derive(Clone)]
pub struct Chart {
    pub created_at: String,
    pub id: String,
    pub title: String,
}

impl From<ChartQueryData> for Chart {
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
    #[error("chart list")]
    ChartList(#[source] crate::port::chart_reader::Error),
}

#[async_trait::async_trait]
pub trait ListCharts: HasChartReader {
    async fn execute(&self, _: Input) -> Result<Output, Error> {
        let chart_reader = self.chart_reader();
        chart_reader
            .list()
            .await
            .map(|charts| Output(charts.into_iter().map(Chart::from).collect()))
            .map_err(Error::ChartList)
    }
}

#[cfg(any(test, feature = "test-util"))]
mockall::mock! {
    pub ListCharts {}

    impl HasChartReader for ListCharts {
        fn chart_reader(&self) -> Arc<dyn ChartReader + Send + Sync>;
    }

    #[async_trait::async_trait]
    impl ListCharts for ListCharts {
        async fn execute(&self, input: Input) -> Result<Output, Error>;
    }
}

pub trait HasListCharts {
    fn list_charts(&self) -> Arc<dyn ListCharts + Send + Sync>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock() {
        let mut mock = MockListCharts::new();
        mock.expect_execute().return_once(|_| {
            Ok(Output(vec![Chart {
                created_at: "created_at".to_string(),
                id: "id".to_string(),
                title: "title".to_string(),
            }]))
        });
    }
}
