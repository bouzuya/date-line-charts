use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json, Router,
};

use query_use_case::{self, list_data_points::HasListDataPoints};

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
struct PathParameters {
    chart_id: String,
}

impl From<PathParameters> for query_use_case::list_data_points::Input {
    fn from(PathParameters { chart_id }: PathParameters) -> Self {
        Self { chart_id }
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
struct ResponseBody {
    data_points: Vec<ResponseBodyDataPoint>,
}

impl From<query_use_case::list_data_points::Output> for ResponseBody {
    fn from(
        query_use_case::list_data_points::Output(data_points): query_use_case::list_data_points::Output,
    ) -> Self {
        Self {
            data_points: data_points
                .into_iter()
                .map(ResponseBodyDataPoint::from)
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
struct ResponseBodyDataPoint {
    chart_id: String,
    created_at: String,
    id: String,
    x_value: String,
    y_value: u32,
}

impl From<query_use_case::list_data_points::DataPoint> for ResponseBodyDataPoint {
    fn from(
        query_use_case::list_data_points::DataPoint {
            chart_id,
            created_at,
            x_value,
            y_value,
        }: query_use_case::list_data_points::DataPoint,
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

async fn handler<T: HasListDataPoints>(
    State(state): State<T>,
    Path(path_parameters): Path<PathParameters>,
) -> Result<Json<ResponseBody>, StatusCode> {
    let use_case = state.list_data_points();
    let input = query_use_case::list_data_points::Input::from(path_parameters);
    let output = use_case
        .execute(input)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ResponseBody::from(output)))
}

pub fn router<T: Clone + HasListDataPoints + Send + Sync + 'static>() -> Router<T> {
    Router::new().route(
        "/charts/:chart_id/data_points",
        axum::routing::get(handler::<T>),
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use query_use_case::{list_data_points::DataPoint, list_data_points::MockListDataPoints};

    use crate::server::handler::tests::{send_request, ResponseExt as _};

    use super::*;

    #[tokio::test]
    async fn test_happy_path() -> anyhow::Result<()> {
        let data_point = build_data_point();
        let mocks = Mocks::with_happy_path_behavior(data_point.clone());
        let app = router().with_state(mocks.clone());
        let request = build_request(&data_point.chart_id)?;
        let response = send_request(app, request).await?;
        assert_eq!(response.status(), StatusCode::OK);
        let id = format!("{}:{}", data_point.chart_id, data_point.x_value);
        assert_eq!(
            response.into_body_as_json::<ResponseBody>().await?,
            ResponseBody {
                data_points: vec![ResponseBodyDataPoint {
                    chart_id: data_point.chart_id,
                    created_at: data_point.created_at,
                    id,
                    x_value: data_point.x_value,
                    y_value: data_point.y_value,
                }]
            }
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_error() -> anyhow::Result<()> {
        let data_point = build_data_point();
        let mut mocks = Mocks::with_happy_path_behavior(data_point.clone());
        mocks.list_data_points = {
            let mut mock = MockListDataPoints::new();
            mock.expect_execute().return_once(|_| {
                Err(query_use_case::list_data_points::Error::DataPointList(
                    build_error(),
                ))
            });
            Arc::new(mock)
        };
        let app = router().with_state(mocks.clone());
        let request = build_request(&data_point.chart_id)?;
        let response = send_request(app, request).await?;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(response.into_body_string().await?, "");
        Ok(())
    }

    #[derive(Clone)]
    struct Mocks {
        list_data_points: Arc<MockListDataPoints>,
    }

    impl Mocks {
        fn with_happy_path_behavior(data_point: DataPoint) -> Self {
            let mut list_data_points = MockListDataPoints::new();
            list_data_points.expect_execute().return_once(move |_| {
                Ok(query_use_case::list_data_points::Output(vec![
                    data_point.clone()
                ]))
            });
            Self {
                list_data_points: Arc::new(list_data_points),
            }
        }
    }

    impl query_use_case::list_data_points::HasListDataPoints for Mocks {
        fn list_data_points(
            &self,
        ) -> Arc<dyn query_use_case::list_data_points::ListDataPoints + Send + Sync> {
            self.list_data_points.clone()
        }
    }

    fn build_data_point() -> DataPoint {
        DataPoint {
            chart_id: "chart_id1".to_string(),
            created_at: "created_at1".to_string(),
            x_value: "2020-01-02".to_string(),
            y_value: 123_u32,
        }
    }

    fn build_error() -> Box<dyn std::error::Error + Send + Sync> {
        Box::new(std::io::Error::new(std::io::ErrorKind::Other, "error"))
    }

    fn build_request(chart_id: &str) -> anyhow::Result<axum::http::Request<axum::body::Body>> {
        Ok(axum::http::Request::builder()
            .method(axum::http::Method::GET)
            .uri(format!("/charts/{}/data_points", chart_id))
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(axum::body::Body::empty())?)
    }
}
