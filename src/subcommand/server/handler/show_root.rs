async fn handler() -> &'static str {
    "OK"
}

pub fn router() -> axum::Router {
    axum::Router::new().route("/", axum::routing::get(handler))
}
