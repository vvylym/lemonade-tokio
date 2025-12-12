//! Lemonade worker Axum
//!
mod handler;
mod router;

use lemonade_service::{AppState, config::Config};
use router::create_router;
use tokio::net::TcpListener;

/// Run the Axum worker server
pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState::new(config);
    let app = create_router(state.clone());

    let listener = TcpListener::bind(state.config.listen_address().as_ref()).await?;
    println!(
        "Axum worker listening on {}",
        state.config.listen_address().as_ref()
    );

    axum::serve(listener, app).await?;
    Ok(())
}
