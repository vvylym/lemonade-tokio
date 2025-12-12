//! Work Port module
//!
use super::{error::WorkError, models::WorkResponse};
use async_trait::async_trait;

/// Worker service trait
#[async_trait]
pub trait WorkService: Send + Sync + 'static {
    /// Perform a work
    async fn work(&self) -> Result<WorkResponse, WorkError>;
}
