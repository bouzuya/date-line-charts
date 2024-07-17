mod app;
mod handler;

use std::{env, net::Ipv4Addr};

use command_use_case::{
    create_chart::HasCreateChart, create_data_point::HasCreateDataPoint,
    delete_chart::HasDeleteChart, delete_data_point::HasDeleteDataPoint,
    update_chart::HasUpdateChart, update_data_point::HasUpdateDataPoint,
};
use query_use_case::{
    get_chart::HasGetChart, get_data_point::HasGetDataPoint, list_charts::HasListCharts,
    list_data_points::HasListDataPoints,
};

pub use self::app::App;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bind")]
    Bind(#[source] std::io::Error),
    #[error("invalid port")]
    InvalidPort(#[source] std::num::ParseIntError),
    #[error("serve")]
    Serve(#[source] std::io::Error),
}

pub async fn run<
    T: Clone
        + HasCreateChart
        + HasCreateDataPoint
        + HasDeleteChart
        + HasDeleteDataPoint
        + HasGetChart
        + HasGetDataPoint
        + HasListCharts
        + HasListDataPoints
        + HasUpdateChart
        + HasUpdateDataPoint
        + Send
        + Sync
        + 'static,
>(
    app: T,
) -> Result<(), Error> {
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_owned())
        .parse::<u16>()
        .map_err(Error::InvalidPort)?;
    let router = handler::router().with_state(app);
    let tcp_listener = tokio::net::TcpListener::bind((Ipv4Addr::UNSPECIFIED, port))
        .await
        .map_err(Error::Bind)?;
    tracing::info!("server listening on port 3000");
    axum::serve(tcp_listener, router)
        .await
        .map_err(Error::Serve)?;
    Ok(())
}
