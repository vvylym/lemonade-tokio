//! Strategy Error module
//!

/// Strategy error enum
#[derive(Debug, thiserror::Error)]
pub enum StrategyError {
    /// Strategy not found
    #[error("strategy not found: {0}")]
    NotFound(String),
    /// No backend available
    #[error("no backend available")]
    NoBackendAvailable,
    /// Unexpected error
    #[error("unexpected error: {0}")]
    UnexpectedError(String),
}
