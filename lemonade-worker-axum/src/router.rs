//! Router module
//!
use crate::handler;
use axum::{Router, routing::get};
use lemonade_service::AppState;

/// Create the application router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handler::health_handler))
        .route("/work", get(handler::work_handler))
        .with_state(state)
}
