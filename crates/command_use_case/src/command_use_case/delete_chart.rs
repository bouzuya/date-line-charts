use std::sync::Arc;

pub struct Input {
    pub chart_id: String,
}

pub struct Output;

#[derive(Debug, thiserror::Error)]
#[error("FIXME")]
pub struct Error;

#[cfg_attr(any(test, feature = "test-util"), mockall::automock)]
#[async_trait::async_trait]
pub trait DeleteChart {
    async fn execute(&self, input: Input) -> Result<Output, Error>;
}

pub trait HasDeleteChart {
    fn delete_chart(&self) -> Arc<dyn DeleteChart + Send + Sync>;
}
