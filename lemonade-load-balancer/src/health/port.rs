//! Health Port module
//!
use crate::prelude::*;

/// Health service trait
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait HealthService: Send + Sync + 'static {
    /// Start the health service
    async fn start(&self, ctx: Arc<Context>) -> Result<(), HealthError>;

    /// Shutdown the health service
    async fn shutdown(&self) -> Result<(), HealthError>;
}

#[cfg(test)]
mockall::mock! {
    pub MockHealthServiceSuccess {}

    #[async_trait]
    impl HealthService for MockHealthServiceSuccess {
        async fn start(&self, _ctx: Arc<Context>) -> Result<(), HealthError> {
            Ok(())
        }
        async fn shutdown(&self) -> Result<(), HealthError> {
            Ok(())
        }
    }
}
