use std::sync::Arc;

use in_memory_app::{InMemoryApp, InMemoryChartStore};

pub async fn run() -> anyhow::Result<()> {
    let store = Arc::new(InMemoryChartStore::new());
    let app = InMemoryApp::new(store.clone(), store);
    Ok(server::run(app).await?)
}
