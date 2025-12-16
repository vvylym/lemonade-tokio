//! Lemonade Load Balancer Library
//!

pub(crate) mod app;
pub(crate) mod config;
pub(crate) mod health;
pub(crate) mod metrics;
pub(crate) mod proxy;
pub(crate) mod strategy;
pub(crate) mod types;

#[macro_use]
mod helpers;

pub mod error;
pub mod prelude;
pub use app::App;

use prelude::*;
use std::{path::PathBuf, sync::Arc};

/// Run the load balancer with the given configuration
///
/// # Arguments
/// * `config_file` - Optional path to config file for hot-reloading
///
/// # Returns
/// * `Ok(())` if the load balancer ran successfully
/// * `Err(Box<dyn std::error::Error>)` if there was an error
#[tracing::instrument(skip_all, fields(service.name = "lemonade-load-balancer", config.file = ?config_file))]
pub async fn run(config_file: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    // Create static config service (uses provided config, no file watching)
    let config_service: Arc<dyn ConfigService> =
        Arc::new(NotifyConfigService::new(config_file)?);

    let config = config_service.snapshot();

    // Extract service-specific configs and wrap in ArcSwap
    let health_config = Arc::new(ArcSwap::from_pointee(config.health.clone()));
    let metrics_config = Arc::new(ArcSwap::from_pointee(config.metrics.clone()));
    let proxy_config = Arc::new(ArcSwap::from_pointee(config.proxy.clone()));

    // Create services using the adapters
    let health_service: Arc<dyn HealthService> =
        Arc::new(BackendHealthService::new(health_config)?);
    let metrics_service: Arc<dyn MetricsService> =
        Arc::new(AggregatingMetricsService::new(metrics_config)?);
    let proxy_service: Arc<dyn ProxyService> =
        Arc::new(TokioProxyService::new(proxy_config)?);

    // Create and run the app
    let app = App::new(
        config_service,
        health_service,
        metrics_service,
        proxy_service,
    )
    .await;
    app.run().await?;

    Ok(())
}
