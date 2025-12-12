//! Config module
//!
use serde::{Deserialize, Serialize};
use std::time::Duration;

mod builder;
mod error;
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
    work_delay: Duration,
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
}
