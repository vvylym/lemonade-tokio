//! Handler module
//!
use http_body_util::Full;
use hyper::{Request, Response, StatusCode, body::Bytes};
use lemonade_service::AppState;
use lemonade_service::{
    error_response::ErrorResponse,
    worker::{HealthService, WorkService},
};
use std::convert::Infallible;
use tracing::instrument;

/// Handle HTTP request
#[instrument(skip(state), fields(http.method = %req.method(), http.route = %req.uri().path()))]
pub async fn handle_request(
    req: Request<hyper::body::Incoming>,
    state: AppState,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = req.uri().path();

    let response = match path {
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
