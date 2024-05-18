mod create_chart;
mod show_root;

use axum::Router;

use super::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(create_chart::router())
        .merge(show_root::router())
}
