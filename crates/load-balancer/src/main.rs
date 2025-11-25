mod algorithms;
mod backend;
mod config;
mod error;
mod proxy;
mod state;
mod strategies;

use std::sync::Arc;

use tokio::{net::TcpListener, sync::broadcast};
use tracing::{Level, error, info, span};
use tracing_subscriber::EnvFilter;

use config::Config;
use error::Result;
use proxy::handle_proxy;
use state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber with environment filter
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let span = span!(Level::INFO, "load_balancer_startup");
    let _enter = span.enter();

    info!("Initializing load balancer");

    // load configuration
    let config = Config::parse()?;

    // Create application state with metrics manager and backend state
    let app_state = Arc::new(AppState::new(&config));

    // Setup graceful shutdown mechanism using broadcast channel
    let (shutdown_sender, _) = broadcast::channel(1);
    let _shutdown_receiver = shutdown_sender.subscribe();

    // Start listening for incoming client connections on all interfaces
    let listener = TcpListener::bind(&config.address)
        .await
        .expect("failed to listen to the address");
    info!("Load balancer listening {}", config.address);

    // TODO: Spawn a task to handle health checks

    // TODO: Spawn a task to handle metrics

    // Main accept loop - handles new client connections
    loop {
        let accept_future = listener.accept();

        tokio::select! {
            result = accept_future => {
                match result {
                    Ok((stream, addr)) => {
                        info!(client_addr = %addr, "New connection accepted");
                        // TODO: handle metrics

                        // Clone app state and create shutdown receiver for this connection
                        let state = app_state.clone();
                        let shutdown_receiver = shutdown_sender.subscribe();

                        // Spawn task to handle this connection asynchronously
                        tokio::spawn(async move {
                            // TODO: Maybe record the start for metrics
                            if let Err(e) = handle_proxy(state, stream, shutdown_receiver).await {
                                error!(error = %e, "Error handling connection");
                                // TODO: record connection error
                            }
                            // TODO: Maybe record the duration for live metrics
                        });
                    }
                    Err(e) => {
                        error!(error = %e, "Accept error");
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                // Handle Ctrl+C for graceful shutdown
                info!("Received shutdown signal, shutting down gracefully...");
                break;
            }
        }

        break;
    }
    // Graceful shutdown sequence
    info!("Initiating graceful shutdown");
    // Dropping the sender will signal all receivers (including health checker) to shutdown
    // Wait for health checker task to complete after signaling shutdown

    Ok(())
}
