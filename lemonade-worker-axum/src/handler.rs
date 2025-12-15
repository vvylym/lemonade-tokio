//! Handler module
//!
use axum::{extract::State, http::StatusCode, response::Json};
use lemonade_service::AppState;
use lemonade_service::{
    error_response::ErrorResponse,
    worker::{HealthResponse, HealthService, WorkResponse, WorkService},
};
use tracing::instrument;

type HealthHandlerResult =
    Result<Json<HealthResponse>, (StatusCode, Json<ErrorResponse>)>;
type WorkHandlerResult = Result<Json<WorkResponse>, (StatusCode, Json<ErrorResponse>)>;

/// Health check handler
#[instrument(skip(state), fields(http.method = "GET", http.route = "/health"))]
pub async fn health_handler(State(state): State<AppState>) -> HealthHandlerResult {
    match state.worker_service.health_check().await {
        Ok(response) => {
            tracing::Span::current().record("http.status_code", 200);
            Ok(Json(response))
        }
        Err(e) => {
            tracing::Span::current().record("http.status_code", 500);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("{}", e))),
            ))
        }
    }
}

/// Work handler
#[instrument(skip(state), fields(http.method = "GET", http.route = "/work"))]
pub async fn work_handler(State(state): State<AppState>) -> WorkHandlerResult {
    match state.worker_service.work().await {
        Ok(response) => {
            tracing::Span::current().record("http.status_code", 200);
            Ok(Json(response))
        }
        Err(e) => {
            tracing::Span::current().record("http.status_code", 500);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!("{}", e))),
            ))
        }
    }
}
