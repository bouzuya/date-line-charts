mod handler;

pub async fn run() -> anyhow::Result<()> {
    let router = handler::router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, router).await?;
    Ok(())
}
