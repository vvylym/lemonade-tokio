//! Health Port module
//!

use super::{error::HealthError, models::HealthResponse};
use async_trait::async_trait;

/// Health service trait
#[async_trait]
pub trait HealthService: Send + Sync + 'static {
    /// Perform a health check
    async fn health_check(&self) -> Result<HealthResponse, HealthError>;
}
