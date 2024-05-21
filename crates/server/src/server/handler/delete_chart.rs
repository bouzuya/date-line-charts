use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
};

use command_use_case::{self, delete_chart::HasDeleteChart};

#[derive(serde::Deserialize)]
struct PathParameters {
    chart_id: String,
}

impl From<PathParameters> for command_use_case::delete_chart::Input {
    fn from(PathParameters { chart_id }: PathParameters) -> Self {
        Self { chart_id }
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use command_use_case::delete_chart::MockDeleteChart;

    use crate::server::handler::tests::{send_request, ResponseExt as _};

    use super::*;

    #[tokio::test]
    async fn test_happy_path() -> anyhow::Result<()> {
        let chart_id = "chart_id1".to_string();
        let mocks = Mocks::with_happy_path_behavior(chart_id.clone());
        let app = router().with_state(mocks.clone());
        let request = build_request(&PathParameters { chart_id })?;
        let response = send_request(app, request).await?;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.into_body_as_json::<ResponseBody>().await?,
            ResponseBody {}
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_error() -> anyhow::Result<()> {
        let chart_id = "chart_id1".to_string();
        let mut mocks = Mocks::with_happy_path_behavior(chart_id.clone());
        mocks.delete_chart = {
            let mut mock = MockDeleteChart::new();
            mock.expect_execute().return_once(|_| {
                Err(command_use_case::delete_chart::Error::ChartStore(
                    build_error(),
                ))
            });
            Arc::new(mock)
        };
        let app = router().with_state(mocks.clone());
        let request = build_request(&PathParameters { chart_id })?;
        let response = send_request(app, request).await?;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(response.into_body_string().await?, "");
        Ok(())
    }

    #[derive(Clone)]
    struct Mocks {
        delete_chart: Arc<MockDeleteChart>,
    }

    impl Mocks {
        fn with_happy_path_behavior(chart_id: String) -> Self {
            let mut delete_chart = MockDeleteChart::new();
            delete_chart.expect_execute().return_once(move |input| {
                assert_eq!(input.chart_id, chart_id);
                Ok(command_use_case::delete_chart::Output)
            });
            Self {
                delete_chart: Arc::new(delete_chart),
            }
        }
    }

    impl command_use_case::delete_chart::HasDeleteChart for Mocks {
        fn delete_chart(
            &self,
        ) -> Arc<dyn command_use_case::delete_chart::DeleteChart + Send + Sync> {
            self.delete_chart.clone()
        }
    }

    fn build_error() -> Box<dyn std::error::Error + Send + Sync> {
        Box::new(std::io::Error::new(std::io::ErrorKind::Other, "error"))
    }

    fn build_request(
        path_parameters: &PathParameters,
    ) -> anyhow::Result<axum::http::Request<axum::body::Body>> {
        Ok(axum::http::Request::builder()
            .method(axum::http::Method::DELETE)
            .uri(format!("/charts/{}", path_parameters.chart_id))
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(axum::body::Body::empty())?)
    }
}
