//! Tokio implementation of ProxyService
//!
//! Uses tokio for async connection acceptance and bidirectional proxying

use crate::prelude::*;
use crate::proxy::error::ProxyError;
use crate::proxy::models::ProxyConfig;
use crate::proxy::port::ProxyService;
use arc_swap::ArcSwap;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify as TokioNotify;

/// Tokio-based proxy service implementation
#[derive(Clone)]
pub struct TokioProxyService {
    /// Proxy configuration (reference to global config's proxy slice)
    config: Arc<ArcSwap<ProxyConfig>>,
    /// Shutdown notification
    shutdown: Arc<TokioNotify>,
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
        Ok(Self {
            config,
            shutdown: Arc::new(TokioNotify::new()),
        })
    }

    /// Handle a single proxy connection
    async fn handle_connection(
        &self,
        client_stream: TcpStream,
        backend: BackendMeta,
        ctx: Arc<Context>,
    ) -> Result<(), ProxyError> {
        let backend_id = *backend.id();
        let backend_addr = *backend.address().as_ref();
        let routing = ctx.routing_table();
        let backend_idx = routing.find_index(backend_id).ok_or_else(|| {
            ProxyError::Unexpected("Backend not found in routing table".to_string())
        })?;

        // Track connection
        let connections = ctx.connection_registry();
        connections.increment(backend_idx);

        // Send ConnectionOpened event
        let metrics_tx = ctx.channel_bundle().metrics_sender();
        let connection_start = Instant::now();
        let _ = metrics_tx
            .send(MetricsEvent::ConnectionOpened {
                backend_id,
                at_micros: connection_start.elapsed().as_micros() as u64,
            })
            .await;

        // Connect to backend
        let backend_stream = match TcpStream::connect(backend_addr).await {
            Ok(stream) => stream,
            Err(e) => {
                connections.decrement(backend_idx);
                let _ = metrics_tx
                    .send(MetricsEvent::RequestFailed {
                        backend_id,
                        latency_micros: connection_start.elapsed().as_micros() as u64,
                        error_class: MetricsErrorClass::ConnectionRefused,
                    })
                    .await;
                return Err(ProxyError::Io(e));
            }
        };

        // Proxy data bidirectionally using copy
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

        // Decrement connection count
        connections.decrement(backend_idx);

        // Send ConnectionClosed event
        let _ = metrics_tx
            .send(MetricsEvent::ConnectionClosed {
                backend_id,
                duration_micros,
                bytes_in: bytes_received,
                bytes_out: bytes_sent,
            })
            .await;

        // Notify connection closed for drain logic
        ctx.notify_connection_closed();

        Ok(())
    }
}

#[async_trait]
impl ProxyService for TokioProxyService {
    async fn accept_connections(&self, ctx: &Arc<Context>) -> Result<(), ProxyError> {
        let config = self.config.load_full();
        let listener = TcpListener::bind(config.listen_address).await?;
        let _local_addr = listener.local_addr()?;

        // Note: If config.listen_address.port() was 0, the OS assigned a port
        // The actual bound address is in _local_addr, but we can't update config as it's read-only

        let shutdown_clone = self.shutdown.clone();
        let service_clone = self.clone();
        let ctx_clone = ctx.clone();

        loop {
            tokio::select! {
                _ = shutdown_clone.notified() => {
                    break;
                }
                result = listener.accept() => {
                    match result {
                        Ok((client_stream, _)) => {
                            // Check max_connections limit
                            let connections = ctx_clone.connection_registry();
                            let config = self.config.load_full();
                            if let Some(max_conns) = config.max_connections
                                && connections.total() >= max_conns as usize
                            {
                                // Reject connection
                                drop(client_stream);
                                continue;
                            }

                            // Pick backend using strategy
                            let strategy = ctx_clone.strategy();
                            match strategy.pick_backend(ctx_clone.clone()).await {
                                Ok(backend) => {
                                    // Spawn proxy task
                                    let service = service_clone.clone();
                                    let ctx_task = ctx_clone.clone();
                                    tokio::spawn(async move {
                                        if let Err(e) = service.handle_connection(client_stream, backend, ctx_task).await {
                                            eprintln!("Proxy connection error: {}", e);
                                        }
                                    });
                                }
                                Err(_) => {
                                    // No backend available, close connection
                                    drop(client_stream);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to accept connection: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
