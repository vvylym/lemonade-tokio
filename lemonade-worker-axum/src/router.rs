//! Router module
//!
use crate::handler;
use axum::{Router, routing::get};
use lemonade_service::AppState;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::trace::TraceLayer;
use tracing::Level;

/// Create the application router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handler::health_handler))
        .route("/work", get(handler::work_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    tracing::span!(
                        Level::INFO,
                        "http_request",
                        framework.name = "axum",
                        http.method = %request.method(),
                        http.route = %request.uri().path(),
                        http.scheme = %request.uri().scheme_str().unwrap_or("http"),
                        http.target = %request.uri().path_and_query().map(|p| p.as_str()).unwrap_or(""),
                    )
                })
                .on_request(|_request: &axum::http::Request<_>, _span: &tracing::Span| {
                    tracing::event!(Level::DEBUG, "request started");
                })
                .on_response(|_response: &axum::http::Response<_>, latency: std::time::Duration, _span: &tracing::Span| {
                    tracing::event!(
                        Level::INFO,
                        http.status_code = %_response.status().as_u16(),
                        latency_ms = latency.as_millis(),
                        "request completed"
                    );
                })
                .on_failure(|_error: ServerErrorsFailureClass, _latency: std::time::Duration, _span: &tracing::Span| {
                    tracing::event!(Level::ERROR, "request failed");
                }),
        )
        .with_state(state)
}
