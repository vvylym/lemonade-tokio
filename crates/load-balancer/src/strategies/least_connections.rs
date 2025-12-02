//! Least Connections Strategy implementation
//!
use crate::{
    backend::BackendId,
    strategies::{Strategy, StrategyType, error::StrategyResult},
};

/// Least Connections Strategy selection implementation
#[allow(unused)]
pub struct LeastConnectionsStrategy;

impl Strategy for LeastConnectionsStrategy {
    fn strategy_type(&self) -> StrategyType {
        StrategyType::LeastConnections
    }

    async fn select_backend(&self) -> StrategyResult<BackendId> {
        todo!()
    }
}
