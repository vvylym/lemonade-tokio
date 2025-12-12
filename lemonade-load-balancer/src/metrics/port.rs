//! Metrics Port module
//!
use crate::prelude::*;

/// Metrics service trait
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MetricsService: Send + Sync + 'static {
    /// Snapshot the metrics
    async fn snapshot(&self) -> Result<MetricsSnapshot, MetricsError>;

    /// Start the metrics service
    async fn start(&self, ctx: Arc<Context>) -> Result<(), MetricsError>;

    /// Shutdown the metrics service
    async fn shutdown(&self) -> Result<(), MetricsError>;
}

#[cfg(test)]
mockall::mock! {
    pub MockMetricsServiceSuccess {}

    #[async_trait]
    impl MetricsService for MockMetricsServiceSuccess {
        async fn snapshot(&self) -> Result<MetricsSnapshot, MetricsError> {
            Ok(MetricsSnapshot::default())
        }
        async fn start(&self, _ctx: Arc<Context>) -> Result<(), MetricsError> {
            Ok(())
        }
        async fn shutdown(&self) -> Result<(), MetricsError> {
            Ok(())
        }
    }
}
