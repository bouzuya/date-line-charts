use std::sync::Arc;

pub struct Input;

pub struct Output(pub Vec<Chart>);

#[derive(Clone)]
pub struct Chart {
    pub created_at: String,
    pub id: String,
    pub title: String,
}

#[derive(Debug, thiserror::Error)]
#[error("FIXME")]
pub struct Error;

#[cfg_attr(any(test, feature = "test-util"), mockall::automock)]
#[async_trait::async_trait]
pub trait ListCharts {
    async fn execute(&self, input: Input) -> Result<Output, Error>;
}

pub trait HasListCharts {
    fn list_charts(&self) -> Arc<dyn ListCharts + Send + Sync>;
}
