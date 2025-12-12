//! Handler module
//!
use actix_web::{HttpResponse, Responder, web};
use lemonade_service::AppState;
use lemonade_service::{
    error_response::ErrorResponse,
    worker::{HealthService, WorkService},
};

/// Health check handler
pub async fn health_handler(state: web::Data<AppState>) -> impl Responder {
    match state.worker_service.health_check().await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            HttpResponse::InternalServerError().json(ErrorResponse::new(format!("{}", e)))
        }
    }
}

/// Work handler
pub async fn work_handler(state: web::Data<AppState>) -> impl Responder {
    match state.worker_service.work().await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            HttpResponse::InternalServerError().json(ErrorResponse::new(format!("{}", e)))
        }
    }
}
