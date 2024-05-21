use std::sync::Arc;

use in_memory_store::InMemoryChartStore;

pub async fn run() -> anyhow::Result<()> {
    let store = Arc::new(InMemoryChartStore::new());
    let app = server::App::new(store.clone(), store);
    Ok(server::run(app).await?)
}
