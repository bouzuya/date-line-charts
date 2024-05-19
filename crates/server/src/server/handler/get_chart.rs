use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
};

use query_use_case::{self, get_chart::GetChart, get_chart::HasGetChart};

#[derive(serde::Deserialize)]
struct PathParameters {
    chart_id: String,
}

impl From<PathParameters> for query_use_case::get_chart::Input {
    fn from(PathParameters { chart_id }: PathParameters) -> Self {
        Self { chart_id }
    }
}

#[derive(serde::Serialize)]
struct ResponseBody {
    created_at: String,
    id: String,
    title: String,
}

impl From<query_use_case::get_chart::Output> for ResponseBody {
    fn from(
        query_use_case::get_chart::Output {
            created_at,
            id,
            title,
        }: query_use_case::get_chart::Output,
    ) -> Self {
        Self {
            created_at,
            id,
            title,
        }
    }
}

async fn handler<T: HasGetChart>(
    State(state): State<T>,
    Path(path_parameters): Path<PathParameters>,
) -> Result<Json<ResponseBody>, StatusCode> {
    let use_case = state.get_chart();
    let output = use_case
        .execute(query_use_case::get_chart::Input::from(path_parameters))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ResponseBody::from(output)))
}

pub fn router<T: Clone + HasGetChart + Send + Sync + 'static>() -> Router<T> {
    Router::new().route("/charts/:chart_id", axum::routing::get(handler::<T>))
}
