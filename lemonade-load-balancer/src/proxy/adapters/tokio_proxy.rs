//! Tokio implementation of ProxyService
//!
//! Uses tokio for async connection acceptance and bidirectional proxying
//! Runs on main thread for maximum performance (hot path)

use crate::prelude::*;
use crate::proxy::error::ProxyError;
use crate::proxy::models::{ConnectionEvent, ProxyConfig};
use crate::proxy::port::ProxyService;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use std::io;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinSet;
use tracing::instrument;

/// Tokio-based proxy service implementation
#[derive(Clone)]
pub struct TokioProxyService {
    /// Proxy configuration (reference to global config's proxy slice)
    config: Arc<ArcSwap<ProxyConfig>>,
}

impl TokioProxyService {
    /// Create a new TokioProxyService
    ///
    /// # Arguments
    /// * `config` - Arc<ArcSwap<ProxyConfig>> reference to proxy config
    ///
    /// # Returns
    /// * `Ok(Self)` if service was created successfully
    pub fn new(config: Arc<ArcSwap<ProxyConfig>>) -> Result<Self, ProxyError> {
        Ok(Self { config })
    }

    /// Handle a single proxy connection
    #[instrument(
        skip(self, client_stream, backend, ctx),
        fields(
            service.name = "lemonade-load-balancer",
            backend.id = %backend.id(),
            backend.name = %backend.name().unwrap_or("unknown"),
            backend.addr = %backend.address()
        )
    )]
    async fn handle_connection(
        &self,
        client_stream: TcpStream,
        backend: Arc<Backend>,
        ctx: Arc<Context>,
    ) -> Result<(), ProxyError> {
        let backend_id = backend.id();
        let backend_addr = backend.address();

        // Increment connection counter
        backend.increment_connection();

        // Send connection opened event (non-blocking)
        let _ = ctx
            .channels()
            .connection_tx()
            .try_send(ConnectionEvent::Opened { backend_id });

        let connection_start = Instant::now();

        // Connect to backend
        let backend_stream = match TcpStream::connect(backend_addr).await {
            Ok(stream) => stream,
            Err(e) => {
                backend.decrement_connection();
                ctx.notify_connection_closed();

                // ALERT HEALTH SERVICE - send failure event
                let failure_event = match e.kind() {
                    io::ErrorKind::ConnectionRefused => {
                        BackendFailureEvent::ConnectionRefused { backend_id }
                    }
                    io::ErrorKind::TimedOut => {
                        BackendFailureEvent::Timeout { backend_id }
                    }
                    _ => BackendFailureEvent::BackendClosed { backend_id },
                };

                let _ = ctx.channels().backend_failure_tx().try_send(failure_event);

                // Send metrics event
                let _ =
                    ctx.channels()
                        .metrics_tx()
                        .try_send(MetricsEvent::RequestFailed {
                            backend_id,
                            latency_micros: connection_start.elapsed().as_micros() as u64,
                            error_class: MetricsErrorClass::ConnectionRefused,
                        });

                return Err(ProxyError::Io(e));
            }
        };

        // Proxy data bidirectionally
        let (mut client_read, mut client_write) = tokio::io::split(client_stream);
        let (mut backend_read, mut backend_write) = tokio::io::split(backend_stream);

        let client_to_backend = tokio::spawn(async move {
            let mut bytes_sent = 0u64;
            let mut buf = [0u8; 8192];
            loop {
                match client_read.read(&mut buf).await {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        if backend_write.write_all(&buf[..n]).await.is_err() {
                            break;
                        }
                        bytes_sent += n as u64;
                    }
                    Err(_) => break,
                }
            }
            bytes_sent
        });

        let backend_to_client = tokio::spawn(async move {
            let mut bytes_received = 0u64;
            let mut buf = [0u8; 8192];
            loop {
                match backend_read.read(&mut buf).await {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        if client_write.write_all(&buf[..n]).await.is_err() {
                            break;
                        }
                        bytes_received += n as u64;
                    }
                    Err(_) => break,
                }
            }
            bytes_received
        });

        // Wait for both directions to complete
        let (bytes_sent, bytes_received) =
            tokio::join!(client_to_backend, backend_to_client);
        let bytes_sent = bytes_sent.unwrap_or(0);
        let bytes_received = bytes_received.unwrap_or(0);
        let duration_micros = connection_start.elapsed().as_micros() as u64;

        // Decrement connection counter
        backend.decrement_connection();
        ctx.notify_connection_closed();

        // Send connection closed event
        let _ = ctx
            .channels()
            .connection_tx()
            .try_send(ConnectionEvent::Closed { backend_id });

        // Send metrics event
        let _ = ctx
            .channels()
            .metrics_tx()
            .try_send(MetricsEvent::ConnectionClosed {
                backend_id,
                duration_micros,
                bytes_in: bytes_received,
                bytes_out: bytes_sent,
            });

        Ok(())
    }
}

#[async_trait]
impl ProxyService for TokioProxyService {
    #[tracing::instrument(skip(self, ctx), fields(service.name = "lemonade-load-balancer", service.type = "proxy"))]
    async fn accept_connections(&self, ctx: Arc<Context>) -> Result<(), ProxyError> {
        let mut shutdown_rx = ctx.channels().shutdown_rx();
        let mut config_rx = ctx.channels().config_rx();

        // Get initial listen address
        let mut current_addr = ctx.config().proxy.listen_address;
        let mut listener = TcpListener::bind(current_addr).await?;
        tracing::info!("Proxy listening on {}", current_addr);

        // Track active connection tasks
        let mut conn_tasks = JoinSet::new();

        loop {
            tokio::select! {
                // Shutdown signal
                _ = shutdown_rx.recv() => {
                    tracing::info!("Proxy received shutdown signal");
                    break;
                }

                // Config change (listen address change)
                result = config_rx.recv() => {
                    if let Ok(ConfigEvent::ListenAddressChanged(new_addr)) = result && new_addr != current_addr {
                        tracing::info!(
                            "Listen address changed: {} -> {}",
                            current_addr,
                            new_addr
                        );

                        // Stop accepting on old listener (drop it)
                        // Active connections continue via spawned tasks
                        drop(listener);

                        // Bind to new address
                        match TcpListener::bind(new_addr).await {
                            Ok(new_listener) => {
                                listener = new_listener;
                                current_addr = new_addr;
                                tracing::info!("Now accepting on {}", new_addr);
                            }
                            Err(e) => {
                                tracing::error!("Failed to bind to {}: {}", new_addr, e);
                                // Re-bind to old address
                                match TcpListener::bind(current_addr).await {
                                    Ok(old_listener) => {
                                        listener = old_listener;
                                        tracing::warn!("Reverted to {}", current_addr);
                                    }
                                    Err(e2) => {
                                        tracing::error!("Failed to revert to old address: {}", e2);
                                        return Err(ProxyError::Io(e2));
                                    }
                                }
                            }
                        }
                    }
                }

                // Accept new connection
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, peer_addr)) => {
                            // Check max connections
                            let config = self.config.load();
                            if let Some(max_conns) = config.max_connections {
                                let routing = ctx.routing_table();
                                let total_connections: usize = routing
                                    .all_backends()
                                    .iter()
                                    .map(|b| b.active_connections())
                                    .sum();
                                if total_connections >= max_conns as usize {
                                    tracing::warn!(
                                        "Max connections reached ({}), rejecting connection from {}",
                                        max_conns,
                                        peer_addr
                                    );
                                    drop(stream);
                                    continue;
                                }
                            }

                            // Pick backend using strategy
                            let strategy = ctx.strategy();
                            let backend_meta = match strategy.pick_backend(ctx.clone()).await {
                                Ok(b) => b,
                                Err(e) => {
                                    tracing::warn!("No backend available: {}", e);
                                    drop(stream);
                                    continue;
                                }
                            };

                            // Get backend from route table
                            let routing = ctx.routing_table();
                            let backend = match routing.get(*backend_meta.id()) {
                                Some(b) => b,
                                None => {
                                    tracing::warn!("Backend {} not found in route table", backend_meta.id());
                                    drop(stream);
                                    continue;
                                }
                            };

                            // Check if backend accepts new connections (not draining and healthy)
                            if !backend.can_accept_new_connections() {
                                tracing::debug!(
                                    "Backend {} is draining or unhealthy, cannot accept new connection",
                                    backend.id()
                                );
                                drop(stream);
                                continue;
                            }

                            // Spawn connection handler (clone ctx before move)
                            let svc_clone = self.clone();
                            let ctx_clone = ctx.clone();
                            conn_tasks.spawn(async move {
                                let _ = svc_clone.handle_connection(stream, backend, ctx_clone).await;
                            });
                        }
                        Err(e) => {
                            tracing::error!("Accept error: {}", e);
                            // Brief pause to avoid tight loop on errors
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                    }
                }

                // Clean up finished connection tasks
                Some(_) = conn_tasks.join_next() => {
                    // Connection finished, task cleaned up
                }
            }
        }

        // Shutdown: wait for active connections to finish
        tracing::info!(
            "Waiting for {} active connections to complete",
            conn_tasks.len()
        );
        while (conn_tasks.join_next().await).is_some() {
            // Drain all active connection tasks
        }

        tracing::info!("All proxy connections closed");
        Ok(())
    }
}
