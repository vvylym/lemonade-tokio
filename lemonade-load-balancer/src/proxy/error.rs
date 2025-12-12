//! Proxy Error module
//!

/// Proxy error enum
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    /// IO error
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// Unexpected error
    #[error("unexpected error: {0}")]
    Unexpected(String),
}
