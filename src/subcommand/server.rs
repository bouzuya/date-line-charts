use std::{sync::Arc, time::SystemTime};

use tokio::sync::Mutex;

mod handler;

#[derive(Clone)]
struct AppState {
    data: Arc<Mutex<Vec<Chart>>>,
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
