//! Round Robin Strategy implementation
//!
use crate::{
    backend::BackendId,
    strategies::{Strategy, StrategyType, error::StrategyResult},
};

/// Round Robin Strategy selection implementation
pub struct RoundRobinStrategy;

impl Strategy for RoundRobinStrategy {
    fn strategy_type(&self) -> StrategyType {
        StrategyType::RoundRobin
    }

    async fn select_backend(&self) -> StrategyResult<BackendId> {
        todo!()
    }
}
