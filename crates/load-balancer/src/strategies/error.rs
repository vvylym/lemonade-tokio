//! Strategies error module
//!
use core::result::Result;
use thiserror::Error;

/// Strategy result type
pub type StrategyResult<T> = Result<T, StrategyError>;

/// Strategy error type
#[derive(Debug, Error)]
pub enum StrategyError {
    /// Invalid strategy type
    #[error("Invalid strategy type: {0}")]
    InvalidStrategyType(String),
}
