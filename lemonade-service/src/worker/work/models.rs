//! Work models module
//!
use serde::{Deserialize, Serialize};

/// Work response structure
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct WorkResponse {
    /// Status
    status: bool,
    /// Service name
    service: String,
    /// Duration in milliseconds
    duration_ms: u64,
}

impl WorkResponse {
    /// Create a new work response
    #[must_use]
    pub fn new(status: bool, service: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            status,
            service: service.into(),
            duration_ms,
        }
    }

    /// Get the status
    #[must_use]
    pub fn status(&self) -> bool {
        self.status
    }

    /// Get the service name
    #[must_use]
    pub fn service(&self) -> &str {
        &self.service
    }

    /// Get the duration in milliseconds
    #[must_use]
    pub fn duration_ms(&self) -> u64 {
        self.duration_ms
    }
}
