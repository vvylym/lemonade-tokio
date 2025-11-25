//! //! Axum Worker
//!
//! This server provides two endpoints:
//! - GET /health: Health check endpoint
//! - GET /work: Work endpoint that simulates processing with a 20ms delay

use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;
use tracing::{Level, debug, error, info, span, trace};

mod handlers;

use handlers::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber with environment filter
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let span = span!(Level::INFO, "axum_server_startup");
    let _enter = span.enter();

    // Get port from environment variable or default to 4002
    let addr = std::env::var("AXUM_WORKER_ADDRESS").expect("failed to retrieve worker address");

    info!(bind_addr = %addr, "Axum server starting");

    info!("Registering routes: /health, /work");
    let app = Router::new()
        .route("/health", get(health::handle_health_check))
        .route("/work", get(work::handle_work))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                    )
                })
                .on_request(|_request: &axum::http::Request<_>, _span: &tracing::Span| {
                    trace!("Received HTTP request");
                })
                .on_response(
                    |_response: &axum::http::Response<_>,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        debug!(latency_ms = latency.as_millis(), "HTTP response sent");
                    },
                )
                .on_failure(
                    |_failure_class: tower_http::classify::ServerErrorsFailureClass,
                     _latency: std::time::Duration,
                     _span: &tracing::Span| {
                        error!("HTTP request failed");
                    },
                ),
        );

    let listener = tokio::net::TcpListener::bind(addr.to_owned())
        .await
        .map_err(|e| {
            error!(error = %e, bind_addr = %addr, "Failed to bind server");
            e
        })
        .unwrap();

    info!("Server listening, ready to accept connections");

    axum::serve(listener, app)
        .await
        .map_err(|e| {
            error!(error = %e, "Server error");
            e
        })
        .unwrap();

    Ok(())
}
