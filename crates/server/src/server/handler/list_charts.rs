use axum::{extract::State, http::StatusCode, Json, Router};

use query_use_case::{self, list_charts::HasListCharts};

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
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

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use query_use_case::{list_charts::Chart, list_charts::MockListCharts};
    use write_model::value_object::DateTime;

    use crate::server::handler::tests::{send_request, ResponseExt as _};

    use super::*;

    #[tokio::test]
    async fn test_happy_path() -> anyhow::Result<()> {
        let chart = build_chart();
        let mocks = Mocks::with_happy_path_behavior(chart.clone());
        let app = router().with_state(mocks.clone());
        let request = build_request()?;
        let response = send_request(app, request).await?;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.into_body_as_json::<ResponseBody>().await?,
            ResponseBody {
                charts: vec![ResponseBodyChart {
                    created_at: chart.created_at,
                    id: chart.id,
                    title: chart.title
                }]
            }
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_error() -> anyhow::Result<()> {
        let chart = build_chart();
        let mut mocks = Mocks::with_happy_path_behavior(chart.clone());
        mocks.list_charts = {
            let mut mock = MockListCharts::new();
            mock.expect_execute()
                .return_once(|_| Err(query_use_case::list_charts::Error::ChartList(build_error())));
            Arc::new(mock)
        };
        let app = router().with_state(mocks.clone());
        let request = build_request()?;
        let response = send_request(app, request).await?;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(response.into_body_string().await?, "");
        Ok(())
    }

    #[derive(Clone)]
    struct Mocks {
        list_charts: Arc<MockListCharts>,
    }

    impl Mocks {
        fn with_happy_path_behavior(chart: Chart) -> Self {
            let mut list_charts = MockListCharts::new();
            list_charts
                .expect_execute()
                .return_once(move |_| Ok(query_use_case::list_charts::Output(vec![chart.clone()])));
            Self {
                list_charts: Arc::new(list_charts),
            }
        }
    }

    impl query_use_case::list_charts::HasListCharts for Mocks {
        fn list_charts(&self) -> Arc<dyn query_use_case::list_charts::ListCharts + Send + Sync> {
            self.list_charts.clone()
        }
    }

    fn build_chart() -> Chart {
        Chart {
            created_at: DateTime::now().to_string(),
            id: "chart_id1".to_string(),
            title: "title1".to_string(),
        }
    }

    fn build_error() -> Box<dyn std::error::Error + Send + Sync> {
        Box::new(std::io::Error::new(std::io::ErrorKind::Other, "error"))
    }

    fn build_request() -> anyhow::Result<axum::http::Request<axum::body::Body>> {
        Ok(axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri("/charts")
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(axum::body::Body::empty())?)
    }
}
