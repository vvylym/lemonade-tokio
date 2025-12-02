//! Adaptive Strategy implementation
//!
use crate::{
    backend::BackendId,
    strategies::{Strategy, StrategyType, error::StrategyResult},
};

/// Adaptive Strategy selection implementation]
pub struct AdaptiveStrategy;

impl Strategy for AdaptiveStrategy {
    fn strategy_type(&self) -> StrategyType {
        StrategyType::Adaptive
    }

    async fn select_backend(&self) -> StrategyResult<BackendId> {
        todo!()
    }
}
