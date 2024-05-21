use std::sync::Arc;

use write_model::value_object::{ChartId, DateTime};

#[derive(Clone)]
pub struct ChartQueryData {
    pub created_at: DateTime,
    pub id: ChartId,
    pub title: String,
}

#[async_trait::async_trait]
pub trait ChartReader {
    async fn get(
        &self,
        id: ChartId,
    ) -> Result<ChartQueryData, Box<dyn std::error::Error + Send + Sync>>;
    async fn list(&self) -> Result<Vec<ChartQueryData>, Box<dyn std::error::Error + Send + Sync>>;
}

pub trait HasChartReader {
    fn chart_reader(&self) -> Arc<dyn ChartReader + Send + Sync>;
}
