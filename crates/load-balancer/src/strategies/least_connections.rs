//! Least Connections Strategy implementation
//!
use crate::{backend::BackendId, error::Result, strategies::Strategy};

/// Least Connections Strategy selection implementation
pub struct LeastConnectionsStrategy;

impl Strategy for LeastConnectionsStrategy {
    async fn execute_strategy(&self) -> Result<BackendId> {
        todo!()
    }
}