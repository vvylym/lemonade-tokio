//! Handler module
//!
use lemonade_service::{
    AppState,
    error_response::ErrorResponse,
    worker::{HealthResponse, HealthService, WorkResponse, WorkService},
};
use rocket::serde::json::Json;
use std::time::Instant;
use tracing::instrument;

type HealthHandlerResult =
    Result<Json<HealthResponse>, (rocket::http::Status, Json<ErrorResponse>)>;
type WorkHandlerResult =
    Result<Json<WorkResponse>, (rocket::http::Status, Json<ErrorResponse>)>;

/// Health check handler
#[rocket::get("/health")]
#[instrument(skip(state), fields(framework.name = "rocket", http.route = "/health"))]
pub async fn health_handler(state: &rocket::State<AppState>) -> HealthHandlerResult {
    let start = Instant::now();
    let metrics = lemonade_observability::get_http_metrics("lemonade-worker-rocket");

    let result = match state.worker_service.health_check().await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            rocket::http::Status::InternalServerError,
            Json(ErrorResponse::new(format!("{}", e))),
        )),
    };

    let status = result.as_ref().map(|_| 200).unwrap_or(500);
    let duration_micros = start.elapsed().as_micros() as u64;
    metrics.record_request("GET", "/health", status, duration_micros);

    result
}

/// Work handler
#[rocket::get("/work")]
#[instrument(skip(state), fields(framework.name = "rocket", http.route = "/work"))]
pub async fn work_handler(state: &rocket::State<AppState>) -> WorkHandlerResult {
    let start = Instant::now();
    let metrics = lemonade_observability::get_http_metrics("lemonade-worker-rocket");

    let result = match state.worker_service.work().await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            rocket::http::Status::InternalServerError,
            Json(ErrorResponse::new(format!("{}", e))),
        )),
    };

    let status = result.as_ref().map(|_| 200).unwrap_or(500);
    let duration_micros = start.elapsed().as_micros() as u64;
    metrics.record_request("GET", "/work", status, duration_micros);

    result
}
