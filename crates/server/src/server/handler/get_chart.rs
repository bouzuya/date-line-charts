use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
};

use query_use_case::{self, get_chart::HasGetChart};

#[derive(serde::Deserialize)]
struct PathParameters {
    chart_id: String,
}

impl From<PathParameters> for query_use_case::get_chart::Input {
    fn from(PathParameters { chart_id }: PathParameters) -> Self {
        Self { chart_id }
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use query_use_case::{get_chart::MockGetChart, list_charts::Chart};
    use write_model::value_object::DateTime;

    use crate::server::handler::tests::{send_request, ResponseExt as _};

    use super::*;

    #[tokio::test]
    async fn test_happy_path() -> anyhow::Result<()> {
        let chart = build_chart();
        let mocks = Mocks::with_happy_path_behavior(chart.clone());
        let app = router().with_state(mocks.clone());
        let request = build_request(&PathParameters {
            chart_id: chart.id.clone(),
        })?;
        let response = send_request(app, request).await?;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.into_body_as_json::<ResponseBody>().await?,
            ResponseBody {
                created_at: chart.created_at,
                id: chart.id,
                title: chart.title
            }
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_error() -> anyhow::Result<()> {
        let chart = build_chart();
        let mut mocks = Mocks::with_happy_path_behavior(chart.clone());
        mocks.get_chart = {
            let mut mock = MockGetChart::new();
            mock.expect_execute()
                .return_once(|_| Err(query_use_case::get_chart::Error));
            Arc::new(mock)
        };
        let app = router().with_state(mocks.clone());
        let request = build_request(&PathParameters {
            chart_id: chart.id.clone(),
        })?;
        let response = send_request(app, request).await?;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(response.into_body_string().await?, "");
        Ok(())
    }

    #[derive(Clone)]
    struct Mocks {
        get_chart: Arc<MockGetChart>,
    }

    impl Mocks {
        fn with_happy_path_behavior(chart: Chart) -> Self {
            let mut get_chart = MockGetChart::new();
            get_chart.expect_execute().return_once(move |input| {
                assert_eq!(input.chart_id, chart.id);
                Ok(query_use_case::get_chart::Output {
                    created_at: chart.created_at,
                    id: chart.id,
                    title: chart.title,
                })
            });
            Self {
                get_chart: Arc::new(get_chart),
            }
        }
    }

    impl query_use_case::get_chart::HasGetChart for Mocks {
        fn get_chart(&self) -> Arc<dyn query_use_case::get_chart::GetChart + Send + Sync> {
            self.get_chart.clone()
        }
    }

    fn build_chart() -> Chart {
        Chart {
            created_at: DateTime::now().to_string(),
            id: "chart_id1".to_string(),
            title: "title1".to_string(),
        }
    }

    fn build_request(
        path_parameters: &PathParameters,
    ) -> anyhow::Result<axum::http::Request<axum::body::Body>> {
        Ok(axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri(format!("/charts/{}", path_parameters.chart_id))
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(axum::body::Body::empty())?)
    }
}
