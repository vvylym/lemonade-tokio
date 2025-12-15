//! Health Port module
//!
use crate::prelude::*;

/// Health service trait
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait HealthService: Send + Sync + 'static {
    /// Check backend health periodically (loops until shutdown)
    /// - Performs periodic health checks via TCP connection
    /// - Listens for BackendFailureEvent from proxy for immediate alerting
    /// - Respects backend load (skips health checks on busy backends)
    async fn check_health(&self, ctx: Arc<Context>);
}

#[cfg(test)]
mockall::mock! {
    pub MockHealthServiceSuccess {}

    #[async_trait]
    impl HealthService for MockHealthServiceSuccess {
        async fn check_health(&self, _ctx: Arc<Context>) {}
    }
}
