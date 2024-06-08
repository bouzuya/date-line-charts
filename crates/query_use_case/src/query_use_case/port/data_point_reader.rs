use std::sync::Arc;

use write_model::value_object::{ChartId, DataPointId, DateTime, XValue, YValue};

#[derive(Clone, Debug)]
pub struct DataPointQueryData {
    pub chart_id: ChartId,
    pub created_at: DateTime,
    pub x_value: XValue,
    pub y_value: YValue,
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] Box<dyn std::error::Error + Send + Sync>);

#[async_trait::async_trait]
pub trait DataPointReader {
    async fn get(&self, id: DataPointId) -> Result<DataPointQueryData, Error>;
    async fn list(&self, chart_id: ChartId) -> Result<Vec<DataPointQueryData>, Error>;
}

pub trait HasDataPointReader {
    fn data_point_reader(&self) -> Arc<dyn DataPointReader + Send + Sync>;
}
