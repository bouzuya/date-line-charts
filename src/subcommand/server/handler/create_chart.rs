use std::time::SystemTime;

use axum::{extract::State, Json, Router};

use crate::subcommand::server::{AppState, Chart};

#[derive(serde::Deserialize)]
struct RequestBody {
    title: String,
}

async fn handler(State(state): State<AppState>, Json(body): Json<RequestBody>) -> String {
    let mut data = state.data.lock().await;
    let id = format!("{}", data.len() + 1);
    data.push(Chart {
        created_at: SystemTime::now(),
        id: id.clone(),
        title: body.title,
    });
    id
}

pub fn router() -> Router<AppState> {
    Router::new().route("/charts", axum::routing::post(handler))
}
