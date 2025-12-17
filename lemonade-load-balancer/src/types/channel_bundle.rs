//! Channel Bundle module
//!
use crate::prelude::*;
use std::sync::Mutex;

/// Channel bundle with separate typed channels for all event types
#[derive(Debug)]
pub struct ChannelBundle {
    // All fields private for encapsulation

    // Config change notifications (broadcast - multiple listeners)
    config_tx: broadcast::Sender<ConfigEvent>,

    // Health events (mpsc - single processor)
    health_tx: mpsc::Sender<HealthEvent>,
    health_rx: Mutex<Option<mpsc::Receiver<HealthEvent>>>,

    // Backend failure events from proxy to health service (mpsc - single processor)
    backend_failure_tx: mpsc::Sender<BackendFailureEvent>,
    backend_failure_rx: Mutex<Option<mpsc::Receiver<BackendFailureEvent>>>,

    // Metrics events (mpsc - single processor)
    metrics_tx: mpsc::Sender<MetricsEvent>,
    metrics_rx: Mutex<Option<mpsc::Receiver<MetricsEvent>>>,

    // Connection lifecycle events (mpsc - single processor)
    connection_tx: mpsc::Sender<ConnectionEvent>,
    connection_rx: Mutex<Option<mpsc::Receiver<ConnectionEvent>>>,

    // Shutdown signal (broadcast - all services listen)
    shutdown_tx: broadcast::Sender<()>,
}

impl ChannelBundle {
    /// Create a new channel bundle
    pub fn new(
        metrics_cap: usize,
        health_cap: usize,
        connection_cap: usize,
        backend_failure_cap: usize,
    ) -> Self {
        let (config_tx, _) = broadcast::channel(16);
        let (health_tx, health_rx) = mpsc::channel(health_cap);
        let (backend_failure_tx, backend_failure_rx) = mpsc::channel(backend_failure_cap);
        let (metrics_tx, metrics_rx) = mpsc::channel(metrics_cap);
        let (connection_tx, connection_rx) = mpsc::channel(connection_cap);
        let (shutdown_tx, _) = broadcast::channel(1);

        Self {
            config_tx,
            health_tx,
            health_rx: Mutex::new(Some(health_rx)),
            backend_failure_tx,
            backend_failure_rx: Mutex::new(Some(backend_failure_rx)),
            metrics_tx,
            metrics_rx: Mutex::new(Some(metrics_rx)),
            connection_tx,
            connection_rx: Mutex::new(Some(connection_rx)),
            shutdown_tx,
        }
    }

    // Config channel accessors

    /// Get config event sender (broadcast - can be cloned)
    pub fn config_tx(&self) -> broadcast::Sender<ConfigEvent> {
        self.config_tx.clone()
    }

    /// Get config event receiver (broadcast - can have multiple subscribers)
    pub fn config_rx(&self) -> broadcast::Receiver<ConfigEvent> {
        self.config_tx.subscribe()
    }

    // Health channel accessors

    /// Get health event sender (mpsc - can be cloned)
    pub fn health_tx(&self) -> mpsc::Sender<HealthEvent> {
        self.health_tx.clone()
    }

    /// Get health event receiver (mpsc - taken once, single consumer)
    pub fn health_rx(&self) -> Option<mpsc::Receiver<HealthEvent>> {
        self.health_rx.lock().unwrap().take()
    }

    // Backend failure channel accessors

    /// Get backend failure event sender (mpsc - can be cloned)
    pub fn backend_failure_tx(&self) -> mpsc::Sender<BackendFailureEvent> {
        self.backend_failure_tx.clone()
    }

    /// Get backend failure event receiver (mpsc - taken once, single consumer)
    pub fn backend_failure_rx(&self) -> Option<mpsc::Receiver<BackendFailureEvent>> {
        self.backend_failure_rx.lock().unwrap().take()
    }

    // Metrics channel accessors

    /// Get metrics event sender (mpsc - can be cloned)
    pub fn metrics_tx(&self) -> mpsc::Sender<MetricsEvent> {
        self.metrics_tx.clone()
    }

    /// Get metrics event receiver (mpsc - taken once, single consumer)
    pub fn metrics_rx(&self) -> Option<mpsc::Receiver<MetricsEvent>> {
        self.metrics_rx.lock().unwrap().take()
    }

    // Connection channel accessors

    /// Get connection event sender (mpsc - can be cloned)
    pub fn connection_tx(&self) -> mpsc::Sender<ConnectionEvent> {
        self.connection_tx.clone()
    }

    /// Get connection event receiver (mpsc - taken once, single consumer)
    pub fn connection_rx(&self) -> Option<mpsc::Receiver<ConnectionEvent>> {
        self.connection_rx.lock().unwrap().take()
    }

    // Shutdown channel accessors

    /// Get shutdown signal sender (broadcast - can be cloned)
    pub fn shutdown_tx(&self) -> broadcast::Sender<()> {
        self.shutdown_tx.clone()
    }

    /// Get shutdown signal receiver (broadcast - can have multiple subscribers)
    pub fn shutdown_rx(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }
}
