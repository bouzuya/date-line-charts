mod create_chart;
mod delete_chart;
mod show_chart;
mod show_root;
mod update_chart;

use axum::Router;

use super::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(create_chart::router())
        .merge(delete_chart::router())
        .merge(show_chart::router())
        .merge(show_root::router())
        .merge(update_chart::router())
}
