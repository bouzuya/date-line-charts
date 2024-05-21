use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
};

use command_use_case::{self, update_chart::HasUpdateChart};

#[derive(serde::Deserialize)]
struct PathParameters {
    chart_id: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct RequestBody {
    title: String,
}

fn input_from_request(
    PathParameters { chart_id }: PathParameters,
    RequestBody { title }: RequestBody,
) -> command_use_case::update_chart::Input {
    command_use_case::update_chart::Input { chart_id, title }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use command_use_case::update_chart::MockUpdateChart;

    use crate::server::handler::tests::{send_request, ResponseExt as _};

    use super::*;

    #[tokio::test]
    async fn test_happy_path() -> anyhow::Result<()> {
        let title = "title1".to_string();
        let chart_id = "chart_id1".to_string();
        let mocks = Mocks::with_happy_path_behavior(title.clone(), chart_id.clone());
        let app = router().with_state(mocks.clone());
        let request = build_request(&PathParameters { chart_id }, &RequestBody { title })?;
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
        let title = "title1".to_string();
        let chart_id = "chart_id1".to_string();
        let mut mocks = Mocks::with_happy_path_behavior(title.clone(), chart_id.clone());
        mocks.update_chart = {
            let mut mock = MockUpdateChart::new();
            mock.expect_execute().return_once(|_| {
                Err(command_use_case::update_chart::Error::ChartStore(
                    build_error(),
                ))
            });
            Arc::new(mock)
        };
        let app = router().with_state(mocks.clone());
        let request = build_request(&PathParameters { chart_id }, &RequestBody { title })?;
        let response = send_request(app, request).await?;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(response.into_body_string().await?, "");
        Ok(())
    }

    #[derive(Clone)]
    struct Mocks {
        update_chart: Arc<MockUpdateChart>,
    }

    impl Mocks {
        fn with_happy_path_behavior(title: String, chart_id: String) -> Self {
            let mut update_chart = MockUpdateChart::new();
            update_chart.expect_execute().return_once(move |input| {
                assert_eq!(input.chart_id, chart_id);
                assert_eq!(input.title, title);
                Ok(command_use_case::update_chart::Output {})
            });
            Self {
                update_chart: Arc::new(update_chart),
            }
        }
    }

    impl command_use_case::update_chart::HasUpdateChart for Mocks {
        fn update_chart(
            &self,
        ) -> Arc<dyn command_use_case::update_chart::UpdateChart + Send + Sync> {
            self.update_chart.clone()
        }
    }

    fn build_error() -> Box<dyn std::error::Error + Send + Sync> {
        Box::new(std::io::Error::new(std::io::ErrorKind::Other, "error"))
    }

    fn build_request<T: serde::Serialize>(
        path_parameters: &PathParameters,
        request_body: &T,
    ) -> anyhow::Result<axum::http::Request<axum::body::Body>> {
        Ok(axum::http::Request::builder()
            .method(axum::http::Method::PATCH)
            .uri(format!("/charts/{}", path_parameters.chart_id))
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(axum::body::Body::from(serde_json::to_string(request_body)?))?)
    }
}
