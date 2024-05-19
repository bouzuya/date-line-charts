use std::{collections::BTreeMap, str::FromStr as _, sync::Arc};

use command_use_case::create_chart::{ChartRepository, HasChartRepository as _};
use domain::{
    aggregate::{chart::Event, Chart},
    value_object::{ChartId, DateTime, Version},
};
use tokio::sync::Mutex;

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

    async fn get_impl(
        &self,
        input: query_use_case::get_chart::Input,
    ) -> Result<query_use_case::get_chart::Output, query_use_case::get_chart::Error> {
        let query_data = self.query_data.lock().await;
        let chart = query_data
            .iter()
            .find(|chart| chart.id == input.chart_id)
            .ok_or(query_use_case::get_chart::Error)?;
        Ok(query_use_case::get_chart::Output {
            created_at: chart.created_at.to_string(),
            id: chart.id.clone(),
            title: chart.title.clone(),
        })
    }

    async fn list_impl(
        &self,
    ) -> Result<query_use_case::list_charts::Output, query_use_case::list_charts::Error> {
        let query_data = self.query_data.lock().await;
        Ok(query_use_case::list_charts::Output(
            query_data
                .iter()
                .map(|chart| query_use_case::list_charts::Chart {
                    created_at: chart.created_at.to_string(),
                    id: chart.id.clone(),
                    title: chart.title.clone(),
                })
                .collect(),
        ))
    }
}

#[axum::async_trait]
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
                let id = match &events[0] {
                    Event::Created(event) => event.id,
                    Event::Deleted(event) => event.id,
                    Event::Updated(event) => event.id,
                };
                command_data.insert(id, events.to_vec());

                let state = Chart::from_events(events)?;
                query_data.push(ChartQueryData {
                    created_at: state.created_at(),
                    id: state.id().to_string(),
                    title: state.title().to_string(),
                });
            }
            Some(_version) => {
                let id = match &events[0] {
                    Event::Created(event) => event.id,
                    Event::Deleted(event) => event.id,
                    Event::Updated(event) => event.id,
                };
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
pub struct AppState {
    chart_database: Arc<ChartDatabase>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            chart_database: Arc::new(ChartDatabase::new()),
        }
    }
}

impl command_use_case::create_chart::HasChartRepository for AppState {
    fn chart_repository(&self) -> Arc<dyn ChartRepository + Send + Sync> {
        self.chart_database.clone()
    }
}

#[axum::async_trait]
impl command_use_case::create_chart::CreateChart for AppState {
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

impl command_use_case::create_chart::HasCreateChart for AppState {
    fn create_chart(&self) -> Arc<dyn command_use_case::create_chart::CreateChart + Send + Sync> {
        Arc::new(self.clone())
    }
}

#[axum::async_trait]
impl command_use_case::delete_chart::DeleteChart for AppState {
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

impl command_use_case::delete_chart::HasDeleteChart for AppState {
    fn delete_chart(&self) -> Arc<dyn command_use_case::delete_chart::DeleteChart + Send + Sync> {
        Arc::new(self.clone())
    }
}

impl command_use_case::update_chart::HasUpdateChart for AppState {
    fn update_chart(&self) -> Arc<dyn command_use_case::update_chart::UpdateChart + Send + Sync> {
        Arc::new(self.clone())
    }
}

#[axum::async_trait]
impl command_use_case::update_chart::UpdateChart for AppState {
    async fn execute(
        &self,
        input: command_use_case::update_chart::Input,
    ) -> Result<command_use_case::update_chart::Output, command_use_case::update_chart::Error> {
        let chart_id = ChartId::from_str(&input.chart_id)
            .map_err(|_| command_use_case::update_chart::Error)?;
        let chart = self
            .chart_repository()
            .find(chart_id)
            .await
            .map_err(|_| command_use_case::update_chart::Error)?
            .ok_or(command_use_case::update_chart::Error)?;
        let (_, events) = chart
            .update(input.title)
            .map_err(|_| command_use_case::update_chart::Error)?;
        self.chart_repository()
            .store(Some(chart.version()), &events)
            .await
            .map_err(|_| command_use_case::update_chart::Error)?;
        Ok(command_use_case::update_chart::Output)
    }
}

#[axum::async_trait]
impl query_use_case::get_chart::GetChart for AppState {
    async fn execute(
        &self,
        input: query_use_case::get_chart::Input,
    ) -> Result<query_use_case::get_chart::Output, query_use_case::get_chart::Error> {
        self.chart_database.get_impl(input).await
    }
}

impl query_use_case::get_chart::HasGetChart for AppState {
    fn get_chart(&self) -> Arc<dyn query_use_case::get_chart::GetChart + Send + Sync> {
        Arc::new(self.clone())
    }
}

impl query_use_case::list_charts::HasListCharts for AppState {
    fn list_charts(&self) -> Arc<dyn query_use_case::list_charts::ListCharts + Send + Sync> {
        Arc::new(self.clone())
    }
}

#[axum::async_trait]
impl query_use_case::list_charts::ListCharts for AppState {
    async fn execute(
        &self,
        _: query_use_case::list_charts::Input,
    ) -> Result<query_use_case::list_charts::Output, query_use_case::list_charts::Error> {
        self.chart_database.list_impl().await
    }
}

#[derive(Clone)]
struct ChartQueryData {
    created_at: DateTime,
    id: String,
    title: String,
}
