use std::sync::Arc;

use firestore_store::{FirestoreChartStore, FirestoreDataPointStore};

pub async fn run() -> anyhow::Result<()> {
    let chart_store = Arc::new(
        FirestoreChartStore::new()
            .await
            .map_err(|e| anyhow::anyhow!(e))?,
    );
    let data_point_store = Arc::new(
        FirestoreDataPointStore::new()
            .await
            .map_err(|e| anyhow::anyhow!(e))?,
    );
    let app = server::App::new(
        chart_store.clone(),
        chart_store,
        data_point_store.clone(),
        data_point_store,
    );
    Ok(server::run(app).await?)
}
