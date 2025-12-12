//! Backend address module
//!
pub use error::BackendAddressError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::net::{SocketAddr, ToSocketAddrs};

/// Backend address struct
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BackendAddress(SocketAddr);

impl Serialize for BackendAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for BackendAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        BackendAddress::parse(&s).map_err(serde::de::Error::custom)
    }
}

impl BackendAddress {
    /// Parse address from string
    pub fn parse(addr: &str) -> Result<Self, BackendAddressError> {
        let addr_inner = addr
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| BackendAddressError::InvalidFormat(addr.to_string()))?;
        Ok(BackendAddress(addr_inner))
    }
}

impl From<SocketAddr> for BackendAddress {
    fn from(value: SocketAddr) -> Self {
        Self(value)
    }
}

impl AsRef<SocketAddr> for BackendAddress {
    fn as_ref(&self) -> &SocketAddr {
        &self.0
    }
}

mod error {
    //! Backend address error module
    //!

    /// Backend address error enum
    #[derive(Debug, thiserror::Error)]
    pub enum BackendAddressError {
        /// Invalid address format
        #[error("backend address invalid format: {0}")]
        InvalidFormat(String),
        /// Backend address resolution failed
        #[error("backend address resolution failed: {0}")]
        ResolutionFailed(#[from] std::io::Error),
    }
}
