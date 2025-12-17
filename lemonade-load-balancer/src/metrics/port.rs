//! Metrics Port module
//!
use crate::prelude::*;

/// Metrics service trait
#[async_trait]
pub trait MetricsService: Send + Sync + 'static {
    /// Collect and aggregate metrics (loops until shutdown)
    async fn collect_metrics(&self, ctx: Arc<Context>);
}

#[cfg(test)]
mockall::mock! {
    pub MockMetricsServiceSuccess {}

    #[async_trait]
    impl MetricsService for MockMetricsServiceSuccess {
        async fn collect_metrics(&self, _ctx: Arc<Context>) {}
    }
}
