mod app_state;
mod handler;

use self::app_state::AppState;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bind")]
    Bind(#[source] std::io::Error),
    #[error("serve")]
    Serve(#[source] std::io::Error),
}

pub async fn run() -> Result<(), Error> {
    let router = handler::router().with_state(AppState::new());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .map_err(Error::Bind)?;
    axum::serve(listener, router).await.map_err(Error::Serve)?;
    Ok(())
}
