//! Worker Address module
//!
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub use error::WorkerAddressError;

/// Worker address struct
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkerAddress(SocketAddr);

impl WorkerAddress {
    /// Create a new worker address
    pub fn parse(address: &str) -> Result<Self, WorkerAddressError> {
        let addr = address.parse::<SocketAddr>()?;
        Ok(Self(addr))
    }
}

impl From<SocketAddr> for WorkerAddress {
    fn from(address: SocketAddr) -> Self {
        Self(address)
    }
}

impl AsRef<SocketAddr> for WorkerAddress {
    fn as_ref(&self) -> &SocketAddr {
        &self.0
    }
}

mod error {
    use std::net::AddrParseError;

    /// Worker address error struct
    #[derive(Debug, thiserror::Error)]
    #[error("invalid worker address: {0}")]
    pub struct WorkerAddressError(String);

    impl From<AddrParseError> for WorkerAddressError {
        fn from(error: AddrParseError) -> Self {
            Self(error.to_string())
        }
    }
}
