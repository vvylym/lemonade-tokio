//! Lemonade worker Axum
//!
mod handler;
mod router;

use lemonade_service::{AppState, config::Config};
use router::create_router;
use tokio::net::TcpListener;

/// Run the Axum worker server
pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with service name from config and worker package version
    lemonade_observability::init_tracing(
        "lemonade-worker-axum",
        env!("CARGO_PKG_VERSION"),
        config.service_name(),
        config.otlp_endpoint(),
        config.otlp_protocol(),
    )?;

    // Initialize metrics
    lemonade_observability::init_metrics(
        "lemonade-worker-axum",
        env!("CARGO_PKG_VERSION"),
        config.service_name(),
        config.otlp_endpoint(),
        config.otlp_protocol(),
    )?;

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
