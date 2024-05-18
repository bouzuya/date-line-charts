use std::{sync::Arc, time::SystemTime};

use tokio::sync::Mutex;

use crate::command_use_case;

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
        Ok(command_use_case::create_chart::Output { id })
    }
}

impl command_use_case::create_chart::HasCreateChart for AppState {
    type CreateChart = Self;
    fn create_chart(&self) -> Self::CreateChart {
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
