use std::sync::Arc;

use write_model::{
    aggregate::{data_point::Event, DataPoint},
    value_object::{DataPointId, Version},
};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] Box<dyn std::error::Error + Send + Sync>);

#[async_trait::async_trait]
pub trait DataPointRepository {
    async fn find(&self, id: DataPointId) -> Result<Option<DataPoint>, Error>;
    async fn store(&self, current: Option<Version>, events: &[Event]) -> Result<(), Error>;
}

pub trait HasDataPointRepository {
    fn data_point_repository(&self) -> Arc<dyn DataPointRepository + Send + Sync>;
}
