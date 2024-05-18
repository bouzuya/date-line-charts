mod create_chart;
mod delete_chart;
mod get_chart;
mod get_root;
mod list_charts;
mod update_chart;

use axum::Router;

use crate::{
    command_use_case::{
        create_chart::HasCreateChart, delete_chart::HasDeleteChart, update_chart::HasUpdateChart,
    },
    query_use_case::{get_chart::HasGetChart, list_charts::HasListCharts},
};

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
