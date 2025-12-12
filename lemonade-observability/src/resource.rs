//! OpenTelemetry Resource Detection
//!
//! Creates OpenTelemetry Resource with service identification attributes

use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;

/// Create an OpenTelemetry resource with service identification
///
/// # Arguments
/// * `service_name` - The name of the service (e.g., "lemonade-load-balancer", "lemonade-worker-axum")
/// * `service_version` - Optional service version (defaults to "0.1.0" if not provided)
/// * `instance_id` - Optional instance identifier (defaults to service_name if not provided)
///
/// # Returns
/// OpenTelemetry Resource with standard attributes
pub fn create_resource(
    service_name: &str,
    service_version: Option<&str>,
    instance_id: Option<&str>,
) -> Resource {
    let version = service_version
        .map(|s| s.to_string())
        .or_else(|| std::env::var("OTEL_SERVICE_VERSION").ok())
        .unwrap_or_else(|| "0.1.0".to_string());

    let instance = instance_id
        .map(|s| s.to_string())
        .or_else(|| std::env::var("OTEL_SERVICE_INSTANCE_ID").ok())
        .unwrap_or_else(|| service_name.to_string());

    let environment = std::env::var("OTEL_DEPLOYMENT_ENVIRONMENT")
        .unwrap_or_else(|_| "development".to_string());

    // Use Resource::builder() which is the public API
    Resource::builder()
        .with_service_name(service_name.to_string())
        .with_attributes(vec![
            KeyValue::new("service.version", version),
            KeyValue::new("service.instance.id", instance),
            KeyValue::new("deployment.environment", environment),
        ])
        .build()
}
