//! Config Port module
//!
use crate::prelude::*;

/// Config service trait
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ConfigService: Send + Sync + 'static {
    /// Watch for config changes (loops until shutdown)
    async fn watch_config(&self, ctx: Arc<Context>);
}

#[cfg(test)]
mockall::mock! {
    pub MockConfigServiceSuccess {}

    #[async_trait]
    impl ConfigService for MockConfigServiceSuccess {
        async fn watch_config(&self, _ctx: Arc<Context>) {}
    }
}
