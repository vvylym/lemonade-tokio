//! Handler module
//!
use axum::{extract::State, http::StatusCode, response::Json};
use lemonade_service::AppState;
use lemonade_service::{
    error_response::ErrorResponse,
    worker::{HealthResponse, HealthService, WorkResponse, WorkService},
};
use std::time::Instant;
use tracing::instrument;

type HealthHandlerResult =
    Result<Json<HealthResponse>, (StatusCode, Json<ErrorResponse>)>;
type WorkHandlerResult = Result<Json<WorkResponse>, (StatusCode, Json<ErrorResponse>)>;

/// Health check handler
#[instrument(skip(state), fields(framework.name = "axum", http.route = "/health"))]
pub async fn health_handler(State(state): State<AppState>) -> HealthHandlerResult {
    let start = Instant::now();
    let metrics = lemonade_observability::get_http_metrics("lemonade-worker-axum");

    let result = match state.worker_service.health_check().await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("{}", e))),
        )),
    };

    let status = result.as_ref().map(|_| 200).unwrap_or(500);
    let duration_micros = start.elapsed().as_micros() as u64;
    metrics.record_request("GET", "/health", status, duration_micros);

    result
}

/// Work handler
#[instrument(skip(state), fields(framework.name = "axum", http.route = "/work"))]
pub async fn work_handler(State(state): State<AppState>) -> WorkHandlerResult {
    let start = Instant::now();
    let metrics = lemonade_observability::get_http_metrics("lemonade-worker-axum");

    let result = match state.worker_service.work().await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("{}", e))),
        )),
    };

    let status = result.as_ref().map(|_| 200).unwrap_or(500);
    let duration_micros = start.elapsed().as_micros() as u64;
    metrics.record_request("GET", "/work", status, duration_micros);

    result
}
