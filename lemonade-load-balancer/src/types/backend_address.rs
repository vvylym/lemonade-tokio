//! Backend address module
//!
pub use error::BackendAddressError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::net::{SocketAddr, ToSocketAddrs};

/// Backend address struct
///
/// Supports both IP addresses and hostnames. Hostnames are resolved lazily
/// at connection time, not during config parsing, allowing Docker service names
/// to be used even if DNS isn't ready when the config is loaded.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BackendAddress(String);

impl Serialize for BackendAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for BackendAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // Validate format but don't resolve hostnames yet
        BackendAddress::parse(&s).map_err(serde::de::Error::custom)
    }
}

impl BackendAddress {
    /// Parse address from string
    ///
    /// Validates the format (must contain a colon for port) but doesn't
    /// resolve hostnames. Resolution happens lazily when ToSocketAddrs is used.
    pub fn parse(addr: &str) -> Result<Self, BackendAddressError> {
        // Basic format validation: must contain ':' for port
        if !addr.contains(':') {
            return Err(BackendAddressError::InvalidFormat(addr.to_string()));
        }

        // Try to parse as SocketAddr first (for IP addresses)
        // If it fails, assume it's a hostname and store as-is
        // This allows immediate validation of IP addresses while deferring
        // hostname resolution to connection time
        if let Ok(socket_addr) = addr.parse::<SocketAddr>() {
            // It's a valid IP:port, store it
            Ok(BackendAddress(socket_addr.to_string()))
        } else {
            // Assume it's a hostname:port, store as-is for lazy resolution
            Ok(BackendAddress(addr.to_string()))
        }
    }

    /// Get the address string (hostname or IP:port)
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<SocketAddr> for BackendAddress {
    fn from(value: SocketAddr) -> Self {
        Self(value.to_string())
    }
}

impl ToSocketAddrs for BackendAddress {
    type Iter = std::vec::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        // Resolve hostname or parse IP address at connection time
        self.0
            .to_socket_addrs()
            .map(|iter| iter.collect::<Vec<_>>().into_iter())
    }
}

impl AsRef<str> for BackendAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for BackendAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
