use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
};

use command_use_case::{self, update_data_point::HasUpdateDataPoint};

#[derive(serde::Deserialize)]
struct PathParameters {
    data_point_id: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct RequestBody {
    y_value: u32,
}

fn input_from_request(
    PathParameters { data_point_id }: PathParameters,
    RequestBody { y_value }: RequestBody,
) -> command_use_case::update_data_point::Input {
    command_use_case::update_data_point::Input {
        data_point_id,
        y_value,
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
struct ResponseBody {}

async fn handler<T: HasUpdateDataPoint>(
    State(state): State<T>,
    Path(path_parameters): Path<PathParameters>,
    Json(body): Json<RequestBody>,
) -> Result<Json<ResponseBody>, StatusCode> {
    let use_case = state.update_data_point();
    let _output = use_case
        .execute(input_from_request(path_parameters, body))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ResponseBody {}))
}

pub fn router<T: Clone + HasUpdateDataPoint + Send + Sync + 'static>() -> Router<T> {
    Router::new().route(
        "/data_points/:data_point_id",
        axum::routing::patch(handler::<T>),
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use command_use_case::update_data_point::MockUpdateDataPoint;

    use crate::server::handler::tests::{send_request, ResponseExt as _};

    use super::*;

    #[tokio::test]
    async fn test_happy_path() -> anyhow::Result<()> {
        let data_point_id = "data_point_id1".to_string();
        let y_value = 123_u32;
        let mocks = Mocks::with_happy_path_behavior(data_point_id.clone(), y_value);
        let app = router().with_state(mocks.clone());
        let request = build_request(&PathParameters { data_point_id }, &RequestBody { y_value })?;
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
        let y_value = 123_u32;
        let data_point_id = "data_point_id1".to_string();
        let mut mocks = Mocks::with_happy_path_behavior(data_point_id.clone(), y_value);
        mocks.update_data_point = {
            let mut mock = MockUpdateDataPoint::new();
            mock.expect_execute().return_once(|_| {
                Err(command_use_case::update_data_point::Error::DataPointStore(
                    command_use_case::port::data_point_repository::Error::from(build_error()),
                ))
            });
            Arc::new(mock)
        };
        let app = router().with_state(mocks.clone());
        let request = build_request(&PathParameters { data_point_id }, &RequestBody { y_value })?;
        let response = send_request(app, request).await?;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(response.into_body_string().await?, "");
        Ok(())
    }

    #[derive(Clone)]
    struct Mocks {
        update_data_point: Arc<MockUpdateDataPoint>,
    }

    impl Mocks {
        fn with_happy_path_behavior(data_point_id: String, y_value: u32) -> Self {
            let mut update_data_point = MockUpdateDataPoint::new();
            update_data_point
                .expect_execute()
                .return_once(move |input| {
                    assert_eq!(input.data_point_id, data_point_id);
                    assert_eq!(input.y_value, y_value);
                    Ok(command_use_case::update_data_point::Output)
                });
            Self {
                update_data_point: Arc::new(update_data_point),
            }
        }
    }

    impl command_use_case::update_data_point::HasUpdateDataPoint for Mocks {
        fn update_data_point(
            &self,
        ) -> Arc<dyn command_use_case::update_data_point::UpdateDataPoint + Send + Sync> {
            self.update_data_point.clone()
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
            .uri(format!("/data_points/{}", path_parameters.data_point_id))
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(axum::body::Body::from(serde_json::to_string(request_body)?))?)
    }
}
