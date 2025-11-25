//! Handlers module for the Axum Worker
//!
pub mod health {
    //! Health Handler module
    //!
    use axum::Json;
    use serde::{Deserialize, Serialize};
    use tracing::{instrument, trace};

    /// Health check endpoint
    ///
    /// Returns a JSON response indicating the service is healthy.
    #[instrument(fields(service = "axum"))]
    pub async fn handle_health_check() -> Json<HealthResponse> {
        trace!("Health check endpoint called");
        let response = HealthResponse {
            status: "ok".to_string(),
            service: "axum".to_string(),
        };
        Json(response)
    }

    /// Health check response structure
    #[derive(Serialize, Deserialize)]
    pub struct HealthResponse {
        status: String,
        service: String,
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use axum::{Router, http::StatusCode, routing::get};
        use axum_test::TestServer;

        #[tokio::test]
        async fn test_health_check() {
            let app = Router::new().route("/health", get(handle_health_check));

            let server = TestServer::new(app).unwrap();
            let response = server.get("/health").await;

            response.assert_status(StatusCode::OK);
            let health: HealthResponse = response.json();
            assert_eq!(health.status, "ok");
            assert_eq!(health.service, "axum");
        }
    }
}

pub mod work {
    //! Work Handler module
    //!
    use axum::Json;
    use serde::{Deserialize, Serialize};
    use std::time::Duration;
    use tracing::{instrument, trace};

    /// Work response structure
    #[derive(Serialize, Deserialize)]
    pub struct WorkResponse {
        processed: bool,
        service: String,
    }

    /// Work endpoint
    ///
    /// Simulates work processing with a 20ms delay and returns a response.
    /// Accepts an optional query parameter 'message' for the work message.
    #[instrument(fields(service = "axum"))]
    pub async fn handle_work() -> Json<WorkResponse> {
        trace!("Work endpoint called");

        // Simulate work processing with 20ms sleep
        std::thread::sleep(Duration::from_millis(20));

        let response = WorkResponse {
            service: "axum".to_string(),
            processed: true,
        };

        Json(response)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use axum::{Router, http::StatusCode, routing::get};
        use axum_test::TestServer;

        #[tokio::test]
        async fn test_work_endpoint_with_message() {
            let app = Router::new().route("/work", get(handle_work));

            let server = TestServer::new(app).unwrap();
            let response = server.get("/work").await;

            response.assert_status(StatusCode::OK);
            let work: WorkResponse = response.json();
            assert!(work.processed);
            assert_eq!(work.service, "axum");
        }
    }
}
