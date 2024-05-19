mod create_chart;
mod delete_chart;
mod get_chart;
mod get_root;
mod list_charts;
mod update_chart;

use axum::Router;

use command_use_case::{
    create_chart::HasCreateChart, delete_chart::HasDeleteChart, update_chart::HasUpdateChart,
};
use query_use_case::{get_chart::HasGetChart, list_charts::HasListCharts};

pub fn router<
    T: Clone
        + HasCreateChart
        + HasDeleteChart
        + HasGetChart
        + HasListCharts
        + HasUpdateChart
        + Send
        + Sync
        + 'static,
>() -> Router<T> {
    Router::new()
        .merge(create_chart::router())
        .merge(delete_chart::router())
        .merge(get_chart::router())
        .merge(get_root::router())
        .merge(list_charts::router())
        .merge(update_chart::router())
}

#[cfg(test)]
mod tests {
    #[axum::async_trait]
    pub(crate) trait ResponseExt {
        async fn into_body_string(self) -> anyhow::Result<String>;
        async fn into_body_as_json<T: serde::de::DeserializeOwned>(self) -> anyhow::Result<T>;
    }

    #[axum::async_trait]
    impl ResponseExt for axum::http::Response<axum::body::Body> {
        async fn into_body_string(self) -> anyhow::Result<String> {
            let body = axum::body::to_bytes(self.into_body(), usize::MAX).await?;
            Ok(String::from_utf8(body.to_vec())?)
        }
        async fn into_body_as_json<T: serde::de::DeserializeOwned>(self) -> anyhow::Result<T> {
            Ok(serde_json::from_str(&self.into_body_string().await?)?)
        }
    }

    pub(crate) async fn send_request(
        app: axum::Router,
        request: axum::http::Request<axum::body::Body>,
    ) -> anyhow::Result<axum::response::Response<axum::body::Body>> {
        Ok(tower::ServiceExt::oneshot(app, request).await?)
    }
}
