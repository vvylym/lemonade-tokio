//! Config models module
//!
use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// Config source enum
///
/// Indicates how the configuration was loaded. This is set automatically by
/// the configuration builder and is not part of the serialized config.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConfigSource {
    /// Config loaded from environment variables
    Environment,
    /// Config loaded from file (default)
    #[default]
    File,
}

/// Config struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Source of configuration (set automatically, not part of serialized config)
    #[serde(skip)]
    pub source: ConfigSource,
    /// Runtime config
    pub runtime: RuntimeConfig,
    /// Proxy config
    pub proxy: ProxyConfig,
    /// Strategy
    pub strategy: Strategy,
    /// Backend List
    pub backends: Vec<BackendConfig>,
    /// Health config
    pub health: HealthConfig,
    /// Metrics config
    pub metrics: MetricsConfig,
}

/// Events emitted when configuration changes occur
///
/// These events are sent through the config channel to notify services
/// of configuration changes that require runtime adjustments.
///
/// # Examples
///
/// ```no_run
/// use lemonade_load_balancer::config::models::ConfigEvent;
/// use std::net::SocketAddr;
///
/// // When config migrates successfully
/// let event = ConfigEvent::Migrated;
///
/// // When listen address changes
/// let new_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
/// let event = ConfigEvent::ListenAddressChanged(new_addr);
/// ```
#[derive(Debug, Clone)]
pub enum ConfigEvent {
    /// Configuration successfully migrated
    ///
    /// Emitted when backends, strategy, or other config parameters change.
    /// Services should refresh their view of the configuration.
    Migrated,

    /// Proxy listen address changed
    ///
    /// Emitted when the load balancer's listen address changes, requiring
    /// the proxy service to rebind to the new address. The proxy should:
    /// 1. Stop accepting connections on the old address
    /// 2. Bind to the new address
    /// 3. Allow active connections to drain gracefully
    ListenAddressChanged(SocketAddr),
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
    /// Config file watch interval in milliseconds
    pub config_watch_interval_millis: u64,
}
