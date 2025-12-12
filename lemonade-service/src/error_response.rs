//! Error response module
//!

use serde::{Deserialize, Serialize};

/// Error response struct
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorResponse {
    /// Error message
    message: String,
}

impl ErrorResponse {
    /// Create a new error response
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl<T: Into<String>> From<T> for ErrorResponse {
    fn from(message: T) -> Self {
        Self {
            message: message.into(),
        }
    }
}
