use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
};

use crate::command_use_case::{self, update_chart::HasUpdateChart, update_chart::UpdateChart};

#[derive(serde::Deserialize)]
struct PathParameters {
    chart_id: String,
}

#[derive(serde::Deserialize)]
struct RequestBody {
    title: String,
}

fn input_from_request(
    PathParameters { chart_id }: PathParameters,
    RequestBody { title }: RequestBody,
) -> command_use_case::update_chart::Input {
    command_use_case::update_chart::Input { chart_id, title }
}

#[derive(serde::Serialize)]
struct ResponseBody {}

async fn handler<T: HasUpdateChart>(
    State(state): State<T>,
    Path(path_parameters): Path<PathParameters>,
    Json(body): Json<RequestBody>,
) -> Result<Json<ResponseBody>, StatusCode> {
    let use_case = state.update_chart();
    let _output = use_case
        .execute(input_from_request(path_parameters, body))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ResponseBody {}))
}

pub fn router<T: Clone + HasUpdateChart + Send + Sync + 'static>() -> Router<T> {
    Router::new().route("/charts/:chart_id", axum::routing::patch(handler::<T>))
}
