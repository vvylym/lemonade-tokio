//! Metrics adapters module
//!

mod aggregating;
mod external;

pub use aggregating::AggregatingMetricsService;
pub use external::ExternalMetricsService;
