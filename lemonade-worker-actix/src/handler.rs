//! Handler module
//!
use actix_web::{HttpResponse, Responder, web};
use lemonade_service::AppState;
use lemonade_service::{
    error_response::ErrorResponse,
    worker::{HealthService, WorkService},
};
use std::time::Instant;
use tracing::instrument;

/// Health check handler
#[instrument(skip(state), fields(framework.name = "actix-web", http.route = "/health"))]
pub async fn health_handler(state: web::Data<AppState>) -> impl Responder {
    let start = Instant::now();
    let metrics = lemonade_observability::get_http_metrics("lemonade-worker-actix");

    match state.worker_service.health_check().await {
        Ok(response) => {
            let status = 200;
            let duration_micros = start.elapsed().as_micros() as u64;
            metrics.record_request("GET", "/health", status, duration_micros);
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let status = 500;
            let duration_micros = start.elapsed().as_micros() as u64;
            metrics.record_request("GET", "/health", status, duration_micros);
            HttpResponse::InternalServerError().json(ErrorResponse::new(format!("{}", e)))
        }
    }
}

/// Work handler
#[instrument(skip(state), fields(framework.name = "actix-web", http.route = "/work"))]
pub async fn work_handler(state: web::Data<AppState>) -> impl Responder {
    let start = Instant::now();
    let metrics = lemonade_observability::get_http_metrics("lemonade-worker-actix");

    match state.worker_service.work().await {
        Ok(response) => {
            let status = 200;
            let duration_micros = start.elapsed().as_micros() as u64;
            metrics.record_request("GET", "/work", status, duration_micros);
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let status = 500;
            let duration_micros = start.elapsed().as_micros() as u64;
            metrics.record_request("GET", "/work", status, duration_micros);
            HttpResponse::InternalServerError().json(ErrorResponse::new(format!("{}", e)))
        }
    }
}
