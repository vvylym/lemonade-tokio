//! Weighted Round Robin Strategy implementation
//!
use crate::{backend::BackendId, error::Result, strategies::Strategy};

/// Weighted Round Robin Strategy selection implementation
pub struct WeightedRoundRobinStrategy;

impl Strategy for WeightedRoundRobinStrategy {
    async fn execute_strategy(&self) -> Result<BackendId> {
        todo!()
    }
}