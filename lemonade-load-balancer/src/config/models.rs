//! Config models module
//!
use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// Config struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Runtime config
    pub runtime: RuntimeConfig,
    /// Proxy config
    pub proxy: ProxyConfig,
    /// Strategy
    pub strategy: Strategy,
    /// Backend List
    pub backends: Vec<BackendMeta>,
    /// Health config
    pub health: HealthConfig,
    /// Metrics config
    pub metrics: MetricsConfig,
}

/// Runtime config struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Metrics capacity
    pub metrics_cap: usize,
    /// Health capacity
    pub health_cap: usize,
    /// Drain timeout in milliseconds
    pub drain_timeout_millis: u64,
    /// Background timeout in milliseconds
    pub background_timeout_millis: u64,
    /// Accept timeout in milliseconds
    pub accept_timeout_millis: u64,
}
