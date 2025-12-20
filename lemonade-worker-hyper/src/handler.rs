//! Handler module
//!
use http_body_util::Full;
use hyper::{Request, Response, StatusCode, body::Bytes};
use lemonade_service::AppState;
use lemonade_service::{
    error_response::ErrorResponse,
    worker::{HealthService, WorkService},
};
use opentelemetry::global;
use opentelemetry_http::HeaderExtractor;
use std::convert::Infallible;
use std::time::Instant;
use tracing::{Instrument, instrument};

/// Handle HTTP request
pub async fn handle_request(
    req: Request<hyper::body::Incoming>,
    state: AppState,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let start = Instant::now();
    let metrics = lemonade_observability::get_http_metrics("lemonade-worker-hyper");

    // Extract trace context from headers for distributed tracing
    let extractor = HeaderExtractor(req.headers());
    let _parent_cx = global::get_text_map_propagator(|prop| prop.extract(&extractor));

    // Create span with HTTP attributes
    let method = req.method().clone();
    let method_str = method.to_string();
    let path = req.uri().path().to_string();

    let span = tracing::span!(
        tracing::Level::INFO,
        "http_request",
        framework.name = "hyper",
        http.method = %method,
        http.route = %path,
        http.scheme = %req.uri().scheme_str().unwrap_or("http"),
        http.target = %req.uri().path_and_query().map(|p| p.as_str()).unwrap_or(""),
    );

    // Execute handler within the span context
    let result = handle_request_inner(req, state, path.clone())
        .instrument(span)
        .await;

    // Record metrics
    let status_code = result.as_ref().map(|r| r.status().as_u16()).unwrap_or(500);
    let duration_micros = start.elapsed().as_micros() as u64;
    metrics.record_request(&method_str, &path, status_code, duration_micros);

    result
}

#[instrument(skip(state), fields(framework.name = "hyper", http.route = %path))]
async fn handle_request_inner(
    req: Request<hyper::body::Incoming>,
    state: AppState,
    path: String,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let response = match path.as_str() {
        "/health" => match state.worker_service.health_check().await {
            Ok(response) => {
                let json = serde_json::to_string(&response).unwrap_or_default();
                let resp = Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(Full::new(Bytes::from(json)))
                    .unwrap();
                tracing::Span::current().record("http.status_code", 200);
                resp
            }
            Err(e) => {
                let error = ErrorResponse::new(format!("{}", e));
                let json = serde_json::to_string(&error).unwrap_or_default();
                let resp = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("Content-Type", "application/json")
                    .body(Full::new(Bytes::from(json)))
                    .unwrap();
                tracing::Span::current().record("http.status_code", 500);
                resp
            }
        },
        "/work" => match state.worker_service.work().await {
            Ok(response) => {
                let json = serde_json::to_string(&response).unwrap_or_default();
                let resp = Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(Full::new(Bytes::from(json)))
                    .unwrap();
                tracing::Span::current().record("http.status_code", 200);
                resp
            }
            Err(e) => {
                let error = ErrorResponse::new(format!("{}", e));
                let json = serde_json::to_string(&error).unwrap_or_default();
                let resp = Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("Content-Type", "application/json")
                    .body(Full::new(Bytes::from(json)))
                    .unwrap();
                tracing::Span::current().record("http.status_code", 500);
                resp
            }
        },
        _ => {
            let error = ErrorResponse::new("Not Found");
            let json = serde_json::to_string(&error).unwrap_or_default();
            let resp = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "application/json")
                .body(Full::new(Bytes::from(json)))
                .unwrap();
            tracing::Span::current().record("http.status_code", 404);
            resp
        }
    };
    Ok(response)
}
