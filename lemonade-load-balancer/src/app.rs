//! App module
//!
use crate::error::Result;
use crate::prelude::*;
use crate::{
    drain_and_run_handles_with_timeout, spawn_background_handle, wait_for_shutdown,
};

/// App struct
pub struct App {
    /// Config service
    config_service: Arc<dyn ConfigService>,
    /// Health service
    health_service: Arc<dyn HealthService>,
    /// Metrics service
    metrics_service: Arc<dyn MetricsService>,
    /// Proxy service
    proxy_service: Arc<dyn ProxyService>,
}

impl App {
    /// Create a new app
    pub async fn new(
        config_service: Arc<dyn ConfigService>,
        health_service: Arc<dyn HealthService>,
        metrics_service: Arc<dyn MetricsService>,
        proxy_service: Arc<dyn ProxyService>,
    ) -> Self {
        Self {
            config_service,
            health_service,
            metrics_service,
            proxy_service,
        }
    }

    /// Run the app
    pub async fn run(&self) -> Result<()> {
        // create state
        let ctx = Arc::new(Context::new(&self.config_service.snapshot())?);

        // Config handle
        let config_handle = spawn_background_handle!(self.config_service, &ctx);
        // Health handle
        let health_handle = spawn_background_handle!(self.health_service, &ctx);
        // Metrics handle
        let metrics_handle = spawn_background_handle!(self.metrics_service, &ctx);

        // Proxy handle (entirely owned by proxy service)
        let accept_handle = self.proxy_service.accept_connections(&ctx);

        // create runtime context
        let shutdown_tx = ctx.shutdown_sender();
        // ctrl-c triggers graceful shutdown
        wait_for_shutdown!(shutdown_tx);

        // drain and run handles with timeout
        drain_and_run_handles_with_timeout!(
            ctx,
            config_handle,
            health_handle,
            metrics_handle,
            accept_handle
        );

        // return success
        Ok(())
    }
}
