//! Handler module
//!
use lemonade_service::{
    AppState,
    error_response::ErrorResponse,
    worker::{HealthResponse, HealthService, WorkResponse, WorkService},
};
use rocket::serde::json::Json;
use tracing::instrument;

type HealthHandlerResult =
    Result<Json<HealthResponse>, (rocket::http::Status, Json<ErrorResponse>)>;
type WorkHandlerResult =
    Result<Json<WorkResponse>, (rocket::http::Status, Json<ErrorResponse>)>;

/// Health check handler
#[rocket::get("/health")]
#[instrument(skip(state), fields(http.method = "GET", http.route = "/health"))]
pub async fn health_handler(state: &rocket::State<AppState>) -> HealthHandlerResult {
    match state.worker_service.health_check().await {
        Ok(response) => {
            tracing::Span::current().record("http.status_code", 200);
            Ok(Json(response))
        }
        Err(e) => {
            tracing::Span::current().record("http.status_code", 500);
            Err((
                rocket::http::Status::InternalServerError,
                Json(ErrorResponse::new(format!("{}", e))),
            ))
        }
    }
}

/// Work handler
#[rocket::get("/work")]
#[instrument(skip(state), fields(http.method = "GET", http.route = "/work"))]
pub async fn work_handler(state: &rocket::State<AppState>) -> WorkHandlerResult {
    match state.worker_service.work().await {
        Ok(response) => {
            tracing::Span::current().record("http.status_code", 200);
            Ok(Json(response))
        }
        Err(e) => {
            tracing::Span::current().record("http.status_code", 500);
            Err((
                rocket::http::Status::InternalServerError,
                Json(ErrorResponse::new(format!("{}", e))),
            ))
        }
    }
}
