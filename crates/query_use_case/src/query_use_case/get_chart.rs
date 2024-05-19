use std::sync::Arc;

pub struct Input {
    pub chart_id: String,
}

pub struct Output {
    pub created_at: String,
    pub id: String,
    pub title: String,
}

#[derive(Debug, thiserror::Error)]
#[error("FIXME")]
pub struct Error;

#[cfg_attr(any(test, feature = "test-util"), mockall::automock)]
#[async_trait::async_trait]
pub trait GetChart {
    async fn execute(&self, input: Input) -> Result<Output, Error>;
}

pub trait HasGetChart {
    fn get_chart(&self) -> Arc<dyn GetChart + Send + Sync>;
}
