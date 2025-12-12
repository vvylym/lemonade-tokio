//! Proxy models module
//!
use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// Proxy config struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Listen address
    pub listen_address: SocketAddr,
    /// Max connections
    pub max_connections: Option<u64>,
}
