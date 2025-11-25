//! Handlers module for the Axum Worker
//!
pub mod health {
    //! Health Handler module
    //!
    use actix_web::{HttpResponse, Responder};
    use serde::{Deserialize, Serialize};
    use tracing::{instrument, trace};

    /// Health check endpoint
    ///
    /// Returns a JSON response indicating the service is healthy.
    #[instrument(fields(service = "actix-web"))]
    pub async fn handle_health_check() -> impl Responder {
        trace!("Health check endpoint called");

        let response = HealthResponse {
            status: "ok".to_string(),
            service: "actix-web".to_string(),
        };
        HttpResponse::Ok().json(response)
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
        use actix_web::{
            App,
            test::{TestRequest, call_service, init_service, read_body_json},
            web,
        };

        #[actix_web::test]
        async fn test_health_check() {
            let app = init_service(App::new().route("/", web::get().to(handle_health_check))).await;

            let req = TestRequest::get().uri("/").to_request();
            let resp = call_service(&app, req).await;

            assert!(resp.status().is_success());

            let body: HealthResponse = read_body_json(resp).await;
            assert_eq!(body.status, "ok");
            assert_eq!(body.service, "actix-web");
        }
    }
}

pub mod work {
    //! Work Handler module
    //!
    use actix_web::{HttpResponse, Responder};
    use serde::{Deserialize, Serialize};
    use std::{thread::sleep, time::Duration};
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
    #[instrument(fields(service = "actix-web"))]
    pub async fn handle_work() -> impl Responder {
        trace!("Work endpoint called");

        // Simulate work processing with 20ms sleep
        sleep(Duration::from_millis(20));

        let response = WorkResponse {
            processed: true,
            service: "actix-web".to_string(),
        };
        HttpResponse::Ok().json(response)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use actix_web::{
            App,
            test::{TestRequest, call_service, init_service, read_body_json},
            web,
        };

        #[actix_web::test]
        async fn test_handle_work() {
            let app = init_service(App::new().route("/", web::get().to(handle_work))).await;

            let req = TestRequest::get().uri("/").to_request();
            let resp = call_service(&app, req).await;

            assert!(resp.status().is_success());

            let body: WorkResponse = read_body_json(resp).await;
            assert!(body.processed);
            assert_eq!(body.service, "actix-web");
        }
    }
}
