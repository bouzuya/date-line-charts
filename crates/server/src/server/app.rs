use std::sync::Arc;

use command_use_case::port::{ChartRepository, DataPointRepository};
use query_use_case::port::ChartReader;

#[derive(Clone)]
pub struct App {
    chart_reader: Arc<dyn ChartReader + Send + Sync>,
    chart_repository: Arc<dyn ChartRepository + Send + Sync>,
    data_point_repository: Arc<dyn DataPointRepository + Send + Sync>,
}

impl App {
    pub fn new(
        chart_reader: Arc<dyn ChartReader + Send + Sync>,
        chart_repository: Arc<dyn ChartRepository + Send + Sync>,
        data_point_repository: Arc<dyn DataPointRepository + Send + Sync>,
    ) -> Self {
        Self {
            chart_reader,
            chart_repository,
            data_point_repository,
        }
    }
}

impl command_use_case::create_chart::CreateChart for App {}

impl command_use_case::create_chart::HasCreateChart for App {
    fn create_chart(&self) -> Arc<dyn command_use_case::create_chart::CreateChart + Send + Sync> {
        Arc::new(self.clone())
    }
}

impl command_use_case::create_data_point::CreateDataPoint for App {}

impl command_use_case::create_data_point::HasCreateDataPoint for App {
    fn create_data_point(
        &self,
    ) -> Arc<dyn command_use_case::create_data_point::CreateDataPoint + Send + Sync> {
        Arc::new(self.clone())
    }
}

impl command_use_case::delete_chart::DeleteChart for App {}

impl command_use_case::delete_chart::HasDeleteChart for App {
    fn delete_chart(&self) -> Arc<dyn command_use_case::delete_chart::DeleteChart + Send + Sync> {
        Arc::new(self.clone())
    }
}

impl command_use_case::delete_data_point::DeleteDataPoint for App {}

impl command_use_case::delete_data_point::HasDeleteDataPoint for App {
    fn delete_data_point(
        &self,
    ) -> Arc<dyn command_use_case::delete_data_point::DeleteDataPoint + Send + Sync> {
        Arc::new(self.clone())
    }
}

impl command_use_case::port::HasChartRepository for App {
    fn chart_repository(&self) -> Arc<dyn ChartRepository + Send + Sync> {
        self.chart_repository.clone()
    }
}

impl command_use_case::port::HasDataPointRepository for App {
    fn data_point_repository(&self) -> Arc<dyn DataPointRepository + Send + Sync> {
        self.data_point_repository.clone()
    }
}

impl command_use_case::update_chart::HasUpdateChart for App {
    fn update_chart(&self) -> Arc<dyn command_use_case::update_chart::UpdateChart + Send + Sync> {
        Arc::new(self.clone())
    }
}

impl command_use_case::update_chart::UpdateChart for App {}

impl command_use_case::update_data_point::HasUpdateDataPoint for App {
    fn update_data_point(
        &self,
    ) -> Arc<dyn command_use_case::update_data_point::UpdateDataPoint + Send + Sync> {
        Arc::new(self.clone())
    }
}

impl command_use_case::update_data_point::UpdateDataPoint for App {}

impl query_use_case::port::HasChartReader for App {
    fn chart_reader(&self) -> Arc<dyn query_use_case::port::ChartReader + Send + Sync> {
        self.chart_reader.clone()
    }
}

impl query_use_case::get_chart::GetChart for App {}

impl query_use_case::get_chart::HasGetChart for App {
    fn get_chart(&self) -> Arc<dyn query_use_case::get_chart::GetChart + Send + Sync> {
        Arc::new(self.clone())
    }
}

impl query_use_case::list_charts::HasListCharts for App {
    fn list_charts(&self) -> Arc<dyn query_use_case::list_charts::ListCharts + Send + Sync> {
        Arc::new(self.clone())
    }
}

impl query_use_case::list_charts::ListCharts for App {}
