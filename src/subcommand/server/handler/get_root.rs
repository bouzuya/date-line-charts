use axum::Router;

async fn handler() -> &'static str {
    "OK"
}

pub fn router<T: Clone + Send + Sync + 'static>() -> Router<T> {
    Router::new().route("/", axum::routing::get(handler))
}
