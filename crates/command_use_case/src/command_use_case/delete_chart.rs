use std::{str::FromStr as _, sync::Arc};

use write_model::value_object::ChartId;

#[cfg(any(test, feature = "test-util"))]
use crate::command_use_case::port::ChartRepository;
use crate::port::HasChartRepository;

pub struct Input {
    pub chart_id: String,
}

pub struct Output;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("chart delete")]
    ChartDelete(#[source] write_model::aggregate::chart::Error),
    #[error("chart find")]
    ChartFind(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("chart id")]
    ChartId(#[source] write_model::value_object::chart_id::Error),
    #[error("chart not found")]
    ChartNotFound(ChartId),
    #[error("chart store")]
    ChartStore(#[source] Box<dyn std::error::Error + Send + Sync>),
}

#[async_trait::async_trait]
pub trait DeleteChart: HasChartRepository {
    async fn execute(&self, input: Input) -> Result<Output, Error> {
        let chart_repository = self.chart_repository();
        let chart_id = ChartId::from_str(&input.chart_id).map_err(Error::ChartId)?;
        let chart = chart_repository
            .find(chart_id)
            .await
            .map_err(Error::ChartFind)?
            .ok_or(Error::ChartNotFound(chart_id))?;
        let (_, events) = chart.delete().map_err(Error::ChartDelete)?;
        chart_repository
            .store(Some(chart.version()), &events)
            .await
            .map_err(Error::ChartStore)?;
        Ok(Output)
    }
}

#[cfg(any(test, feature = "test-util"))]
mockall::mock! {
    pub DeleteChart {}

    impl HasChartRepository for DeleteChart {
        fn chart_repository(&self) -> Arc<dyn ChartRepository + Send + Sync>;
    }

    #[async_trait::async_trait]
    impl DeleteChart for DeleteChart {
        async fn execute(&self, input: Input) -> Result<Output, Error>;
    }
}

pub trait HasDeleteChart {
    fn delete_chart(&self) -> Arc<dyn DeleteChart + Send + Sync>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock() {
        let mut mock = MockDeleteChart::new();
        mock.expect_execute().return_once(|_| Ok(Output));
    }

    // TODO: test execute
}
