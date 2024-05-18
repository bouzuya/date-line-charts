pub struct Input {
    pub title: String,
}

pub struct Output {
    pub chart_id: String,
}

#[derive(Debug, thiserror::Error)]
#[error("FIXME")]
pub struct Error;

#[axum::async_trait]
pub trait CreateChart: Send + Sync {
    async fn execute(&self, input: Input) -> Result<Output, Error>;
}

pub trait HasCreateChart {
    type CreateChart: CreateChart;
    fn create_chart(&self) -> Self::CreateChart;
}
