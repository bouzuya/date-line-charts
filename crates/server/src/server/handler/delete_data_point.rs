use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
};

use command_use_case::{self, delete_data_point::HasDeleteDataPoint};

#[derive(serde::Deserialize)]
struct PathParameters {
    data_point_id: String,
}

impl From<PathParameters> for command_use_case::delete_data_point::Input {
    fn from(PathParameters { data_point_id }: PathParameters) -> Self {
        Self { data_point_id }
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
struct ResponseBody {}

async fn handler<T: HasDeleteDataPoint>(
    State(state): State<T>,
    Path(path_parameters): Path<PathParameters>,
) -> Result<Json<ResponseBody>, StatusCode> {
    let use_case = state.delete_data_point();
    let _output = use_case
        .execute(command_use_case::delete_data_point::Input::from(
            path_parameters,
        ))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ResponseBody {}))
}

pub fn router<T: Clone + HasDeleteDataPoint + Send + Sync + 'static>() -> Router<T> {
    Router::new().route(
        "/data_points/:data_point_id",
        axum::routing::delete(handler::<T>),
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use command_use_case::delete_data_point::MockDeleteDataPoint;

    use crate::server::handler::tests::{send_request, ResponseExt as _};

    use super::*;

    #[tokio::test]
    async fn test_happy_path() -> anyhow::Result<()> {
        let data_point_id = "data_point_id1".to_string();
        let mocks = Mocks::with_happy_path_behavior(data_point_id.clone());
        let app = router().with_state(mocks.clone());
        let request = build_request(&PathParameters { data_point_id })?;
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
        let data_point_id = "data_point_id1".to_string();
        let mut mocks = Mocks::with_happy_path_behavior(data_point_id.clone());
        mocks.delete_data_point = {
            let mut mock = MockDeleteDataPoint::new();
            mock.expect_execute().return_once(|_| {
                Err(command_use_case::delete_data_point::Error::DataPointStore(
                    command_use_case::port::data_point_repository::Error::from(build_error()),
                ))
            });
            Arc::new(mock)
        };
        let app = router().with_state(mocks.clone());
        let request = build_request(&PathParameters { data_point_id })?;
        let response = send_request(app, request).await?;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(response.into_body_string().await?, "");
        Ok(())
    }

    #[derive(Clone)]
    struct Mocks {
        delete_data_point: Arc<MockDeleteDataPoint>,
    }

    impl Mocks {
        fn with_happy_path_behavior(data_point_id: String) -> Self {
            let mut delete_data_point = MockDeleteDataPoint::new();
            delete_data_point
                .expect_execute()
                .return_once(move |input| {
                    assert_eq!(input.data_point_id, data_point_id);
                    Ok(command_use_case::delete_data_point::Output)
                });
            Self {
                delete_data_point: Arc::new(delete_data_point),
            }
        }
    }

    impl command_use_case::delete_data_point::HasDeleteDataPoint for Mocks {
        fn delete_data_point(
            &self,
        ) -> Arc<dyn command_use_case::delete_data_point::DeleteDataPoint + Send + Sync> {
            self.delete_data_point.clone()
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
            .uri(format!("/data_points/{}", path_parameters.data_point_id))
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(axum::body::Body::empty())?)
    }
}
