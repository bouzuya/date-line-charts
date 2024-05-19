use std::sync::Arc;

pub struct Input {
    pub title: String,
}

pub struct Output {
    pub chart_id: String,
}

#[derive(Debug, thiserror::Error)]
#[error("FIXME")]
pub struct Error;

#[cfg_attr(any(test, feature = "test-util"), mockall::automock)]
#[async_trait::async_trait]
pub trait CreateChart {
    async fn execute(&self, input: Input) -> Result<Output, Error>;
}

pub trait HasCreateChart {
    fn create_chart(&self) -> Arc<dyn CreateChart + Send + Sync>;
}
