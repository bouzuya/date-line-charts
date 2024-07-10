use std::sync::Arc;

use write_model::{
    aggregate::Chart,
    event::ChartEvent,
    value_object::{ChartId, Version},
};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] Box<dyn std::error::Error + Send + Sync>);

#[async_trait::async_trait]
pub trait ChartRepository {
    async fn find(&self, id: ChartId) -> Result<Option<Chart>, Error>;
    async fn store(&self, current: Option<Version>, events: &[ChartEvent]) -> Result<(), Error>;
}

pub trait HasChartRepository {
    fn chart_repository(&self) -> Arc<dyn ChartRepository + Send + Sync>;
}
