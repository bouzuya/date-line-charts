pub struct Input;

pub struct Output(pub Vec<Chart>);

pub struct Chart {
    pub created_at: String,
    pub id: String,
    pub title: String,
}

#[derive(Debug, thiserror::Error)]
#[error("FIXME")]
pub struct Error;

#[axum::async_trait]
pub trait ListCharts: Send + Sync {
    async fn execute(&self, input: Input) -> Result<Output, Error>;
}

pub trait HasListCharts {
    type ListCharts: ListCharts;
    fn list_charts(&self) -> Self::ListCharts;
}
