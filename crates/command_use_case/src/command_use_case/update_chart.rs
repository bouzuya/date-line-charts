use std::{str::FromStr as _, sync::Arc};

use write_model::value_object::ChartId;

#[cfg(any(test, feature = "test-util"))]
use crate::command_use_case::port::ChartRepository;
use crate::port::HasChartRepository;

#[derive(Debug)]
pub struct Input {
    pub chart_id: String,
    pub title: String,
}

#[derive(Debug)]
pub struct Output;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("chart find")]
    ChartFind(#[source] crate::command_use_case::port::chart_repository::Error),
    #[error("chart id")]
    ChartId(#[source] write_model::value_object::chart_id::Error),
    #[error("chart not found")]
    ChartNotFound(write_model::value_object::ChartId),
    #[error("chart store")]
    ChartStore(#[source] crate::command_use_case::port::chart_repository::Error),
    #[error("chart update")]
    ChartUpdate(#[source] write_model::aggregate::chart::Error),
}

#[async_trait::async_trait]
pub trait UpdateChart: HasChartRepository {
    #[tracing::instrument(level = tracing::Level::INFO, err(Debug), ret, skip(self))]
    async fn execute(&self, input: Input) -> Result<Output, Error> {
        let chart_repository = self.chart_repository();
        let chart_id = ChartId::from_str(&input.chart_id).map_err(Error::ChartId)?;
        let chart = chart_repository
            .find(chart_id)
            .await
            .map_err(Error::ChartFind)?
            .ok_or(Error::ChartNotFound(chart_id))?;
        let (_, events) = chart.update(input.title).map_err(Error::ChartUpdate)?;
        chart_repository
            .store(Some(chart.version()), &events)
            .await
            .map_err(Error::ChartStore)?;
        Ok(Output)
    }
}

#[cfg(any(test, feature = "test-util"))]
mockall::mock! {
    pub UpdateChart {}

    impl HasChartRepository for UpdateChart {
        fn chart_repository(&self) -> Arc<dyn ChartRepository + Send + Sync>;
    }

    #[async_trait::async_trait]
    impl UpdateChart for UpdateChart {
        async fn execute(&self, input: Input) -> Result<Output, Error>;
    }
}

pub trait HasUpdateChart {
    fn update_chart(&self) -> Arc<dyn UpdateChart + Send + Sync>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock() {
        let mut mock = MockUpdateChart::new();
        mock.expect_execute().return_once(|_| Ok(Output));
    }

    // TODO: test execute
}
