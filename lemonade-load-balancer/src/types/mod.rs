//! Common module for the Load Balancer
//!

mod backend_address;
mod backend_meta;
mod channel_bundle;
mod connection_registry;
mod context;
mod health_registry;
mod metrics_registry;
mod route_table;

/// Backend identifier
pub type BackendId = u8;

pub use backend_address::{BackendAddress, BackendAddressError};
pub use backend_meta::BackendMeta;
pub use channel_bundle::ChannelBundle;
pub use connection_registry::ConnectionRegistry;
pub use context::{Context, ContextError};
pub use health_registry::HealthRegistry;
pub use metrics_registry::{BackendMetrics, MetricsSnapshot};
pub use route_table::RouteTable;
