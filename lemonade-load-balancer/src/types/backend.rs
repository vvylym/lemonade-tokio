//! Backend module
//!
//! Unified backend representation with metadata and runtime state

use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, AtomicUsize, Ordering};

/// Unified backend representation with metadata and runtime state
#[derive(Debug)]
pub struct Backend {
    // Immutable metadata
    id: BackendId,
    name: Option<String>,
    address: SocketAddr,
    weight: Option<u8>,

    // Mutable state (atomic for lock-free access)
    alive: AtomicBool, // Default: true (healthy until proven otherwise)
    last_health_check_ms: AtomicU64,
    active_connections: AtomicUsize, // Used by health service to avoid checking busy backends
    total_requests: AtomicU64,
    total_errors: AtomicU64,
    total_latency_ms: AtomicU64,
    last_metrics_update_ms: AtomicU64,

    // Migration state
    status: AtomicU8, // Active = 0, Draining = 1
}

impl Backend {
    /// Create new backend (STARTS HEALTHY by default)
    pub fn new(config: BackendConfig) -> Self {
        Self {
            id: config.id,
            name: config.name,
            address: config.address,
            weight: config.weight,
            alive: AtomicBool::new(true), // ← HEALTHY BY DEFAULT
            last_health_check_ms: AtomicU64::new(0),
            active_connections: AtomicUsize::new(0),
            total_requests: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            total_latency_ms: AtomicU64::new(0),
            last_metrics_update_ms: AtomicU64::new(0),
            status: AtomicU8::new(0), // Active
        }
    }

    // Metadata accessors

    /// Get the backend id
    pub fn id(&self) -> BackendId {
        self.id
    }

    /// Get the backend name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Get the backend address
    pub fn address(&self) -> SocketAddr {
        self.address
    }

    /// Get the backend weight
    pub fn weight(&self) -> Option<u8> {
        self.weight
    }

    // Health methods

    /// Check if backend is alive
    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
    }

    /// Set health status
    pub fn set_health(&self, alive: bool, now_ms: u64) {
        self.alive.store(alive, Ordering::Relaxed);
        self.last_health_check_ms.store(now_ms, Ordering::Relaxed);
    }

    /// Get last health check timestamp
    pub fn last_health_check(&self) -> u64 {
        self.last_health_check_ms.load(Ordering::Relaxed)
    }

    // Connection methods

    /// Increment active connection count
    pub fn increment_connection(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active connection count
    pub fn decrement_connection(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get active connection count
    pub fn active_connections(&self) -> usize {
        self.active_connections.load(Ordering::Relaxed)
    }

    /// Check if backend has capacity for health check
    /// Health service should skip backends with high connection load
    pub fn has_capacity_for_health_check(&self, max_connections: usize) -> bool {
        self.active_connections() < max_connections
    }

    // Metrics methods

    /// Record a request (success or failure)
    pub fn record_request(&self, latency_ms: u64, is_error: bool) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if is_error {
            self.total_errors.fetch_add(1, Ordering::Relaxed);
        }
        self.total_latency_ms
            .fetch_add(latency_ms, Ordering::Relaxed);
    }

    /// Get metrics snapshot
    pub fn metrics_snapshot(&self) -> BackendMetrics {
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let total_errors = self.total_errors.load(Ordering::Relaxed);
        let total_latency_ms = self.total_latency_ms.load(Ordering::Relaxed);
        let last_updated_ms = self.last_metrics_update_ms.load(Ordering::Relaxed);

        // Calculate average latency
        let avg_latency_ms = if total_requests > 0 && total_latency_ms > 0 {
            total_latency_ms as f64 / total_requests as f64
        } else {
            0.0
        };

        // Calculate error rate
        let error_rate = if total_requests > 0 {
            total_errors as f32 / total_requests as f32
        } else {
            0.0
        };

        // Calculate p95 latency (simplified - use average * 1.5 for now)
        let p95_latency_ms = avg_latency_ms * 1.5;

        BackendMetrics {
            avg_latency_ms,
            p95_latency_ms,
            error_rate,
            last_updated_ms,
        }
    }

    /// Update last metrics update timestamp
    pub fn update_metrics_timestamp(&self, now_ms: u64) {
        self.last_metrics_update_ms.store(now_ms, Ordering::Relaxed);
    }

    // Migration methods

    /// Mark backend as draining
    pub fn mark_draining(&self) {
        self.status.store(1, Ordering::Relaxed);
    }

    /// Check if backend is draining
    pub fn is_draining(&self) -> bool {
        self.status.load(Ordering::Relaxed) == 1
    }

    /// Check if backend is active (not draining)
    pub fn is_active(&self) -> bool {
        self.status.load(Ordering::Relaxed) == 0
    }

    /// Check if backend can accept new connections
    /// Returns true if backend is alive and not draining
    pub fn can_accept_new_connections(&self) -> bool {
        self.is_alive() && self.is_active()
    }
}

/// Backend configuration for deserialization
///
/// This struct represents the backend configuration as it appears in
/// configuration files (JSON/TOML/YAML). It is converted into a [`Backend`]
/// instance with runtime state tracking.
///
/// # Examples
///
/// ```
/// use lemonade_load_balancer::types::backend::BackendConfig;
/// use std::net::SocketAddr;
///
/// let config = BackendConfig {
///     id: 0,
///     name: Some("backend-1".to_string()),
///     address: "127.0.0.1:8080".parse().unwrap(),
///     weight: Some(10),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    /// Unique backend identifier (0-255)
    pub id: BackendId,
    /// Optional human-readable backend name
    pub name: Option<String>,
    /// Backend socket address (IP:port)
    pub address: SocketAddr,
    /// Optional weight for weighted load balancing strategies
    pub weight: Option<u8>,
}

impl From<BackendMeta> for BackendConfig {
    fn from(meta: BackendMeta) -> Self {
        Self {
            id: *meta.id(),
            name: meta.name().cloned(),
            address: *meta.address().as_ref(),
            weight: meta.weight(),
        }
    }
}
