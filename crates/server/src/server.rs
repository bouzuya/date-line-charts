mod app_state;
mod handler;

use self::app_state::AppState;

pub async fn run() -> anyhow::Result<()> {
    let router = handler::router().with_state(AppState::new());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, router).await?;
    Ok(())
}
