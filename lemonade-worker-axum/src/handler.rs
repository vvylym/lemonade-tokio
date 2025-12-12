//! Handler module
//!
use axum::{extract::State, http::StatusCode, response::Json};
use lemonade_service::AppState;
use lemonade_service::{
    error_response::ErrorResponse,
    worker::{HealthService, WorkService},
};

/// Health check handler
pub async fn health_handler(
    State(state): State<AppState>,
) -> Result<
    Json<lemonade_service::worker::HealthResponse>,
    (StatusCode, Json<ErrorResponse>),
> {
    match state.worker_service.health_check().await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("{}", e))),
        )),
    }
}

/// Work handler
pub async fn work_handler(
    State(state): State<AppState>,
) -> Result<Json<lemonade_service::worker::WorkResponse>, (StatusCode, Json<ErrorResponse>)>
{
    match state.worker_service.work().await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!("{}", e))),
        )),
    }
}
