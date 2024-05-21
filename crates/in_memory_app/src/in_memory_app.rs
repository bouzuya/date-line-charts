use std::{collections::BTreeMap, str::FromStr as _, sync::Arc};

use command_use_case::port::{ChartRepository, HasChartRepository as _};
use query_use_case::port::{ChartQueryData, HasChartReader as _};
use tokio::sync::Mutex;
use write_model::{
    aggregate::{chart::Event, Chart},
    value_object::{ChartId, Version},
};

struct ChartDatabase {
    command_data: Arc<Mutex<BTreeMap<ChartId, Vec<Event>>>>,
    query_data: Arc<Mutex<Vec<ChartQueryData>>>,
}

impl ChartDatabase {
    fn new() -> Self {
        Self {
            command_data: Arc::new(Mutex::new(BTreeMap::new())),
            query_data: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl query_use_case::port::ChartReader for ChartDatabase {
    async fn get(
        &self,
        id: ChartId,
    ) -> Result<ChartQueryData, Box<dyn std::error::Error + Send + Sync>> {
        let query_data = self.query_data.lock().await;
        Ok(query_data
            .iter()
            .find(|chart| chart.id == id.to_string())
            .cloned()
            .ok_or(query_use_case::get_chart::Error)?)
    }

    async fn list(&self) -> Result<Vec<ChartQueryData>, Box<dyn std::error::Error + Send + Sync>> {
        let query_data = self.query_data.lock().await;
        Ok(query_data.iter().cloned().collect())
    }
}

#[async_trait::async_trait]
impl ChartRepository for ChartDatabase {
    async fn find(
        &self,
        id: ChartId,
    ) -> Result<Option<Chart>, Box<dyn std::error::Error + Send + Sync>> {
        let command_data = self.command_data.lock().await;
        Ok(match command_data.get(&id) {
            None => None,
            Some(events) => Some(Chart::from_events(events)?),
        })
    }

    async fn store(
        &self,
        current: Option<Version>,
        events: &[Event],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut command_data = self.command_data.lock().await;
        let mut query_data = self.query_data.lock().await;
        if events.is_empty() {
            return Ok(());
        }
        match current {
            None => {
                let id = events[0].id;
                command_data.insert(id, events.to_vec());

                let state = Chart::from_events(events)?;
                query_data.push(ChartQueryData {
                    created_at: state.created_at(),
                    id: state.id().to_string(),
                    title: state.title().to_string(),
                });
            }
            Some(_version) => {
                let id = events[0].id;
                let stored_events = command_data.get_mut(&id).ok_or("not found")?;
                // TODO: check version
                stored_events.extend(events.to_vec());

                let state = Chart::from_events(stored_events)?;
                let index = query_data
                    .iter()
                    .position(|chart| chart.id == state.id().to_string())
                    .ok_or("not found")?;
                if state.deleted_at().is_some() {
                    query_data.remove(index);
                } else {
                    query_data[index] = ChartQueryData {
                        created_at: state.created_at(),
                        id: state.id().to_string(),
                        title: state.title().to_string(),
                    };
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct InMemoryApp {
    chart_database: Arc<ChartDatabase>,
}

impl InMemoryApp {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            chart_database: Arc::new(ChartDatabase::new()),
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
        self.chart_database.clone()
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
        self.chart_database.clone()
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
                    id,
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
                                id,
                                title,
                            },
                        )
                        .collect(),
                )
            })
            .map_err(|_| query_use_case::list_charts::Error)
    }
}
