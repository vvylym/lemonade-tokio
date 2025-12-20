//! Aggregating implementation of MetricsService
//!
//! Aggregates metrics from events and updates backend state directly

use crate::metrics::error::MetricsError;
use crate::metrics::models::MetricsEvent;
use crate::metrics::port::MetricsService;
use crate::prelude::*;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Aggregating metrics service implementation
pub struct AggregatingMetricsService {
    /// Metrics configuration (reference to global config's metrics slice)
    config: Arc<ArcSwap<MetricsConfig>>,
}

impl AggregatingMetricsService {
    /// Create a new AggregatingMetricsService
    ///
    /// # Arguments
    /// * `config` - Arc<ArcSwap<MetricsConfig>> reference to metrics config
    ///
    /// # Returns
    /// * `Ok(Self)` if service was created successfully
    pub fn new(config: Arc<ArcSwap<MetricsConfig>>) -> Result<Self, MetricsError> {
        Ok(Self { config })
    }

    /// Get current time in milliseconds
    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

#[async_trait]
impl MetricsService for AggregatingMetricsService {
    #[tracing::instrument(skip(self, ctx), fields(service.name = "lemonade-load-balancer", service.type = "metrics"))]
    async fn collect_metrics(&self, ctx: Arc<Context>) {
        tracing::info!("Starting metrics service");

        // Get receiver for metrics events
        let mut metrics_rx = ctx
            .channels()
            .metrics_rx()
            .expect("Metrics receiver already taken");
        let mut shutdown_rx = ctx.channels().shutdown_rx();

        // Get initial config
        let initial_config = self.config.load();
        let mut interval = tokio::time::interval(initial_config.interval);

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    tracing::info!("Metrics service received shutdown signal");
                    break;
                }

                event = metrics_rx.recv() => {
                    match event {
                        Some(MetricsEvent::ConnectionOpened { .. }) => {
                            // Connection opened - no action needed, connection count tracked in backend
                        }
                        Some(MetricsEvent::ConnectionClosed {
                            backend_id,
                            duration_micros,
                            ..
                        }) => {
                            // Record connection metrics
                            let routing = ctx.routing_table();
                            if let Some(backend) = routing.get(backend_id) {
                                // Record as a request (connection duration as latency)
                                let latency_ms = duration_micros / 1000;
                                backend.record_request(latency_ms, false);

                                // Export to OpenTelemetry (each connection = one request from client perspective)
                                let metrics = lemonade_observability::get_http_metrics("lemonade-load-balancer");
                                metrics.record_request("PROXY", "/", 200, duration_micros);
                            }
                        }
                        Some(MetricsEvent::RequestCompleted {
                            backend_id,
                            latency_micros,
                            status_code,
                        }) => {
                            // Record successful request
                            let routing = ctx.routing_table();
                            if let Some(backend) = routing.get(backend_id) {
                                let latency_ms = latency_micros / 1000;
                                backend.record_request(latency_ms, false);

                                // Export to OpenTelemetry
                                let metrics = lemonade_observability::get_http_metrics("lemonade-load-balancer");
                                metrics.record_request("PROXY", "/", status_code, latency_micros);
                            }
                        }
                        Some(MetricsEvent::RequestFailed {
                            backend_id,
                            latency_micros,
                            ..
                        }) => {
                            // Record failed request
                            let routing = ctx.routing_table();
                            if let Some(backend) = routing.get(backend_id) {
                                let latency_ms = latency_micros / 1000;
                                backend.record_request(latency_ms, true);

                                // Export to OpenTelemetry (failed request)
                                let metrics = lemonade_observability::get_http_metrics("lemonade-load-balancer");
                                metrics.record_request("PROXY", "/", 500, latency_micros);
                            }
                        }
                        Some(MetricsEvent::FlushSnapshot) | None => {
                            // Update metrics timestamps for all backends
                            let routing = ctx.routing_table();
                            let now_ms = Self::now_ms();
                            for backend in routing.all_backends() {
                                backend.update_metrics_timestamp(now_ms);
                            }
                        }
                    }
                }

                _ = interval.tick() => {
                    // Periodically update metrics timestamps
                    let routing = ctx.routing_table();
                    let now_ms = Self::now_ms();
                    for backend in routing.all_backends() {
                        backend.update_metrics_timestamp(now_ms);
                    }
                    tracing::debug!("Metrics timestamps updated");
                }
            }
        }
        tracing::info!("Metrics service stopped");
    }
}
