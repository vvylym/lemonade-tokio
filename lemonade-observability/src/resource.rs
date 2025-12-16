//! OpenTelemetry Resource Detection
//!
//! Creates OpenTelemetry Resource with service identification attributes

use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;

/// Create an OpenTelemetry resource with service identification
///
/// # Arguments
/// * `service_name` - The name of the service (e.g., "lemonade-load-balancer", "lemonade-worker-axum")
/// * `service_version` - Service version (e.g., "1.0.0")
///
/// # Returns
/// OpenTelemetry Resource with standard attributes
pub fn create_resource(
    service_name: impl Into<String>,
    service_version: impl Into<String>,
) -> Resource {
    // Use Resource::builder() which is the public API
    Resource::builder()
        .with_service_name(service_name.into())
        .with_attributes(vec![KeyValue::new(
            "service.version",
            service_version.into(),
        )])
        .build()
}
