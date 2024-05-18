pub async fn run() -> anyhow::Result<()> {
    let router = axum::Router::new().route("/", axum::routing::get(|| async { "OK" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, router).await?;
    Ok(())
}
