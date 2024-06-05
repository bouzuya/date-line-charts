use std::sync::Arc;

use write_model::value_object::{ChartId, DateTime};

#[derive(Clone)]
pub struct ChartQueryData {
    pub created_at: DateTime,
    pub id: ChartId,
    pub title: String,
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] Box<dyn std::error::Error + Send + Sync>);

#[async_trait::async_trait]
pub trait ChartReader {
    async fn get(&self, id: ChartId) -> Result<ChartQueryData, Error>;
    async fn list(&self) -> Result<Vec<ChartQueryData>, Error>;
}

pub trait HasChartReader {
    fn chart_reader(&self) -> Arc<dyn ChartReader + Send + Sync>;
}
