//! Config module
//!
use serde::{Deserialize, Serialize};
use std::time::Duration;

mod builder;
mod error;
mod serde_helpers;
mod worker_address;

pub use builder::ConfigBuilder;
pub use error::ConfigError;
pub use worker_address::{WorkerAddress, WorkerAddressError};

/// Config struct
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    /// Backend address
    listen_address: WorkerAddress,
    /// Service name
    service_name: String,
    /// Work delay
    #[serde(with = "serde_helpers")]
    work_delay: Duration,
    /// OTLP exporter endpoint (optional)
    #[serde(default)]
    otlp_endpoint: Option<String>,
    /// OTLP exporter protocol (optional)
    #[serde(default)]
    otlp_protocol: Option<String>,
}

impl Config {
    /// Create a new config
    pub fn new(
        listen_address: impl Into<WorkerAddress>,
        service_name: impl Into<String>,
        work_delay: impl Into<Duration>,
    ) -> Self {
        Self {
            listen_address: listen_address.into(),
            service_name: service_name.into(),
            work_delay: work_delay.into(),
            otlp_endpoint: None,
            otlp_protocol: None,
        }
    }
    /// Get the listen address
    pub fn listen_address(&self) -> &WorkerAddress {
        &self.listen_address
    }

    /// Get the service name
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// Get the work delay
    pub fn work_delay(&self) -> Duration {
        self.work_delay
    }

    /// Get the OTLP endpoint (if configured)
    pub fn otlp_endpoint(&self) -> Option<&str> {
        self.otlp_endpoint.as_deref()
    }

    /// Get the OTLP protocol (if configured)
    pub fn otlp_protocol(&self) -> Option<&str> {
        self.otlp_protocol.as_deref()
    }
}
