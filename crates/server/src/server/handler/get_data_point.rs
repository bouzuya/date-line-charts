use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
};

use query_use_case::{self, get_data_point::HasGetDataPoint};

#[derive(serde::Deserialize)]
struct PathParameters {
    data_point_id: String,
}

impl From<PathParameters> for query_use_case::get_data_point::Input {
    fn from(PathParameters { data_point_id }: PathParameters) -> Self {
        Self { data_point_id }
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
struct ResponseBody {
    chart_id: String,
    created_at: String,
    id: String,
    x_value: String,
    y_value: u32,
}

impl From<query_use_case::get_data_point::Output> for ResponseBody {
    fn from(
        query_use_case::get_data_point::Output {
            chart_id,
            created_at,
            x_value,
            y_value,
        }: query_use_case::get_data_point::Output,
    ) -> Self {
        let id = format!("{}:{}", chart_id, x_value);
        Self {
            chart_id,
            created_at,
            id,
            x_value,
            y_value,
        }
    }
}

async fn handler<T: HasGetDataPoint>(
    State(state): State<T>,
    Path(path_parameters): Path<PathParameters>,
) -> Result<Json<ResponseBody>, StatusCode> {
    let use_case = state.get_data_point();
    let output = use_case
        .execute(query_use_case::get_data_point::Input::from(path_parameters))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ResponseBody::from(output)))
}

pub fn router<T: Clone + HasGetDataPoint + Send + Sync + 'static>() -> Router<T> {
    Router::new().route(
        "/data_points/:data_point_id",
        axum::routing::get(handler::<T>),
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use query_use_case::{get_data_point::MockGetDataPoint, list_data_points::DataPoint};
    use write_model::value_object::DateTime;

    use crate::server::handler::tests::{send_request, ResponseExt as _};

    use super::*;

    #[tokio::test]
    async fn test_happy_path() -> anyhow::Result<()> {
        let data_point = build_data_point();
        let mocks = Mocks::with_happy_path_behavior(data_point.clone());
        let app = router().with_state(mocks.clone());
        let request = build_request(&PathParameters {
            data_point_id: format!("{}:{}", data_point.chart_id, data_point.x_value),
        })?;
        let response = send_request(app, request).await?;
        assert_eq!(response.status(), StatusCode::OK);
        let id = format!("{}:{}", data_point.chart_id, data_point.x_value);
        assert_eq!(
            response.into_body_as_json::<ResponseBody>().await?,
            ResponseBody {
                chart_id: data_point.chart_id,
                created_at: data_point.created_at,
                id,
                x_value: data_point.x_value,
                y_value: data_point.y_value,
            }
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_error() -> anyhow::Result<()> {
        let data_point = build_data_point();
        let mut mocks = Mocks::with_happy_path_behavior(data_point.clone());
        mocks.get_data_point = {
            let mut mock = MockGetDataPoint::new();
            mock.expect_execute().return_once(|_| {
                Err(query_use_case::get_data_point::Error::DataPointGet(
                    build_error(),
                ))
            });
            Arc::new(mock)
        };
        let app = router().with_state(mocks.clone());
        let request = build_request(&PathParameters {
            data_point_id: format!("{}:{}", data_point.chart_id, data_point.x_value),
        })?;
        let response = send_request(app, request).await?;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(response.into_body_string().await?, "");
        Ok(())
    }

    #[derive(Clone)]
    struct Mocks {
        get_data_point: Arc<MockGetDataPoint>,
    }

    impl Mocks {
        fn with_happy_path_behavior(data_point: DataPoint) -> Self {
            let mut get_data_point = MockGetDataPoint::new();
            get_data_point.expect_execute().return_once(move |input| {
                assert_eq!(
                    input.data_point_id,
                    format!("{}:{}", data_point.chart_id, data_point.x_value)
                );
                Ok(query_use_case::get_data_point::Output {
                    chart_id: data_point.chart_id,
                    created_at: data_point.created_at,
                    x_value: data_point.x_value,
                    y_value: data_point.y_value,
                })
            });
            Self {
                get_data_point: Arc::new(get_data_point),
            }
        }
    }

    impl query_use_case::get_data_point::HasGetDataPoint for Mocks {
        fn get_data_point(
            &self,
        ) -> Arc<dyn query_use_case::get_data_point::GetDataPoint + Send + Sync> {
            self.get_data_point.clone()
        }
    }

    fn build_data_point() -> DataPoint {
        DataPoint {
            chart_id: "chart_id1".to_string(),
            created_at: DateTime::now().to_string(),
            x_value: "2020-01-02".to_string(),
            y_value: 123_u32,
        }
    }

    fn build_error() -> Box<dyn std::error::Error + Send + Sync> {
        Box::new(std::io::Error::new(std::io::ErrorKind::Other, "error"))
    }

    fn build_request(
        path_parameters: &PathParameters,
    ) -> anyhow::Result<axum::http::Request<axum::body::Body>> {
        Ok(axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri(format!("/data_points/{}", path_parameters.data_point_id))
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(axum::body::Body::empty())?)
    }
}
