//! Backend implementation of HealthService
//!
//! Performs periodic health checks on backends using TCP connections
//! and listens for immediate failure alerts from proxy

use crate::health::error::HealthError;
use crate::health::models::{
    BackendFailureEvent, HealthEvent, HealthFailureReason, HealthStatus,
};
use crate::health::port::HealthService;
use crate::prelude::*;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Backend health service implementation
pub struct BackendHealthService {
    /// Health configuration (reference to global config's health slice)
    config: Arc<ArcSwap<HealthConfig>>,
}

impl BackendHealthService {
    /// Create a new BackendHealthService
    ///
    /// # Arguments
    /// * `config` - Arc<ArcSwap<HealthConfig>> reference to health config
    ///
    /// # Returns
    /// * `Ok(Self)` if service was created successfully
    pub fn new(config: Arc<ArcSwap<HealthConfig>>) -> Result<Self, HealthError> {
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
impl HealthService for BackendHealthService {
    #[tracing::instrument(skip(self, ctx), fields(service.name = "lemonade-load-balancer", service.type = "health"))]
    async fn check_health(&self, ctx: Arc<Context>) {
        tracing::info!("Starting health service");

        // Get receivers for failure events and shutdown
        let mut backend_failure_rx = ctx
            .channels()
            .backend_failure_rx()
            .expect("Backend failure receiver already taken");
        let mut shutdown_rx = ctx.channels().shutdown_rx();
        let health_tx = ctx.channels().health_tx();

        // Get initial config
        let initial_config = self.config.load();
        let mut interval = tokio::time::interval(initial_config.interval);

        let backend_count = ctx.routing_table().len();
        tracing::debug!("Health service will monitor {} backends", backend_count);

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    tracing::info!("Health service received shutdown signal");
                    break;
                }

                // IMMEDIATE: Proxy detected backend failure
                Some(failure) = backend_failure_rx.recv() => {
                    let routing = ctx.routing_table();
                    let backend_id = match &failure {
                        BackendFailureEvent::ConnectionRefused { backend_id } => *backend_id,
                        BackendFailureEvent::Timeout { backend_id } => *backend_id,
                        BackendFailureEvent::BackendClosed { backend_id } => *backend_id,
                        BackendFailureEvent::ConsecutiveErrors { backend_id, .. } => *backend_id,
                    };

                    if let Some(backend) = routing.get(backend_id) {
                        let was_alive = backend.is_alive();
                        let now_ms = Self::now_ms();

                        tracing::warn!(
                            "Backend {} marked unhealthy due to proxy failure: {:?}",
                            backend_id,
                            failure
                        );

                        backend.set_health(false, now_ms);

                        // Send health event for observability
                        let _ = health_tx.send(HealthEvent::BackendUnhealthy {
                            backend_id,
                            reason: match &failure {
                                BackendFailureEvent::ConnectionRefused { .. } => HealthFailureReason::ConnectionRefused,
                                BackendFailureEvent::Timeout { .. } => HealthFailureReason::Timeout,
                                _ => HealthFailureReason::Transport,
                            },
                        }).await;

                        // Send transition event if state changed
                        if was_alive {
                            tracing::info!("Backend {} health transition: healthy -> unhealthy", backend_id);
                            let _ = health_tx.send(HealthEvent::HealthTransition {
                                backend_id,
                                from: HealthStatus::Healthy,
                                to: HealthStatus::Unhealthy,
                            }).await;
                        }
                    }
                }

                // PERIODIC: Proactive health checks
                _ = interval.tick() => {
                    let routing = ctx.routing_table();
                    let config = self.config.load();
                    let health_tx = health_tx.clone();

                    tracing::debug!("Starting health check cycle for {} backends", routing.len());

                    for backend in routing.all_backends() {
                        let backend_id = backend.id();
                        let address = backend.address();

                        // Skip if backend has high load (respect backend capacity)
                        // Use a reasonable threshold (e.g., 100 connections) to avoid overloading
                        if !backend.has_capacity_for_health_check(100) {
                            tracing::debug!(
                                "Skipping health check for busy backend {} ({} active connections)",
                                backend_id,
                                backend.active_connections()
                            );
                            continue;
                        }

                        let check_span = tracing::debug_span!(
                            "health.check",
                            service.name = "lemonade-load-balancer",
                            backend.id = %backend_id,
                            backend.addr = %address
                        );
                        let _check_guard = check_span.enter();

                        // Perform TCP health check
                        let check_start = std::time::Instant::now();
                        let is_healthy = match tokio::time::timeout(
                            config.timeout,
                            tokio::net::TcpStream::connect(address),
                        )
                        .await
                        {
                            Ok(Ok(_)) => {
                                let rtt_micros = check_start.elapsed().as_micros() as u64;
                                tracing::debug!("Backend {} is healthy (RTT: {}μs)", backend_id, rtt_micros);
                                let _ = health_tx.send(HealthEvent::BackendHealthy {
                                    backend_id,
                                    rtt_micros,
                                }).await;
                                true
                            }
                            Ok(Err(_)) => {
                                tracing::warn!("Backend {} health check failed: connection refused", backend_id);
                                let _ = health_tx.send(HealthEvent::BackendUnhealthy {
                                    backend_id,
                                    reason: HealthFailureReason::ConnectionRefused,
                                }).await;
                                false
                            }
                            Err(_) => {
                                tracing::warn!("Backend {} health check failed: timeout", backend_id);
                                let _ = health_tx.send(HealthEvent::BackendUnhealthy {
                                    backend_id,
                                    reason: HealthFailureReason::Timeout,
                                }).await;
                                false
                            }
                        };

                        // Update backend health state
                        let was_alive = backend.is_alive();
                        let now_ms = Self::now_ms();
                        backend.set_health(is_healthy, now_ms);

                        // Send transition event if state changed
                        if was_alive != is_healthy {
                            let from_status = if was_alive { "healthy" } else { "unhealthy" };
                            let to_status = if is_healthy { "healthy" } else { "unhealthy" };
                            tracing::info!(
                                "Backend {} health transition: {} -> {}",
                                backend_id,
                                from_status,
                                to_status
                            );

                            let _ = health_tx.send(HealthEvent::HealthTransition {
                                backend_id,
                                from: if was_alive {
                                    HealthStatus::Healthy
                                } else {
                                    HealthStatus::Unhealthy
                                },
                                to: if is_healthy {
                                    HealthStatus::Healthy
                                } else {
                                    HealthStatus::Unhealthy
                                },
                            }).await;
                        }
                    }
                    tracing::debug!("Health check cycle completed");
                }
            }
        }
        tracing::info!("Health service stopped");
    }
}
