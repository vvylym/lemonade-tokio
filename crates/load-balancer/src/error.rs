//! Error module for the load balancer
//!

/// Result type alias for load balancer operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types that can occur in the load balancer
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Network I/O error
    #[error("I/O error: {0}")]
    Io(std::io::Error),
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
    /// Backend selection error (e.g., no healthy backends available)
    #[error("Backend selection error: {0}")]
    BackendSelection(String),
    /// Connection timeout
    #[error("Timeout error: {0}")]
    Timeout(String),
    /// Health check error
    #[error("Healtch check error: {0}")]
    HealthCheck(String),
    /// Metrics error
    #[error("Metrics error: {0}")]
    Metrics(String),
    /// Backend ID is required but was not provided
    #[error("Missing backend id")]
    MissingBackendId,
    /// Backend address is required but was not provided
    #[error("Missing backend address")]
    MissingBackendAddress,
    /// Environment variable not found
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(&'static str),
    /// Environment variable has invalid format
    #[error("Invalid environment variable: {0}")]
    InvalidEnvVarFormat(&'static str),
    /// Invalid load balancing algorithm
    #[error("Invalid Algorith: {0}")]
    InvalidStrategy(String),
}
