use axum::{extract::State, http::StatusCode, Json, Router};

use command_use_case::{self, create_chart::HasCreateChart};

#[derive(serde::Deserialize, serde::Serialize)]
struct RequestBody {
    title: String,
}

impl From<RequestBody> for command_use_case::create_chart::Input {
    fn from(RequestBody { title }: RequestBody) -> Self {
        Self { title }
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use command_use_case::create_chart::MockCreateChart;

    use crate::server::handler::tests::{send_request, ResponseExt as _};

    use super::*;

    #[tokio::test]
    async fn test_happy_path() -> anyhow::Result<()> {
        let title = "title1".to_string();
        let chart_id = "chart_id1".to_string();
        let mocks = Mocks::with_happy_path_behavior(title.clone(), chart_id.clone());
        let app = router().with_state(mocks.clone());
        let request = build_request(&RequestBody { title })?;
        let response = send_request(app, request).await?;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.into_body_as_json::<ResponseBody>().await?,
            ResponseBody { chart_id }
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_error() -> anyhow::Result<()> {
        let title = "title1".to_string();
        let chart_id = "chart_id1".to_string();
        let mut mocks = Mocks::with_happy_path_behavior(title.clone(), chart_id.clone());
        mocks.create_chart = {
            let mut mock = MockCreateChart::new();
            mock.expect_execute().return_once(|_| {
                Err(command_use_case::create_chart::Error::ChartStore(
                    command_use_case::port::chart_repository::Error::from(build_error()),
                ))
            });
            Arc::new(mock)
        };
        let app = router().with_state(mocks.clone());
        let request = build_request(&RequestBody { title })?;
        let response = send_request(app, request).await?;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(response.into_body_string().await?, "");
        Ok(())
    }

    #[derive(Clone)]
    struct Mocks {
        create_chart: Arc<MockCreateChart>,
    }

    impl Mocks {
        fn with_happy_path_behavior(title: String, chart_id: String) -> Self {
            let mut create_chart = MockCreateChart::new();
            create_chart.expect_execute().return_once(move |input| {
                assert_eq!(input.title, title);
                Ok(command_use_case::create_chart::Output { chart_id })
            });
            Self {
                create_chart: Arc::new(create_chart),
            }
        }
    }

    impl command_use_case::create_chart::HasCreateChart for Mocks {
        fn create_chart(
            &self,
        ) -> Arc<dyn command_use_case::create_chart::CreateChart + Send + Sync> {
            self.create_chart.clone()
        }
    }

    fn build_error() -> Box<dyn std::error::Error + Send + Sync> {
        Box::new(std::io::Error::new(std::io::ErrorKind::Other, "error"))
    }

    fn build_request<T: serde::Serialize>(
        request_body: &T,
    ) -> anyhow::Result<axum::http::Request<axum::body::Body>> {
        Ok(axum::http::Request::builder()
            .method(axum::http::Method::POST)
            .uri("/charts")
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(axum::body::Body::from(serde_json::to_string(request_body)?))?)
    }
}
