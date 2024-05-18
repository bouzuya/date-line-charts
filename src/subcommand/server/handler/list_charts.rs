use axum::{extract::State, http::StatusCode, Json, Router};

use crate::query_use_case::{self, list_charts::HasListCharts, list_charts::ListCharts};

#[derive(serde::Serialize)]
struct ResponseBody {
    charts: Vec<ResponseBodyChart>,
}

impl From<query_use_case::list_charts::Output> for ResponseBody {
    fn from(
        query_use_case::list_charts::Output(charts): query_use_case::list_charts::Output,
    ) -> Self {
        Self {
            charts: charts.into_iter().map(ResponseBodyChart::from).collect(),
        }
    }
}

#[derive(serde::Serialize)]
struct ResponseBodyChart {
    created_at: String,
    id: String,
    title: String,
}

impl From<query_use_case::list_charts::Chart> for ResponseBodyChart {
    fn from(
        query_use_case::list_charts::Chart {
            created_at,
            id,
            title,
        }: query_use_case::list_charts::Chart,
    ) -> Self {
        Self {
            created_at,
            id,
            title,
        }
    }
}

async fn handler<T: HasListCharts>(
    State(state): State<T>,
) -> Result<Json<ResponseBody>, StatusCode> {
    let use_case = state.list_charts();
    let output = use_case
        .execute(query_use_case::list_charts::Input)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ResponseBody::from(output)))
}

pub fn router<T: Clone + HasListCharts + Send + Sync + 'static>() -> Router<T> {
    Router::new().route("/charts", axum::routing::get(handler::<T>))
}
