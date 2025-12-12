//! Handler module
//!
use lemonade_service::{
    AppState,
    error_response::ErrorResponse,
    worker::{HealthService, WorkService},
};
use rocket::serde::json::Json;

/// Health check handler
#[rocket::get("/health")]
pub async fn health_handler(
    state: &rocket::State<AppState>,
) -> Result<
    Json<lemonade_service::worker::HealthResponse>,
    (rocket::http::Status, Json<ErrorResponse>),
> {
    match state.worker_service.health_check().await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            rocket::http::Status::InternalServerError,
            Json(ErrorResponse::new(format!("{}", e))),
        )),
    }
}

/// Work handler
#[rocket::get("/work")]
pub async fn work_handler(
    state: &rocket::State<AppState>,
) -> Result<
    Json<lemonade_service::worker::WorkResponse>,
    (rocket::http::Status, Json<ErrorResponse>),
> {
    match state.worker_service.work().await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            rocket::http::Status::InternalServerError,
            Json(ErrorResponse::new(format!("{}", e))),
        )),
    }
}
