pub struct Input {
    pub chart_id: String,
    pub title: String,
}

pub struct Output;

#[derive(Debug, thiserror::Error)]
#[error("FIXME")]
pub struct Error;

#[axum::async_trait]
pub trait UpdateChart: Send + Sync {
    async fn execute(&self, input: Input) -> Result<Output, Error>;
}

pub trait HasUpdateChart {
    type UpdateChart: UpdateChart;
    fn update_chart(&self) -> Self::UpdateChart;
}
