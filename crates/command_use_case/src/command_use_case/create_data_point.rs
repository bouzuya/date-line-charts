use std::{str::FromStr as _, sync::Arc};

use write_model::{
    aggregate::DataPoint,
    value_object::{ChartId, XValue, YValue},
};

use crate::port::{
    ChartRepository, DataPointRepository, HasChartRepository, HasDataPointRepository,
};

pub struct Input {
    pub chart_id: String,
    pub x_value: String,
    pub y_value: u32,
}

pub struct Output {
    pub data_point_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("chart find")]
    ChartFind(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("chart id")]
    ChartId(#[source] write_model::value_object::chart_id::Error),
    #[error("chart not found (id = {0})")]
    ChartNotFound(ChartId),
    #[error("data point create")]
    DataPointCreate(#[source] write_model::aggregate::data_point::Error),
    #[error("data point store")]
    DataPointStore(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("x value")]
    XValue(#[source] write_model::value_object::x_value::Error),
}

#[async_trait::async_trait]
pub trait CreateDataPoint: HasChartRepository + HasDataPointRepository {
    async fn execute(
        &self,
        Input {
            chart_id,
            x_value,
            y_value,
        }: Input,
    ) -> Result<Output, Error> {
        let chart_repository = self.chart_repository();
        let data_point_repository = self.data_point_repository();

        let chart_id = ChartId::from_str(&chart_id).map_err(Error::ChartId)?;
        let x_value = XValue::from_str(&x_value).map_err(Error::XValue)?;
        let y_value = YValue::from(y_value);

        let chart = chart_repository
            .find(chart_id)
            .await
            .map_err(Error::ChartFind)?
            .ok_or(Error::ChartNotFound(chart_id))?;

        let (state, events) =
            DataPoint::create(chart.id(), x_value, y_value).map_err(Error::DataPointCreate)?;

        data_point_repository
            .store(None, &events)
            .await
            .map_err(Error::DataPointStore)?;

        Ok(Output {
            data_point_id: state.id().to_string(),
        })
    }
}

#[cfg(any(test, feature = "test-util"))]
mockall::mock! {
    pub CreateDataPoint {}

    impl HasChartRepository for CreateDataPoint {
        fn chart_repository(&self) -> Arc<dyn ChartRepository + Send + Sync>;
    }

    impl HasDataPointRepository for CreateDataPoint {
        fn data_point_repository(&self) -> Arc<dyn DataPointRepository + Send + Sync>;
    }

    #[async_trait::async_trait]
    impl CreateDataPoint for CreateDataPoint {
        async fn execute(&self, input: Input) -> Result<Output, Error>;
    }
}

pub trait HasCreateDataPoint {
    fn create_data_point(&self) -> Arc<dyn CreateDataPoint + Send + Sync>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock() {
        let mut mock = MockCreateDataPoint::new();
        mock.expect_execute().return_once(|_| {
            Ok(Output {
                data_point_id: "test".to_string(),
            })
        });
    }

    // TODO: test execute
}
