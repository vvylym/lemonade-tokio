//! Round Robin Strategy implementation
//!
use crate::{backend::BackendId, error::Result, strategies::Strategy};

/// Round Robin Strategy selection implementation
pub struct RoundRobinStrategy;

impl Strategy for RoundRobinStrategy {
    async fn execute_strategy(&self) -> Result<BackendId> {
        todo!()
    }
}
