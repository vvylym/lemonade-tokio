use std::net::SocketAddr;

use crate::error::Result;

/// Load Balancer Configuration
pub struct Config {
    /// Server configuration
    pub addr: SocketAddr,
    // TODO: probably more fields
}

impl Config {
    /// Parse configuration from environment variables
    ///
    pub fn parse() -> Result<Self> {
        // TODO: implement reading environment variables
        todo!()
    }
}
