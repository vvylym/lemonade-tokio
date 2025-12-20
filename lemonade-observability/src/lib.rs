//! Lemonade Observability Library
//!
//! Provides centralized observability initialization using OpenTelemetry SDK
//! with tracing integration for distributed tracing across load balancer and workers.

pub mod init;
pub mod metrics;
pub mod resource;

pub use init::{init_metrics, init_tracing};
pub use metrics::{HttpMetrics, get_http_metrics};
pub use resource::create_resource;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_resource() {
        let resource = create_resource("test-service", "1.0.0", "test-instance-1");
        // Resource should be created successfully
        assert!(!resource.is_empty());
    }

    #[test]
    fn test_create_resource_with_custom_values() {
        let resource = create_resource("custom-service", "2.0.0", "custom-instance-1");
        // Resource should be created successfully
        assert!(!resource.is_empty());
    }

    #[test]
    fn test_init_tracing_succeeds() {
        // This test verifies init_tracing can be called without panicking
        // Note: It can only be called once due to Once, so we test it doesn't panic
        let result = init_tracing("test-init", "1.0.0", "test-instance-1", None, None);
        assert!(result.is_ok());

        // Second call should also succeed (Once prevents re-initialization)
        let result2 = init_tracing("test-init-2", "1.0.0", "test-instance-2", None, None);
        assert!(result2.is_ok());
    }
}
