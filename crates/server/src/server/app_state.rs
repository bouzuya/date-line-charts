use std::{collections::BTreeMap, str::FromStr as _, sync::Arc};

use domain::{
    aggregate::{chart::Event, Chart},
    value_object::{ChartId, DateTime},
};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    command_data: Arc<Mutex<BTreeMap<ChartId, Vec<Event>>>>,
    query_data: Arc<Mutex<Vec<ChartQueryData>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            command_data: Arc::new(Mutex::new(BTreeMap::new())),
            query_data: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[axum::async_trait]
impl command_use_case::create_chart::CreateChart for AppState {
    async fn execute(
        &self,
        input: command_use_case::create_chart::Input,
    ) -> Result<command_use_case::create_chart::Output, command_use_case::create_chart::Error> {
        let mut command_data = self.command_data.lock().await;
        let mut query_data = self.query_data.lock().await;
        let (state, events) =
            Chart::create(input.title).map_err(|_| command_use_case::create_chart::Error)?;
        command_data.insert(state.id(), events);
        query_data.push(ChartQueryData {
            created_at: state.created_at(),
            id: state.id().to_string(),
            title: state.title().to_string(),
        });
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
        let mut command_data = self.command_data.lock().await;
        let mut query_data = self.query_data.lock().await;
        let stored_events = command_data
            .get_mut(
                &ChartId::from_str(&input.chart_id)
                    .map_err(|_| command_use_case::delete_chart::Error)?,
            )
            .ok_or(command_use_case::delete_chart::Error)?;
        let chart =
            Chart::from_events(stored_events).map_err(|_| command_use_case::delete_chart::Error)?;
        let (_, new_events) = chart
            .delete()
            .map_err(|_| command_use_case::delete_chart::Error)?;
        stored_events.extend(new_events);
        let index = query_data
            .iter()
            .position(|chart| chart.id == input.chart_id)
            .ok_or(command_use_case::delete_chart::Error)?;
        query_data.remove(index);
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
        let mut command_data = self.command_data.lock().await;
        let mut query_data = self.query_data.lock().await;
        let stored_events = command_data
            .get_mut(
                &ChartId::from_str(&input.chart_id)
                    .map_err(|_| command_use_case::update_chart::Error)?,
            )
            .ok_or(command_use_case::update_chart::Error)?;
        let chart =
            Chart::from_events(stored_events).map_err(|_| command_use_case::update_chart::Error)?;
        let (state, new_events) = chart
            .update(input.title)
            .map_err(|_| command_use_case::update_chart::Error)?;
        stored_events.extend(new_events);
        let chart = query_data
            .iter_mut()
            .find(|chart| chart.id == input.chart_id)
            .ok_or(command_use_case::update_chart::Error)?;
        chart.title = state.title().to_string();
        Ok(command_use_case::update_chart::Output)
    }
}

#[axum::async_trait]
impl query_use_case::get_chart::GetChart for AppState {
    async fn execute(
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

#[derive(Clone)]
struct ChartQueryData {
    created_at: DateTime,
    id: String,
    title: String,
}
