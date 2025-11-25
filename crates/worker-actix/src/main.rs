//! Actix Web HTTP Server Example
//!
//! This server provides two endpoints:
//! - GET /health: Health check endpoint
//! - GET /work: Work endpoint that simulates processing with a 20ms delay
use actix_web::{App, HttpServer, middleware::Logger, web};
use tracing::{Level, info, span};
use tracing_subscriber::EnvFilter;

mod handlers;
use handlers::{health::handle_health_check, work::handle_work};

/// Main entrypoint
#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber with environment filter
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let span = span!(Level::INFO, "actix_server_startup");
    let _enter = span.enter();

    // Get port from environment variable or default to 4002
    let addr = std::env::var("ACTIX_WORKER_ADDRESS").expect("failed to retrieve worker address");

    info!(bind_addr = %addr, "Actix Web server starting");

    info!("Registering routes: /health, /work");
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/health", web::get().to(handle_health_check))
            .route("/work", web::get().to(handle_work))
    })
    .bind(addr)?
    .run()
    .await?;

    Ok(())
}
