//! Common module for the Load Balancer
//!

mod backend;
mod backend_address;
mod backend_meta;
mod channel_bundle;
mod context;
mod metrics_registry;
mod route_table;

/// Backend identifier
pub type BackendId = u8;

pub use backend::{Backend, BackendConfig};
pub use backend_address::{BackendAddress, BackendAddressError};
pub use backend_meta::BackendMeta;
pub use channel_bundle::ChannelBundle;
pub use context::{Context, ContextError};
pub use metrics_registry::{BackendMetrics, MetricsSnapshot};
pub use route_table::RouteTable;
