//! HTTP Metrics Collection
//!
//! Provides helpers for collecting HTTP request metrics using OpenTelemetry

use opentelemetry::KeyValue;
use opentelemetry::global;
use opentelemetry::metrics::{Counter, Histogram};
use std::sync::Arc;

/// HTTP metrics for a service
pub struct HttpMetrics {
    /// Counter for total HTTP requests
    pub requests_total: Counter<u64>,
    /// Histogram for HTTP request duration in seconds
    pub request_duration_seconds: Histogram<f64>,
}

impl HttpMetrics {
    /// Create HTTP metrics for a service
    ///
    /// # Arguments
    /// * `service_name` - The name of the service (e.g., "lemonade-worker-actix")
    ///
    /// # Returns
    /// * `Self` with initialized metrics instruments
    pub fn new(service_name: &'static str) -> Self {
        let meter = global::meter(service_name);

        let requests_total = meter
            .u64_counter("lemonade_http_requests_total")
            .with_description("Total number of HTTP requests")
            .build();

        let request_duration_seconds = meter
            .f64_histogram("lemonade_http_request_duration_seconds")
            .with_description("HTTP request duration in seconds")
            .build();

        Self {
            requests_total,
            request_duration_seconds,
        }
    }

    /// Record an HTTP request
    ///
    /// # Arguments
    /// * `method` - HTTP method (e.g., "GET", "POST")
    /// * `route` - HTTP route (e.g., "/health", "/work")
    /// * `status_code` - HTTP status code (e.g., 200, 404, 500)
    /// * `duration_micros` - Request duration in microseconds (converted to seconds for histogram)
    pub fn record_request(
        &self,
        method: &str,
        route: &str,
        status_code: u16,
        duration_micros: u64,
    ) {
        let attributes = vec![
            KeyValue::new("http.method", method.to_string()),
            KeyValue::new("http.route", route.to_string()),
            KeyValue::new("http.status_code", status_code as i64),
        ];

        self.requests_total.add(1, &attributes);
        // Convert microseconds to seconds only when recording (OpenTelemetry uses seconds)
        let duration_seconds = duration_micros as f64 / 1_000_000.0;
        self.request_duration_seconds
            .record(duration_seconds, &attributes);
    }
}

/// Get or create HTTP metrics for a service (thread-safe, supports multiple services)
pub fn get_http_metrics(service_name: &str) -> Arc<HttpMetrics> {
    use dashmap::DashMap;
    use std::sync::OnceLock;

    static METRICS_MAP: OnceLock<DashMap<String, Arc<HttpMetrics>>> = OnceLock::new();

    let map = METRICS_MAP.get_or_init(DashMap::new);

    map.entry(service_name.to_string())
        .or_insert_with(|| {
            // Convert service_name to static string for global::meter
            // This leaks memory but is acceptable for long-lived service names
            let static_name: &'static str =
                Box::leak(service_name.to_string().into_boxed_str());
            Arc::new(HttpMetrics::new(static_name))
        })
        .clone()
}
