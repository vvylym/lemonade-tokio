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

/// Connection lifecycle events
///
/// These events track the lifecycle of connections between the load balancer
/// and backend servers. They are used for connection tracking, metrics collection,
/// and graceful shutdown coordination.
///
/// # Examples
///
/// ```no_run
/// use lemonade_load_balancer::proxy::models::ConnectionEvent;
///
/// // Report new connection opened
/// let event = ConnectionEvent::Opened { backend_id: 0 };
///
/// // Report connection closed
/// let event = ConnectionEvent::Closed { backend_id: 0 };
/// ```
///
/// # Connection Lifecycle
///
/// 1. **Opened**: Emitted when a new connection to a backend is successfully established
/// 2. **Closed**: Emitted when the connection is gracefully closed or terminated
///
/// These events enable accurate tracking of active connections per backend,
/// which is critical for:
/// - Least Connections load balancing strategy
/// - Graceful backend draining during config changes
/// - Connection metrics and observability
#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    /// A connection to a backend was successfully opened
    ///
    /// Emitted after TCP handshake completes and the connection is ready
    /// to forward requests. The backend's active connection count should
    /// be incremented.
    Opened {
        /// Backend identifier
        backend_id: BackendId,
    },

    /// A connection to a backend was closed
    ///
    /// Emitted when the connection is closed, either gracefully or due to
    /// an error. The backend's active connection count should be decremented.
    Closed {
        /// Backend identifier
        backend_id: BackendId,
    },
}
