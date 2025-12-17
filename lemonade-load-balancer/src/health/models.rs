//! Health models module
//!
use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// Health config struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Health check interval
    #[serde(with = "crate::config::serde_helpers")]
    pub interval: Duration,
    /// Health check timeout
    #[serde(with = "crate::config::serde_helpers")]
    pub timeout: Duration,
}

/// Health event struct
#[derive(Debug, Clone)]
pub enum HealthEvent {
    /// Explicit backend check triggered by timer or config reload
    CheckBackend {
        /// Backend ID
        backend_id: u8,
    },
    /// Health probe succeeded
    BackendHealthy {
        /// Backend ID
        backend_id: u8,
        /// Round trip time in microseconds
        rtt_micros: u64,
    },
    /// Health probe failed
    BackendUnhealthy {
        /// Backend ID
        backend_id: u8,
        /// Reason
        reason: HealthFailureReason,
    },
    /// Health state changed (edge-triggered)
    HealthTransition {
        /// Backend ID
        backend_id: u8,
        /// From health status
        from: HealthStatus,
        /// To health status
        to: HealthStatus,
    },
    /// Config reload changed backend parameters
    BackendConfigUpdated {
        /// Backend ID
        backend_id: u8,
    },
}

/// Health failure reason enum
#[derive(Debug, Clone, Copy)]
pub enum HealthFailureReason {
    /// Timeout
    Timeout,
    /// Connection refused
    ConnectionRefused,
    /// Invalid response
    InvalidResponse,
    /// DNS error
    DnsError,
    /// Transport error
    Transport,
}

/// Health status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Healthy
    Healthy,
    /// Unhealthy
    Unhealthy,
}

/// Backend failure events from proxy to health service
///
/// These events are sent by the proxy service when it encounters errors
/// while attempting to forward requests to backends. The health service
/// uses these events for immediate failure detection without waiting for
/// the next periodic health check.
///
/// # Examples
///
/// ```no_run
/// use lemonade_load_balancer::health::models::BackendFailureEvent;
///
/// // Report connection refused
/// let event = BackendFailureEvent::ConnectionRefused { backend_id: 0 };
///
/// // Report timeout
/// let event = BackendFailureEvent::Timeout { backend_id: 1 };
///
/// // Report consecutive errors
/// let event = BackendFailureEvent::ConsecutiveErrors {
///     backend_id: 2,
///     count: 5,
/// };
/// ```
#[derive(Debug, Clone)]
pub enum BackendFailureEvent {
    /// Proxy detected connection refused
    ///
    /// The backend refused the TCP connection. This typically indicates
    /// the backend service is not listening on the expected port or
    /// the backend host is unreachable.
    ConnectionRefused {
        /// Backend identifier
        backend_id: BackendId,
    },

    /// Proxy detected timeout
    ///
    /// The backend did not respond within the configured timeout period.
    /// This may indicate the backend is overloaded or experiencing network issues.
    Timeout {
        /// Backend identifier
        backend_id: BackendId,
    },

    /// Proxy detected backend closed connection
    ///
    /// The backend closed the connection unexpectedly before completing
    /// the request/response cycle. This may indicate a backend crash or
    /// premature connection termination.
    BackendClosed {
        /// Backend identifier
        backend_id: BackendId,
    },

    /// Multiple consecutive errors detected
    ///
    /// The proxy has detected multiple consecutive failures when attempting
    /// to connect to this backend. This triggers immediate health check
    /// escalation.
    ConsecutiveErrors {
        /// Backend identifier
        backend_id: BackendId,
        /// Number of consecutive errors
        count: u32,
    },
}
