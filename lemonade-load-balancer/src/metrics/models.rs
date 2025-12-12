//! Metrics models module
//!
use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// Metrics config struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Metrics collection interval
    #[serde(with = "crate::config::serde_helpers")]
    pub interval: Duration,
    /// Metrics collection timeout
    #[serde(with = "crate::config::serde_helpers")]
    pub timeout: Duration,
}

/// Metrics event enum
#[derive(Debug, Clone)]
pub enum MetricsEvent {
    /// A new connection was accepted for a backend
    ConnectionOpened {
        /// Backend ID
        backend_id: u8,
        /// Timestamp
        at_micros: u64,
    },
    /// A connection was closed (normal or error)
    ConnectionClosed {
        /// Backend ID
        backend_id: u8,
        /// Duration in microseconds
        duration_micros: u64,
        /// Bytes in
        bytes_in: u64,
        /// Bytes out
        bytes_out: u64,
    },
    /// A proxied request finished
    RequestCompleted {
        /// Backend ID
        backend_id: u8,
        /// Latency in microseconds
        latency_micros: u64,
        /// Status code
        status_code: u16,
    },
    /// A proxied request failed
    RequestFailed {
        /// Backend ID
        backend_id: u8,
        /// Latency in microseconds
        latency_micros: u64,
        /// Error class
        error_class: MetricsErrorClass,
    },
    /// Periodic snapshot trigger (internal tick)
    FlushSnapshot,
}

/// Metrics error class enum
#[derive(Debug, Clone, Copy)]
pub enum MetricsErrorClass {
    /// Timeout
    Timeout,
    /// Connection refused
    ConnectionRefused,
    /// Backend closed
    BackendClosed,
    /// Protocol
    Protocol,
    /// Unknown
    Unknown,
}
