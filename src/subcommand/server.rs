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
impl query_use_case::show_chart::ShowChart for AppState {
    async fn execute(
        &self,
        input: query_use_case::show_chart::Input,
    ) -> Result<query_use_case::show_chart::Output, query_use_case::show_chart::Error> {
        let data = self.data.lock().await;
        let chart = data
            .iter()
            .find(|chart| chart.id == input.chart_id)
            .ok_or(query_use_case::show_chart::Error)?;
        Ok(query_use_case::show_chart::Output {
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

impl query_use_case::show_chart::HasShowChart for AppState {
    type ShowChart = Self;
    fn show_chart(&self) -> Self::ShowChart {
        self.clone()
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
