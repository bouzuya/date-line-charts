mod in_memory_chart_store;

use std::{str::FromStr as _, sync::Arc};

use command_use_case::port::{ChartRepository, HasChartRepository};
use query_use_case::port::{ChartQueryData, ChartReader, HasChartReader};
use write_model::{aggregate::Chart, value_object::ChartId};

pub use self::in_memory_chart_store::InMemoryChartStore;

#[derive(Clone)]
pub struct InMemoryApp {
    chart_reader: Arc<dyn ChartReader + Send + Sync>,
    chart_repository: Arc<dyn ChartRepository + Send + Sync>,
}

impl InMemoryApp {
    pub fn new(
        chart_reader: Arc<dyn ChartReader + Send + Sync>,
        chart_repository: Arc<dyn ChartRepository + Send + Sync>,
    ) -> Self {
        Self {
            chart_reader,
            chart_repository,
        }
    }
}

#[async_trait::async_trait]
impl command_use_case::create_chart::CreateChart for InMemoryApp {
    async fn execute(
        &self,
        input: command_use_case::create_chart::Input,
    ) -> Result<command_use_case::create_chart::Output, command_use_case::create_chart::Error> {
        let (state, events) =
            Chart::create(input.title).map_err(|_| command_use_case::create_chart::Error)?;
        self.chart_repository()
            .store(None, &events)
            .await
            .map_err(|_| command_use_case::create_chart::Error)?;
        Ok(command_use_case::create_chart::Output {
            chart_id: state.id().to_string(),
        })
    }
}

impl command_use_case::create_chart::HasCreateChart for InMemoryApp {
    fn create_chart(&self) -> Arc<dyn command_use_case::create_chart::CreateChart + Send + Sync> {
        Arc::new(self.clone())
    }
}

#[async_trait::async_trait]
impl command_use_case::delete_chart::DeleteChart for InMemoryApp {
    async fn execute(
        &self,
        input: command_use_case::delete_chart::Input,
    ) -> Result<command_use_case::delete_chart::Output, command_use_case::delete_chart::Error> {
        let chart_repository = self.chart_repository();
        let chart_id = ChartId::from_str(&input.chart_id)
            .map_err(|_| command_use_case::delete_chart::Error)?;
        let chart = chart_repository
            .find(chart_id)
            .await
            .map_err(|_| command_use_case::delete_chart::Error)?
            .ok_or(command_use_case::delete_chart::Error)?;
        let (_, events) = chart
            .delete()
            .map_err(|_| command_use_case::delete_chart::Error)?;
        chart_repository
            .store(Some(chart.version()), &events)
            .await
            .map_err(|_| command_use_case::delete_chart::Error)?;
        Ok(command_use_case::delete_chart::Output)
    }
}

impl command_use_case::delete_chart::HasDeleteChart for InMemoryApp {
    fn delete_chart(&self) -> Arc<dyn command_use_case::delete_chart::DeleteChart + Send + Sync> {
        Arc::new(self.clone())
    }
}

impl command_use_case::port::HasChartRepository for InMemoryApp {
    fn chart_repository(&self) -> Arc<dyn ChartRepository + Send + Sync> {
        self.chart_repository.clone()
    }
}

impl command_use_case::update_chart::HasUpdateChart for InMemoryApp {
    fn update_chart(&self) -> Arc<dyn command_use_case::update_chart::UpdateChart + Send + Sync> {
        Arc::new(self.clone())
    }
}

#[async_trait::async_trait]
impl command_use_case::update_chart::UpdateChart for InMemoryApp {
    async fn execute(
        &self,
        input: command_use_case::update_chart::Input,
    ) -> Result<command_use_case::update_chart::Output, command_use_case::update_chart::Error> {
        let chart_repository = self.chart_repository();
        let chart_id = ChartId::from_str(&input.chart_id)
            .map_err(|_| command_use_case::update_chart::Error)?;
        let chart = chart_repository
            .find(chart_id)
            .await
            .map_err(|_| command_use_case::update_chart::Error)?
            .ok_or(command_use_case::update_chart::Error)?;
        let (_, events) = chart
            .update(input.title)
            .map_err(|_| command_use_case::update_chart::Error)?;
        chart_repository
            .store(Some(chart.version()), &events)
            .await
            .map_err(|_| command_use_case::update_chart::Error)?;
        Ok(command_use_case::update_chart::Output)
    }
}

impl query_use_case::port::HasChartReader for InMemoryApp {
    fn chart_reader(&self) -> Arc<dyn query_use_case::port::ChartReader + Send + Sync> {
        self.chart_reader.clone()
    }
}

#[async_trait::async_trait]
impl query_use_case::get_chart::GetChart for InMemoryApp {
    async fn execute(
        &self,
        input: query_use_case::get_chart::Input,
    ) -> Result<query_use_case::get_chart::Output, query_use_case::get_chart::Error> {
        let chart_reader = self.chart_reader();
        let chart_id =
            ChartId::from_str(&input.chart_id).map_err(|_| query_use_case::get_chart::Error)?;
        chart_reader
            .get(chart_id)
            .await
            .map(
                |ChartQueryData {
                     created_at,
                     id,
                     title,
                 }| query_use_case::get_chart::Output {
                    created_at: created_at.to_string(),
                    id: id.to_string(),
                    title,
                },
            )
            .map_err(|_| query_use_case::get_chart::Error)
    }
}

impl query_use_case::get_chart::HasGetChart for InMemoryApp {
    fn get_chart(&self) -> Arc<dyn query_use_case::get_chart::GetChart + Send + Sync> {
        Arc::new(self.clone())
    }
}

impl query_use_case::list_charts::HasListCharts for InMemoryApp {
    fn list_charts(&self) -> Arc<dyn query_use_case::list_charts::ListCharts + Send + Sync> {
        Arc::new(self.clone())
    }
}

#[async_trait::async_trait]
impl query_use_case::list_charts::ListCharts for InMemoryApp {
    async fn execute(
        &self,
        _: query_use_case::list_charts::Input,
    ) -> Result<query_use_case::list_charts::Output, query_use_case::list_charts::Error> {
        let chart_reader = self.chart_reader();
        chart_reader
            .list()
            .await
            .map(|charts| {
                query_use_case::list_charts::Output(
                    charts
                        .into_iter()
                        .map(
                            |ChartQueryData {
                                 created_at,
                                 id,
                                 title,
                             }| query_use_case::list_charts::Chart {
                                created_at: created_at.to_string(),
                                id: id.to_string(),
                                title,
                            },
                        )
                        .collect(),
                )
            })
            .map_err(|_| query_use_case::list_charts::Error)
    }
}
