use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
};

use command_use_case::{
    self,
    delete_chart::{DeleteChart, HasDeleteChart},
};

#[derive(serde::Deserialize)]
struct PathParameters {
    chart_id: String,
}

impl From<PathParameters> for command_use_case::delete_chart::Input {
    fn from(PathParameters { chart_id }: PathParameters) -> Self {
        Self { chart_id }
    }
}

#[derive(serde::Serialize)]
struct ResponseBody {}

async fn handler<T: HasDeleteChart>(
    State(state): State<T>,
    Path(path_parameters): Path<PathParameters>,
) -> Result<Json<ResponseBody>, StatusCode> {
    let use_case = state.delete_chart();
    let _output = use_case
        .execute(command_use_case::delete_chart::Input::from(path_parameters))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ResponseBody {}))
}

pub fn router<T: Clone + HasDeleteChart + Send + Sync + 'static>() -> Router<T> {
    Router::new().route("/charts/:chart_id", axum::routing::delete(handler::<T>))
}
