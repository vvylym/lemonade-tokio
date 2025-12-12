//! Health models module
//!
use serde::{Deserialize, Serialize};

/// Health check response structure
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct HealthResponse {
    status: String,
    service: String,
}

impl HealthResponse {
    /// Create a new health check response
    #[must_use]
    pub fn new(status: impl Into<String>, service: impl Into<String>) -> Self {
        Self {
            status: status.into(),
            service: service.into(),
        }
    }

    /// Get the status
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Get the service name
    pub fn service(&self) -> &str {
        &self.service
    }
}
