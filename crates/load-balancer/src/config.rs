use crate::error::Result;

/// Load Balancer Configuration
pub struct Config {
    /// Address of the load balancer
    pub address: String,
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
