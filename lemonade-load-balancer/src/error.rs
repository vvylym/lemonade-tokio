//! Error module
//!
use crate::prelude::*;

/// Result type alias for Lemonade errors.
pub type Result<T> = std::result::Result<T, Error>;

/// Enum representing possible errors in the Lemonade load balancer.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Config error
    #[error("config error: {0}")]
    Config(#[from] ConfigError),
    /// Health error
    #[error("health error: {0}")]
    Health(#[from] HealthError),
    /// Metrics error
    #[error("metrics error: {0}")]
    Metrics(#[from] MetricsError),
    /// Proxy error
    #[error("proxy error: {0}")]
    Proxy(#[from] ProxyError),
    /// State error
    #[error("state error: {0}")]
    State(#[from] ContextError),
    /// Strategy error
    #[error("strategy error: {0}")]
    Strategy(#[from] StrategyError),
}
