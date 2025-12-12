//! Health error module
//!

/// Health error struct
#[derive(Debug, thiserror::Error)]
#[error("health error: {0}")]
pub struct HealthError(String);

impl HealthError {
    /// Create a new health error
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}
