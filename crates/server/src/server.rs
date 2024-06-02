mod app;
mod handler;

use command_use_case::{
    create_chart::HasCreateChart, create_data_point::HasCreateDataPoint,
    delete_chart::HasDeleteChart, update_chart::HasUpdateChart,
    update_data_point::HasUpdateDataPoint,
};
use query_use_case::{get_chart::HasGetChart, list_charts::HasListCharts};

pub use self::app::App;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bind")]
    Bind(#[source] std::io::Error),
    #[error("serve")]
    Serve(#[source] std::io::Error),
}

pub async fn run<
    T: Clone
        + HasCreateChart
        + HasCreateDataPoint
        + HasDeleteChart
        + HasGetChart
        + HasListCharts
        + HasUpdateChart
        + HasUpdateDataPoint
        + Send
        + Sync
        + 'static,
>(
    app: T,
) -> Result<(), Error> {
    let router = handler::router().with_state(app);
    let tcp_listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .map_err(Error::Bind)?;
    axum::serve(tcp_listener, router)
        .await
        .map_err(Error::Serve)?;
    Ok(())
}
