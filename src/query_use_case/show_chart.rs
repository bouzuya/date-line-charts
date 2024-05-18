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

#[axum::async_trait]
pub trait ShowChart: Send + Sync {
    async fn execute(&self, input: Input) -> Result<Output, Error>;
}

pub trait HasShowChart {
    type ShowChart: ShowChart;
    fn show_chart(&self) -> Self::ShowChart;
}
