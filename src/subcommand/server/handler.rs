mod create_chart;
mod delete_chart;
mod show_chart;
mod show_root;
mod update_chart;

use axum::Router;

use crate::{
    command_use_case::{
        create_chart::HasCreateChart, delete_chart::HasDeleteChart, update_chart::HasUpdateChart,
    },
    query_use_case::show_chart::HasShowChart,
};

pub fn router<
    T: Clone + HasCreateChart + HasDeleteChart + HasShowChart + HasUpdateChart + Send + Sync + 'static,
>() -> Router<T> {
    Router::new()
        .merge(create_chart::router())
        .merge(delete_chart::router())
        .merge(show_chart::router())
        .merge(show_root::router())
        .merge(update_chart::router())
}
