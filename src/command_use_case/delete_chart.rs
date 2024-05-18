pub struct Input {
    pub chart_id: String,
}

pub struct Output;

#[derive(Debug, thiserror::Error)]
#[error("FIXME")]
pub struct Error;

#[axum::async_trait]
pub trait DeleteChart: Send + Sync {
    async fn execute(&self, input: Input) -> Result<Output, Error>;
}

pub trait HasDeleteChart {
    type DeleteChart: DeleteChart;
    fn delete_chart(&self) -> Self::DeleteChart;
}
