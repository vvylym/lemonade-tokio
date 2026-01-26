//! Lemonade Load Balancer Library
//!
//! ## Ports and adapters
//!
//! The crate uses a ports-and-adapters (hexagonal) layout. **Ports** are traits defined in
//! `*/port.rs`: `ConfigService`, `HealthService`, `MetricsService`, `ProxyService`, and
//! `StrategyService`. **Adapters** are concrete implementations under `*/impls/` (config) or
//! `*/adapters/` (health, metrics, proxy, strategy). The [`App`] composes these via a shared
//! [`Context`](crate::types::context::Context), which holds config, route table, strategy, and
//! channels used by all services.

pub(crate) mod app;
pub(crate) mod config;
pub(crate) mod health;
pub(crate) mod metrics;
pub(crate) mod proxy;
pub(crate) mod strategy;
pub(crate) mod types;

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
/// * `Err(crate::error::Error)` if there was an error
#[tracing::instrument(skip_all, fields(service.name = "lemonade-load-balancer", config.file = ?config_file))]
pub async fn run(config_file: Option<PathBuf>) -> crate::error::Result<()> {
    // Load config
    let config = ConfigBuilder::from_file(config_file.as_deref())?;

    // Initialize tracing with load balancer service name and package version
    // OTLP config comes from environment variables (OTEL_EXPORTER_OTLP_ENDPOINT, OTEL_EXPORTER_OTLP_PROTOCOL)
    lemonade_observability::init_tracing(
        "lemonade-load-balancer",
        env!("CARGO_PKG_VERSION"),
        "lemonade-load-balancer",
        config.otlp_endpoint.as_deref(),
        config.otlp_protocol.as_deref(),
    )
    .map_err(|e| crate::error::Error::Other(e.to_string()))?;

    // Initialize metrics
    lemonade_observability::init_metrics(
        "lemonade-load-balancer",
        env!("CARGO_PKG_VERSION"),
        "lemonade-load-balancer",
        config.otlp_endpoint.as_deref(),
        config.otlp_protocol.as_deref(),
    )
    .map_err(|e| crate::error::Error::Other(e.to_string()))?;

    // Create context from config (context-first initialization)
    let ctx = Arc::new(Context::new(config.clone())?);

    // Create services (they don't need initial config, they get it from context)
    let config_service: Arc<dyn ConfigService> = if config.source == ConfigSource::File {
        Arc::new(NotifyConfigService::new(config_file)?)
    } else {
        Arc::new(StaticConfigService::new())
    };

    let health_config = Arc::new(ArcSwap::from_pointee(config.health.clone()));
    let health_service: Arc<dyn HealthService> =
        Arc::new(BackendHealthService::new(health_config)?);

    let metrics_config = Arc::new(ArcSwap::from_pointee(config.metrics.clone()));
    let metrics_service: Arc<dyn MetricsService> =
        Arc::new(AggregatingMetricsService::new(metrics_config)?);

    let proxy_config = Arc::new(ArcSwap::from_pointee(config.proxy.clone()));
    let proxy_service: Arc<dyn ProxyService> =
        Arc::new(TokioProxyService::new(proxy_config)?);

    // Create and run app
    let app = App::new(
        config_service,
        health_service,
        metrics_service,
        proxy_service,
    )
    .await;
    app.run(ctx).await?;

    Ok(())
}
