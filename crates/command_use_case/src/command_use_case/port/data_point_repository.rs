use std::sync::Arc;

use write_model::{
    aggregate::{data_point::Event, DataPoint},
    value_object::{DataPointId, Version},
};

#[async_trait::async_trait]
pub trait DataPointRepository {
    async fn find(
        &self,
        id: DataPointId,
    ) -> Result<Option<DataPoint>, Box<dyn std::error::Error + Send + Sync>>;
    async fn store(
        &self,
        current: Option<Version>,
        events: &[Event],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub trait HasDataPointRepository {
    fn data_point_repository(&self) -> Arc<dyn DataPointRepository + Send + Sync>;
}
