//! Proxy Error module
//!

/// Proxy error enum
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    /// Connection refused error
    #[error("tcp stream error: {0}")]
    Io(#[from] tokio::io::Error),
    /// Unexpected error
    #[error("unexpected error: {0}")]
    Unexpected(String),
}
