use std::sync::Arc;

use write_model::{
    aggregate::{chart::Event, Chart},
    value_object::{ChartId, Version},
};

#[async_trait::async_trait]
pub trait ChartRepository {
    async fn find(
        &self,
        id: ChartId,
    ) -> Result<Option<Chart>, Box<dyn std::error::Error + Send + Sync>>;
    async fn store(
        &self,
        current: Option<Version>,
        events: &[Event],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait::async_trait]
pub trait HasChartRepository {
    fn chart_repository(&self) -> Arc<dyn ChartRepository + Send + Sync>;
}
