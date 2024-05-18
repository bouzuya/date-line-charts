mod show_root;

pub fn router() -> axum::Router {
    axum::Router::new().merge(show_root::router())
}
