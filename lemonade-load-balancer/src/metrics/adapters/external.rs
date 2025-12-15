//! External implementation of MetricsService
//!
//! Fetches metrics from external sources (Prometheus/OpenTelemetry OTLP)

use crate::metrics::error::MetricsError;
use crate::metrics::port::MetricsService;
use crate::prelude::*;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use std::sync::Arc;

/// External metrics service implementation
pub struct ExternalMetricsService {
    /// Metrics configuration (reference to global config's metrics slice)
    #[allow(dead_code)]
    config: Arc<ArcSwap<MetricsConfig>>,
}

impl ExternalMetricsService {
    /// Create a new ExternalMetricsService
    ///
    /// # Arguments
    /// * `config` - Arc<ArcSwap<MetricsConfig>> reference to metrics config
    ///
    /// # Returns
    /// * `Ok(Self)` if service was created successfully
    pub fn new(config: Arc<ArcSwap<MetricsConfig>>) -> Result<Self, MetricsError> {
        Ok(Self { config })
    }
}

#[async_trait]
impl MetricsService for ExternalMetricsService {
    async fn collect_metrics(&self, ctx: Arc<Context>) {
        // TODO: Implement periodic fetching from external sources (Prometheus/OTLP)
        // For now, just wait for shutdown
        let mut shutdown_rx = ctx.channels().shutdown_rx();
        let _ = shutdown_rx.recv().await;
    }
}
