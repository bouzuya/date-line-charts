use std::sync::Arc;

use write_model::aggregate::Chart;

#[cfg(any(test, feature = "test-util"))]
use crate::command_use_case::port::ChartRepository;
use crate::port::HasChartRepository;

pub struct Input {
    pub title: String,
}

pub struct Output {
    pub chart_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("chart create")]
    ChartCreate(#[source] write_model::aggregate::chart::Error),
    #[error("chart store")]
    ChartStore(#[source] crate::command_use_case::port::chart_repository::Error),
}

#[async_trait::async_trait]
pub trait CreateChart: HasChartRepository {
    async fn execute(&self, input: Input) -> Result<Output, Error> {
        let (state, events) = Chart::create(input.title).map_err(Error::ChartCreate)?;
        self.chart_repository()
            .store(None, &events)
            .await
            .map_err(Error::ChartStore)?;
        Ok(Output {
            chart_id: state.id().to_string(),
        })
    }
}

#[cfg(any(test, feature = "test-util"))]
mockall::mock! {
    pub CreateChart {}

    impl HasChartRepository for CreateChart {
        fn chart_repository(&self) -> Arc<dyn ChartRepository + Send + Sync>;
    }

    #[async_trait::async_trait]
    impl CreateChart for CreateChart {
        async fn execute(&self, input: Input) -> Result<Output, Error>;
    }
}

pub trait HasCreateChart {
    fn create_chart(&self) -> Arc<dyn CreateChart + Send + Sync>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock() {
        let mut mock = MockCreateChart::new();
        mock.expect_execute().return_once(|_| {
            Ok(Output {
                chart_id: "test".to_string(),
            })
        });
    }

    // TODO: test execute
}
