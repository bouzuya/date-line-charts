use in_memory_app::InMemoryApp;

pub async fn run() -> anyhow::Result<()> {
    let app = InMemoryApp::new();
    Ok(server::run(app).await?)
}
