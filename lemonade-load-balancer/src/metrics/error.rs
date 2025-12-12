//! Metrics Error module
//!
/// Metrics error enum
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}
