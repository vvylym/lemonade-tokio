//! App module
//!

use crate::error::Result;
use crate::prelude::*;

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
    #[tracing::instrument(skip(self, ctx), fields(service.name = "lemonade-load-balancer"))]
    pub async fn run(&self, ctx: Arc<Context>) -> Result<()> {
        tracing::info!("Starting load balancer");

        // Spawn background service tasks
        let config_handle = tokio::spawn({
            let ctx = ctx.clone();
            let svc = self.config_service.clone();
            async move {
                svc.watch_config(ctx).await;
            }
        });

        let health_handle = tokio::spawn({
            let ctx = ctx.clone();
            let svc = self.health_service.clone();
            async move {
                svc.check_health(ctx).await;
            }
        });

        let metrics_handle = tokio::spawn({
            let ctx = ctx.clone();
            let svc = self.metrics_service.clone();
            async move {
                svc.collect_metrics(ctx).await;
            }
        });

        // Spawn Ctrl-C handler
        let shutdown_tx = ctx.channels().shutdown_tx();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            tracing::info!("Shutdown signal received");
            let _ = shutdown_tx.send(());
        });

        // PROXY RUNS ON MAIN THREAD (HOT PATH)
        // This is critical for performance - no extra task overhead
        tracing::info!("Starting proxy service on main thread");
        let proxy_result = self.proxy_service.accept_connections(ctx.clone()).await;

        // If proxy exits (shutdown or error), wait for background services
        tracing::info!("Proxy service stopped, waiting for background services");
        let cfg = ctx.config();
        let timeout_ms = cfg.runtime.background_timeout_millis;
        let _ = tokio::time::timeout(Duration::from_millis(timeout_ms), async {
            let _ = tokio::join!(config_handle, health_handle, metrics_handle);
        })
        .await;

        // Drain remaining connections
        let drain_ms = cfg.runtime.drain_timeout_millis;
        ctx.wait_for_drain(Duration::from_millis(drain_ms)).await?;

        tracing::info!("Shutdown complete");
        proxy_result.map_err(crate::error::Error::Proxy)
    }
}
