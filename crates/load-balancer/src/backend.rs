//! Backend module
//!
//! Defines backend server structures and utilities

use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::{Duration, Instant};

/// Backend Id type
pub type BackendId = u8;

/// Represents a backend server that can receive proxied connections.
///
#[derive(Debug, Clone)]
pub struct Backend {
    /// Typed identifier for efficient HashMap lookups.
    pub id: BackendId,
    /// Human-readable name for logging and display purposes.
    pub name: String,
    /// Network address in the format "host:port" (e.g., "127.0.0.1:8081").
    pub addr: String,
    /// Weight used for weighted round-robin algorithm. Higher weights receive more connections.
    pub weight: u8,
    /// Whether this backend is currently healthy and can accept connections.
    pub healthy: bool,
    /// Current number of active connections to this backend (atomic for lock-free access).
    pub connection_count: Arc<AtomicU64>,
    /// Average response time measured during health checks.
    pub avg_response_time: Duration,
    /// Total connection duration for calculating average (atomic for lock-free access).
    pub total_connection_duration: Arc<AtomicU64>, // in milliseconds
    /// Total number of connections completed (atomic for lock-free access).
    pub total_connections: Arc<AtomicU64>,
    /// Number of connection errors for this backend (atomic for lock-free access).
    pub error_count: Arc<AtomicU64>,
    /// Timestamp of last connection failure (if any)
    pub last_connection_failure: Option<Instant>,
    /// Timestamp of last health check
    pub last_health_check: Option<Instant>,
}
