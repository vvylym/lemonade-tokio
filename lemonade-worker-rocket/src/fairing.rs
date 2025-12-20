//! Tracing fairing for Rocket
//!
use opentelemetry::global;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Request, Response};
use std::collections::HashMap;

/// Helper to extract headers from Rocket request
struct RocketHeaderExtractor<'r> {
    headers: HashMap<String, String>,
    _phantom: std::marker::PhantomData<&'r ()>,
}

impl<'r> RocketHeaderExtractor<'r> {
    fn new(request: &'r Request<'_>) -> Self {
        let mut headers = HashMap::new();
        for header in request.headers().iter() {
            headers.insert(header.name().to_string(), header.value().to_string());
        }
        Self {
            headers,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'r> opentelemetry::propagation::Extractor for RocketHeaderExtractor<'r> {
    fn get(&self, key: &str) -> Option<&str> {
        self.headers.get(key).map(|s| s.as_str())
    }

    fn keys(&self) -> Vec<&str> {
        self.headers.keys().map(|s| s.as_str()).collect()
    }
}

/// Tracing fairing that creates spans for HTTP requests
pub struct TracingFairing;

#[rocket::async_trait]
impl Fairing for TracingFairing {
    fn info(&self) -> Info {
        Info {
            name: "Tracing Fairing",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut rocket::Data<'_>) {
        // Extract trace context from headers for distributed tracing
        let extractor = RocketHeaderExtractor::new(request);
        let _parent_cx = global::get_text_map_propagator(|prop| prop.extract(&extractor));

        // Create span with HTTP attributes
        let method = request.method().to_string();
        let uri = request.uri().to_string();
        let path = request.uri().path().to_string();

        let _span = tracing::span!(
            tracing::Level::INFO,
            "http_request",
            framework.name = "rocket",
            http.method = %method,
            http.route = %path,
            http.target = %uri,
        )
        .entered();
    }

    async fn on_response<'r>(
        &self,
        _request: &'r Request<'_>,
        response: &mut Response<'r>,
    ) {
        // Record status code in the span if it exists
        let status_code = response.status().code;
        tracing::Span::current().record("http.status_code", status_code);
    }
}
