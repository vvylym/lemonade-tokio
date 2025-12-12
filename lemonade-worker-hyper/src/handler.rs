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

/// Handle HTTP request
pub async fn handle_request(
    req: Request<hyper::body::Incoming>,
    state: AppState,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = req.uri().path();

    match path {
        "/health" => match state.worker_service.health_check().await {
            Ok(response) => {
                let json = serde_json::to_string(&response).unwrap_or_default();
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(Full::new(Bytes::from(json)))
                    .unwrap())
            }
            Err(e) => {
                let error = ErrorResponse::new(format!("{}", e));
                let json = serde_json::to_string(&error).unwrap_or_default();
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("Content-Type", "application/json")
                    .body(Full::new(Bytes::from(json)))
                    .unwrap())
            }
        },
        "/work" => match state.worker_service.work().await {
            Ok(response) => {
                let json = serde_json::to_string(&response).unwrap_or_default();
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(Full::new(Bytes::from(json)))
                    .unwrap())
            }
            Err(e) => {
                let error = ErrorResponse::new(format!("{}", e));
                let json = serde_json::to_string(&error).unwrap_or_default();
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("Content-Type", "application/json")
                    .body(Full::new(Bytes::from(json)))
                    .unwrap())
            }
        },
        _ => {
            let error = ErrorResponse::new("Not Found");
            let json = serde_json::to_string(&error).unwrap_or_default();
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "application/json")
                .body(Full::new(Bytes::from(json)))
                .unwrap())
        }
    }
}
