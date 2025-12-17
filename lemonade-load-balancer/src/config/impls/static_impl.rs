//! Static implementation of ConfigService
//!
//! No-op implementation for environment variable configs (no file watching needed)

use crate::config::port::ConfigService;
use crate::prelude::*;
use async_trait::async_trait;

/// Static config service implementation (no-op for env-var configs)
pub struct StaticConfigService;

impl StaticConfigService {
    /// Create a new StaticConfigService
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ConfigService for StaticConfigService {
    async fn watch_config(&self, ctx: Arc<Context>) {
        // For environment variable configs, there's nothing to watch
        // Just wait for shutdown signal
        let mut shutdown_rx = ctx.channels().shutdown_rx();
        let _ = shutdown_rx.recv().await;
    }
}

impl Default for StaticConfigService {
    fn default() -> Self {
        Self::new()
    }
}
