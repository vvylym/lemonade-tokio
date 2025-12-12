//! Backend implementation of HealthService
//!
//! Performs periodic health checks on backends using TCP connections

use crate::health::error::HealthError;
use crate::health::models::{HealthEvent, HealthFailureReason, HealthStatus};
use crate::health::port::HealthService;
use crate::prelude::*;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpStream;
use tokio::sync::Notify as TokioNotify;

/// Backend health service implementation
pub struct BackendHealthService {
    /// Health configuration (reference to global config's health slice)
    config: Arc<ArcSwap<HealthConfig>>,
    /// Shutdown notification
    shutdown: Arc<TokioNotify>,
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
        Ok(Self {
            config,
            shutdown: Arc::new(TokioNotify::new()),
        })
    }
}

#[async_trait]
impl HealthService for BackendHealthService {
    async fn start(&self, ctx: Arc<Context>) -> Result<(), HealthError> {
        let health_tx = ctx.channel_bundle().health_sender();
        let health_registry = ctx.health_registry();
        let config_arc = self.config.clone();
        let shutdown_clone = self.shutdown.clone();

        // Spawn background task to perform periodic health checks
        tokio::spawn(async move {
            loop {
                let config = config_arc.load_full();
                let interval = config.interval;

                tokio::select! {
                    _ = shutdown_clone.notified() => {
                        break;
                    }
                    _ = tokio::time::sleep(interval) => {
                        // Get current routing table and check all backends
                        let routing = ctx.routing_table();

                        // Check each backend
                        for (idx, backend) in routing.iter().enumerate() {
                            let backend_id = *backend.id();
                            let address = *backend.address().as_ref();

                            // Perform health check
                            let check_start = Instant::now();
                            let is_healthy = match tokio::time::timeout(
                                config.timeout,
                                TcpStream::connect(address),
                            )
                            .await
                            {
                                Ok(Ok(_)) => {
                                    let rtt_micros = check_start.elapsed().as_micros() as u64;
                                    let _ = health_tx.send(HealthEvent::BackendHealthy {
                                        backend_id,
                                        rtt_micros,
                                    }).await;
                                    true
                                }
                                Ok(Err(_)) => {
                                    let _ = health_tx.send(HealthEvent::BackendUnhealthy {
                                        backend_id,
                                        reason: HealthFailureReason::ConnectionRefused,
                                    }).await;
                                    false
                                }
                                Err(_) => {
                                    let _ = health_tx.send(HealthEvent::BackendUnhealthy {
                                        backend_id,
                                        reason: HealthFailureReason::Timeout,
                                    }).await;
                                    false
                                }
                            };

                            // Update health registry
                            let now_ms = Instant::now().elapsed().as_millis() as u64;
                            let was_alive = health_registry.is_alive(idx);
                            health_registry.set_alive(idx, is_healthy, now_ms);

                            // Send transition event if state changed
                            if was_alive != is_healthy {
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
                    }
                }
            }
        });

        Ok(())
    }

    async fn shutdown(&self) -> Result<(), HealthError> {
        self.shutdown.notify_waiters();
        Ok(())
    }
}
