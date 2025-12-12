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
