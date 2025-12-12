//! Lemonade Observability Library
//!
//! Provides centralized observability initialization using OpenTelemetry SDK
//! with tracing integration for distributed tracing across load balancer and workers.

pub mod init;
pub mod resource;

pub use init::init_tracing;
pub use resource::create_resource;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_resource_with_defaults() {
        let resource = create_resource("test-service", None, None);
        // Resource should be created successfully
        assert!(!resource.is_empty());
    }

    #[test]
    fn test_create_resource_with_custom_values() {
        let resource =
            create_resource("custom-service", Some("1.0.0"), Some("instance-1"));
        // Resource should be created successfully
        assert!(!resource.is_empty());
    }

    #[test]
    fn test_init_tracing_succeeds() {
        // This test verifies init_tracing can be called without panicking
        // Note: It can only be called once due to Once, so we test it doesn't panic
        let result = init_tracing("test-init");
        assert!(result.is_ok());

        // Second call should also succeed (Once prevents re-initialization)
        let result2 = init_tracing("test-init-2");
        assert!(result2.is_ok());
    }
}
