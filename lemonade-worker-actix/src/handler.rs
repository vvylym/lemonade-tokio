//! Handler module
//!
use actix_web::{HttpResponse, Responder, web};
use lemonade_service::AppState;
use lemonade_service::{
    error_response::ErrorResponse,
    worker::{HealthService, WorkService},
};
use tracing::instrument;

/// Health check handler
#[instrument(skip(state), fields(http.method = "GET", http.route = "/health"))]
pub async fn health_handler(state: web::Data<AppState>) -> impl Responder {
    match state.worker_service.health_check().await {
        Ok(response) => {
            tracing::Span::current().record("http.status_code", 200);
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            tracing::Span::current().record("http.status_code", 500);
            HttpResponse::InternalServerError().json(ErrorResponse::new(format!("{}", e)))
        }
    }
}

/// Work handler
#[instrument(skip(state), fields(http.method = "GET", http.route = "/work"))]
pub async fn work_handler(state: web::Data<AppState>) -> impl Responder {
    match state.worker_service.work().await {
        Ok(response) => {
            tracing::Span::current().record("http.status_code", 200);
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            tracing::Span::current().record("http.status_code", 500);
            HttpResponse::InternalServerError().json(ErrorResponse::new(format!("{}", e)))
        }
    }
}
