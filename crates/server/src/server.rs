use std::{sync::Arc, time::SystemTime};

use tokio::sync::Mutex;

use crate::{command_use_case, query_use_case};

mod handler;

#[derive(Clone)]
struct AppState {
    data: Arc<Mutex<Vec<Chart>>>,
}

#[axum::async_trait]
impl command_use_case::create_chart::CreateChart for AppState {
    async fn execute(
        &self,
        input: command_use_case::create_chart::Input,
    ) -> Result<command_use_case::create_chart::Output, command_use_case::create_chart::Error> {
        let mut data = self.data.lock().await;
        let id = format!("{}", data.len() + 1);
        data.push(Chart {
            created_at: SystemTime::now(),
            id: id.clone(),
            title: input.title,
        });
        Ok(command_use_case::create_chart::Output { chart_id: id })
    }
}

impl command_use_case::create_chart::HasCreateChart for AppState {
    type CreateChart = Self;
    fn create_chart(&self) -> Self::CreateChart {
        self.clone()
    }
}

#[axum::async_trait]
impl command_use_case::delete_chart::DeleteChart for AppState {
    async fn execute(
        &self,
        input: command_use_case::delete_chart::Input,
    ) -> Result<command_use_case::delete_chart::Output, command_use_case::delete_chart::Error> {
        let mut data = self.data.lock().await;
        let index = data
            .iter()
            .position(|chart| chart.id == input.chart_id)
            .ok_or(command_use_case::delete_chart::Error)?;
        data.remove(index);
        Ok(command_use_case::delete_chart::Output)
    }
}

impl command_use_case::delete_chart::HasDeleteChart for AppState {
    type DeleteChart = Self;
    fn delete_chart(&self) -> Self::DeleteChart {
        self.clone()
    }
}

impl command_use_case::update_chart::HasUpdateChart for AppState {
    type UpdateChart = Self;
    fn update_chart(&self) -> Self::UpdateChart {
        self.clone()
    }
}

#[axum::async_trait]
impl command_use_case::update_chart::UpdateChart for AppState {
    async fn execute(
        &self,
        input: command_use_case::update_chart::Input,
    ) -> Result<command_use_case::update_chart::Output, command_use_case::update_chart::Error> {
        let mut data = self.data.lock().await;
        let chart = data
            .iter_mut()
            .find(|chart| chart.id == input.chart_id)
            .ok_or(command_use_case::update_chart::Error)?;
        chart.title = input.title;
        Ok(command_use_case::update_chart::Output)
    }
}

#[axum::async_trait]
impl query_use_case::get_chart::GetChart for AppState {
    async fn execute(
        &self,
        input: query_use_case::get_chart::Input,
    ) -> Result<query_use_case::get_chart::Output, query_use_case::get_chart::Error> {
        let data = self.data.lock().await;
        let chart = data
            .iter()
            .find(|chart| chart.id == input.chart_id)
            .ok_or(query_use_case::get_chart::Error)?;
        Ok(query_use_case::get_chart::Output {
            // FIXME: This is not a good way to convert SystemTime to String.
            created_at: chart
                .created_at
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("FIXME")
                .as_secs()
                .to_string(),
            id: chart.id.clone(),
            title: chart.title.clone(),
        })
    }
}

impl query_use_case::get_chart::HasGetChart for AppState {
    type GetChart = Self;
    fn get_chart(&self) -> Self::GetChart {
        self.clone()
    }
}

impl query_use_case::list_charts::HasListCharts for AppState {
    type ListCharts = Self;
    fn list_charts(&self) -> Self::ListCharts {
        self.clone()
    }
}

#[axum::async_trait]
impl query_use_case::list_charts::ListCharts for AppState {
    async fn execute(
        &self,
        _: query_use_case::list_charts::Input,
    ) -> Result<query_use_case::list_charts::Output, query_use_case::list_charts::Error> {
        let data = self.data.lock().await;
        Ok(query_use_case::list_charts::Output(
            data.iter()
                .map(|chart| query_use_case::list_charts::Chart {
                    created_at: chart
                        .created_at
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .expect("FIXME")
                        .as_secs()
                        .to_string(),
                    id: chart.id.clone(),
                    title: chart.title.clone(),
                })
                .collect(),
        ))
    }
}

#[derive(Clone)]
struct Chart {
    created_at: SystemTime,
    id: String,
    title: String,
}

pub async fn run() -> anyhow::Result<()> {
    let router = handler::router().with_state(AppState {
        data: Arc::new(Mutex::new(Vec::new())),
    });
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, router).await?;
    Ok(())
}