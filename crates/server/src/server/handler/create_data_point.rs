use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
};

use command_use_case::{self, create_data_point::HasCreateDataPoint};

#[derive(serde::Deserialize)]
struct PathParameters {
    chart_id: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct RequestBody {
    x_value: String,
    y_value: u32,
}

trait InputExt {
    fn from_request(
        path_parameters: PathParameters,
        request_body: RequestBody,
    ) -> command_use_case::create_data_point::Input;
}

impl InputExt for command_use_case::create_data_point::Input {
    fn from_request(
        PathParameters { chart_id }: PathParameters,
        RequestBody { x_value, y_value }: RequestBody,
    ) -> command_use_case::create_data_point::Input {
        command_use_case::create_data_point::Input {
            chart_id,
            x_value,
            y_value,
        }
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct ResponseBody {
    data_point_id: String,
}

impl From<command_use_case::create_data_point::Output> for ResponseBody {
    fn from(
        command_use_case::create_data_point::Output { data_point_id }: command_use_case::create_data_point::Output,
    ) -> Self {
        Self { data_point_id }
    }
}

async fn handler<T: HasCreateDataPoint>(
    State(state): State<T>,
    Path(path_parameters): Path<PathParameters>,
    Json(body): Json<RequestBody>,
) -> Result<Json<ResponseBody>, StatusCode> {
    let use_case = state.create_data_point();
    let output = use_case
        .execute(command_use_case::create_data_point::Input::from_request(
            path_parameters,
            body,
        ))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ResponseBody::from(output)))
}

pub fn router<T: Clone + HasCreateDataPoint + Send + Sync + 'static>() -> Router<T> {
    Router::new().route(
        "/charts/:chart_id/data_points",
        axum::routing::post(handler::<T>),
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use command_use_case::create_data_point::MockCreateDataPoint;
    use write_model::value_object::ChartId;

    use crate::server::handler::tests::{send_request, ResponseExt as _};

    use super::*;

    #[tokio::test]
    async fn test_happy_path() -> anyhow::Result<()> {
        let chart_id = ChartId::generate();
        let mocks = Mocks::with_happy_path_behavior();
        let app = router().with_state(mocks.clone());
        let request = build_request(
            &PathParameters {
                chart_id: chart_id.to_string(),
            },
            &RequestBody {
                x_value: "2020-01-02".to_string(),
                y_value: 34,
            },
        )?;
        let response = send_request(app, request).await?;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.into_body_as_json::<ResponseBody>().await?,
            ResponseBody {
                data_point_id: format!("{}:2020-01-02", chart_id)
            }
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_error() -> anyhow::Result<()> {
        let chart_id = ChartId::generate();
        let mut mocks = Mocks::with_happy_path_behavior();
        mocks.create_data_point = {
            let mut mock = MockCreateDataPoint::new();
            mock.expect_execute().return_once(|_| {
                Err(command_use_case::create_data_point::Error::DataPointStore(
                    command_use_case::port::data_point_repository::Error::from(build_error()),
                ))
            });
            Arc::new(mock)
        };
        let app = router().with_state(mocks.clone());
        let request = build_request(
            &PathParameters {
                chart_id: chart_id.to_string(),
            },
            &RequestBody {
                x_value: "2020-01-02".to_string(),
                y_value: 34,
            },
        )?;
        let response = send_request(app, request).await?;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(response.into_body_string().await?, "");
        Ok(())
    }

    #[derive(Clone)]
    struct Mocks {
        create_data_point: Arc<MockCreateDataPoint>,
    }

    impl Mocks {
        fn with_happy_path_behavior() -> Self {
            let mut create_data_point = MockCreateDataPoint::new();
            create_data_point
                .expect_execute()
                .return_once(move |input| {
                    Ok(command_use_case::create_data_point::Output {
                        data_point_id: format!("{}:{}", input.chart_id, input.x_value),
                    })
                });
            Self {
                create_data_point: Arc::new(create_data_point),
            }
        }
    }

    impl command_use_case::create_data_point::HasCreateDataPoint for Mocks {
        fn create_data_point(
            &self,
        ) -> Arc<dyn command_use_case::create_data_point::CreateDataPoint + Send + Sync> {
            self.create_data_point.clone()
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
            .method(axum::http::Method::POST)
            .uri(format!("/charts/{}/data_points", path_parameters.chart_id))
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(axum::body::Body::from(serde_json::to_string(request_body)?))?)
    }
}
