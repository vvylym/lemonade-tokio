//! Work error module
//!

/// Work error struct
#[derive(Debug, thiserror::Error)]
#[error("health error: {0}")]
pub struct WorkError(String);

impl WorkError {
    /// Create a new work error
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}
