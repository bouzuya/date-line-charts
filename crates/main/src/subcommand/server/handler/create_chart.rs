use axum::{extract::State, http::StatusCode, Json, Router};

use crate::command_use_case::{self, create_chart::CreateChart, create_chart::HasCreateChart};

#[derive(serde::Deserialize)]
struct RequestBody {
    title: String,
}

impl From<RequestBody> for command_use_case::create_chart::Input {
    fn from(RequestBody { title }: RequestBody) -> Self {
        Self { title }
    }
}

#[derive(serde::Serialize)]
struct ResponseBody {
    chart_id: String,
}

impl From<command_use_case::create_chart::Output> for ResponseBody {
    fn from(
        command_use_case::create_chart::Output { chart_id }: command_use_case::create_chart::Output,
    ) -> Self {
        Self { chart_id }
    }
}

async fn handler<T: HasCreateChart>(
    State(state): State<T>,
    Json(body): Json<RequestBody>,
) -> Result<Json<ResponseBody>, StatusCode> {
    let use_case = state.create_chart();
    let output = use_case
        .execute(command_use_case::create_chart::Input::from(body))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ResponseBody::from(output)))
}

pub fn router<T: Clone + HasCreateChart + Send + Sync + 'static>() -> Router<T> {
    Router::new().route("/charts", axum::routing::post(handler::<T>))
}
