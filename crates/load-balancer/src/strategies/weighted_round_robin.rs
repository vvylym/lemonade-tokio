//! Weighted Round Robin Strategy implementation
//!
use crate::{
    backend::BackendId,
    strategies::{Strategy, StrategyType, error::StrategyResult},
};

/// Weighted Round Robin Strategy selection implementation
pub struct WeightedRoundRobinStrategy;

impl Strategy for WeightedRoundRobinStrategy {
    fn strategy_type(&self) -> StrategyType {
        StrategyType::WeightedRoundRobin
    }

    async fn select_backend(&self) -> StrategyResult<BackendId> {
        todo!()
    }
}
