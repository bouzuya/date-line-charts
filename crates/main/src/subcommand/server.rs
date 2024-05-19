pub async fn run() -> anyhow::Result<()> {
    Ok(server::run().await?)
}
