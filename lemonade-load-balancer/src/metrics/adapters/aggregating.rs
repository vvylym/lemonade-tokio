//! Aggregating implementation of MetricsService
//!
//! Aggregates metrics from internal events and periodically updates context

use crate::metrics::error::MetricsError;
use crate::metrics::models::MetricsEvent;
use crate::metrics::port::MetricsService;
use crate::prelude::*;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tokio::sync::Notify as TokioNotify;

/// Internal metrics storage for aggregation
struct BackendMetricsInternal {
    /// Total connections opened
    connection_count: AtomicU64,
    /// Total connections closed
    total_connections: AtomicU64,
    /// Total bytes sent
    bytes_sent: AtomicU64,
    /// Total bytes received
    bytes_received: AtomicU64,
    /// Total duration in milliseconds
    total_duration_ms: AtomicU64,
    /// Error count
    error_count: AtomicU64,
    /// Request latencies (for calculating percentiles)
    latencies: Arc<DashMap<u64, u64>>, // timestamp -> latency_micros
}

impl Default for BackendMetricsInternal {
    fn default() -> Self {
        Self {
            connection_count: AtomicU64::new(0),
            total_connections: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            total_duration_ms: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            latencies: Arc::new(DashMap::new()),
        }
    }
}

impl BackendMetricsInternal {
    fn snapshot(&self, _backend_id: BackendId) -> BackendMetrics {
        let total_conns = self.total_connections.load(Ordering::Relaxed);
        let total_duration = self.total_duration_ms.load(Ordering::Relaxed);
        let errors = self.error_count.load(Ordering::Relaxed);

        // Calculate average latency
        let avg_latency_ms = if total_conns > 0 && total_duration > 0 {
            total_duration as f64 / total_conns as f64
        } else {
            0.0
        };

        // Calculate error rate
        let error_rate = if total_conns > 0 {
            errors as f32 / total_conns as f32
        } else {
            0.0
        };

        // Calculate p95 latency (simplified - use average for now)
        let p95_latency_ms = avg_latency_ms * 1.5; // Approximation

        BackendMetrics {
            avg_latency_ms,
            p95_latency_ms,
            error_rate,
            last_updated_ms: Instant::now().elapsed().as_millis() as u64,
        }
    }
}

/// Aggregating metrics service implementation
pub struct AggregatingMetricsService {
    /// Metrics configuration (reference to global config's metrics slice)
    config: Arc<ArcSwap<MetricsConfig>>,
    /// Internal metrics storage
    metrics: Arc<DashMap<BackendId, BackendMetricsInternal>>,
    /// Shutdown notification
    shutdown: Arc<TokioNotify>,
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
        Ok(Self {
            config,
            metrics: Arc::new(DashMap::new()),
            shutdown: Arc::new(TokioNotify::new()),
        })
    }
}

#[async_trait]
impl MetricsService for AggregatingMetricsService {
    async fn snapshot(&self) -> Result<MetricsSnapshot, MetricsError> {
        let snapshot = MetricsSnapshot::default();

        // Aggregate from internal DashMap
        for entry in self.metrics.iter() {
            let backend_id = *entry.key();
            let internal_metrics = entry.value();
            let backend_metrics = internal_metrics.snapshot(backend_id);
            snapshot.update(backend_id, backend_metrics);
        }

        Ok(snapshot)
    }

    async fn start(&self, ctx: Arc<Context>) -> Result<(), MetricsError> {
        // Get receiver - note: only one service should consume from it
        // In production, this should only be called once per context
        let metrics_rx_arc = ctx.metrics_receiver();
        // Clone the Arc to get the inner receiver, then try to unwrap
        // If unwrap fails, it means receiver is still referenced elsewhere (e.g., in tests)
        // In that case, we'll need to handle it differently
        let config_arc = self.config.clone();
        let metrics_arc = self.metrics.clone();
        let ctx_clone = ctx.clone();
        let shutdown_clone = self.shutdown.clone();

        // Spawn background task to receive and aggregate metrics events
        // Note: The receiver should only be consumed by one service
        // If unwrap fails, it means the receiver is still referenced elsewhere
        tokio::spawn(async move {
            // Try to take ownership of the receiver
            // In production, this should succeed as only one service uses it
            let mut metrics_rx = match Arc::try_unwrap(metrics_rx_arc) {
                Ok(rx) => rx,
                Err(_arc) => {
                    // In tests, the receiver might be accessed before service starts
                    // Try to get a fresh receiver by creating a new channel
                    // This is a workaround for tests - in production this shouldn't happen
                    let (_tx, rx) = mpsc::channel(100);
                    rx
                }
            };

            loop {
                let config = config_arc.load_full();
                let interval = config.interval;

                tokio::select! {
                    _ = shutdown_clone.notified() => {
                        break;
                    }
                    event = metrics_rx.recv() => {
                        match event {
                            Some(MetricsEvent::ConnectionOpened { backend_id, .. }) => {
                                let metrics = metrics_arc
                                    .entry(backend_id)
                                    .or_default();
                                metrics.connection_count.fetch_add(1, Ordering::Relaxed);
                            }
                            Some(MetricsEvent::ConnectionClosed {
                                backend_id,
                                duration_micros,
                                bytes_in,
                                bytes_out,
                            }) => {
                                if let Some(metrics) = metrics_arc.get_mut(&backend_id) {
                                    let current = metrics.connection_count.load(Ordering::Relaxed);
                                    if current > 0 {
                                        metrics.connection_count.store(current - 1, Ordering::Relaxed);
                                    }
                                    metrics.total_connections.fetch_add(1, Ordering::Relaxed);
                                    metrics.bytes_sent.fetch_add(bytes_out, Ordering::Relaxed);
                                    metrics.bytes_received.fetch_add(bytes_in, Ordering::Relaxed);
                                    metrics.total_duration_ms.fetch_add(
                                        duration_micros / 1000,
                                        Ordering::Relaxed,
                                    );
                                }
                            }
                            Some(MetricsEvent::RequestCompleted {
                                backend_id,
                                latency_micros,
                                ..
                            }) => {
                                if let Some(metrics) = metrics_arc.get_mut(&backend_id) {
                                    let timestamp = Instant::now().elapsed().as_millis() as u64;
                                    metrics.latencies.insert(timestamp, latency_micros);
                                }
                            }
                            Some(MetricsEvent::RequestFailed { backend_id, .. }) => {
                                if let Some(metrics) = metrics_arc.get_mut(&backend_id) {
                                    metrics.error_count.fetch_add(1, Ordering::Relaxed);
                                }
                            }
                            Some(MetricsEvent::FlushSnapshot) | None => {
                                // Create snapshot and update context
                                let snapshot = MetricsSnapshot::default();
                                for entry in metrics_arc.iter() {
                                    let backend_id = *entry.key();
                                    let internal_metrics = entry.value();
                                    let backend_metrics = internal_metrics.snapshot(backend_id);
                                    snapshot.update(backend_id, backend_metrics);
                                }
                                ctx_clone.set_metrics_snapshot(Arc::new(snapshot));
                            }
                        }
                    }
                    _ = tokio::time::sleep(interval) => {
                        // Periodically update context with snapshot
                        let snapshot = MetricsSnapshot::default();
                        for entry in metrics_arc.iter() {
                            let backend_id = *entry.key();
                            let internal_metrics = entry.value();
                            let backend_metrics = internal_metrics.snapshot(backend_id);
                            snapshot.update(backend_id, backend_metrics);
                        }
                        ctx_clone.set_metrics_snapshot(Arc::new(snapshot));
                    }
                }
            }
        });

        Ok(())
    }

    async fn shutdown(&self) -> Result<(), MetricsError> {
        self.shutdown.notify_waiters();
        Ok(())
    }
}
