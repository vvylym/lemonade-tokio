//! External implementation of MetricsService
//!
//! Fetches metrics from external sources (Prometheus/OpenTelemetry OTLP)

use crate::metrics::error::MetricsError;
use crate::metrics::port::MetricsService;
use crate::prelude::*;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Notify as TokioNotify;

/// External metrics service implementation
pub struct ExternalMetricsService {
    /// Metrics configuration (reference to global config's metrics slice)
    #[allow(dead_code)]
    config: Arc<ArcSwap<MetricsConfig>>,
    /// Shutdown notification
    shutdown: Arc<TokioNotify>,
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
        Ok(Self {
            config,
            shutdown: Arc::new(TokioNotify::new()),
        })
    }
}

#[async_trait]
impl MetricsService for ExternalMetricsService {
    async fn snapshot(&self) -> Result<MetricsSnapshot, MetricsError> {
        // TODO: Fetch from external sources (Prometheus/OTLP)
        Ok(MetricsSnapshot::default())
    }

    async fn start(&self, _ctx: Arc<Context>) -> Result<(), MetricsError> {
        // TODO: Implement periodic fetching from external sources
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), MetricsError> {
        self.shutdown.notify_waiters();
        Ok(())
    }
}
