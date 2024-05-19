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

#[async_trait::async_trait]
pub trait GetChart: Send + Sync {
    async fn execute(&self, input: Input) -> Result<Output, Error>;
}

pub trait HasGetChart {
    type GetChart: GetChart;
    fn get_chart(&self) -> Self::GetChart;
}
