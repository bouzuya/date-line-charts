use std::sync::Arc;

use in_memory_store::{InMemoryChartStore, InMemoryDataPointStore};

pub async fn run() -> anyhow::Result<()> {
    let chart_store = Arc::new(InMemoryChartStore::new());
    let data_point_store = Arc::new(InMemoryDataPointStore::new());
    let app = server::App::new(chart_store.clone(), chart_store, data_point_store);
    Ok(server::run(app).await?)
}
