use std::str::FromStr;
use crate::error::{Error, Result};

/// Load Balancer Configuration
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
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

/// Load Balancer Configuration
pub struct ServerConfig {
    /// Host of the load balancer
    pub host: String,
    /// Port of the load balancer
    pub port: u16,
}

impl FromStr for ServerConfig {
    type Err = Error;

    fn from_str(addr: &str) -> Result<Self> {
        let addr_trimmed = addr.trim();
        if addr_trimmed.is_empty() {
            return Err(Error::Config("server address cannot be empty".into()));
        }
        let idx = addr_trimmed.rfind(":")
            .ok_or_else(|| Error::Config(format!("invalid server address: {}", addr_trimmed)))?;

        let (host_part, port_part) = addr_trimmed.split_at(idx);

        if host_part.is_empty() {
            return Err(Error::Config(format!("invalid server host value for address: {}", addr_trimmed)));
        }

        let port = port_part[1..].parse::<u16>().map_err(|_| {
            Error::Config(format!("invalid server port value for address: {}", addr_trimmed))
        })?;

        Ok(ServerConfig { host: host_part.to_string(), port })
    }
}

impl std::fmt::Display for ServerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_display() {
        let config = ServerConfig { host: "127.0.0.1".to_string(), port: 8080 };
        assert_eq!(config.to_string(), "127.0.0.1:8080");
    }

    #[test]
    fn test_server_config_valid_address() {
        let addr = "127.0.0.1:8080";
        let config = ServerConfig::from_str(addr).unwrap();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_server_config_invalid_address_with_empty_host() {
        let addr = ":8080";
        let result = ServerConfig::from_str(addr);
        assert!(result.is_err());
    }

    #[test]
    fn test_server_config_invalid_address_with_empty_port() {
        let addr = "127.0.0.1:";
        let result = ServerConfig::from_str(addr);
        assert!(result.is_err());
    }

    #[test]
    fn test_server_config_invalid_address_with_invalid_port() {
        let addr = "127.0.0.1:invalid";
        let result = ServerConfig::from_str(addr);
        assert!(result.is_err());
    }

    #[test]
    fn test_server_config_invalid_address_with_port_too_large() {
        let addr = "127.0.0.1:65536";
        let result = ServerConfig::from_str(addr);
        assert!(result.is_err());
    }
}