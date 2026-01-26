//! Strategy builder module
//!
use crate::prelude::*;

/// Strategy builder struct
///
/// Strategies get backends from [`Context::routing_table()`] at runtime when
/// [`StrategyService::pick_backend`] is called; the builder only selects strategy type.
#[derive(Debug, Default)]
pub struct StrategyBuilder {
    /// Strategy
    strategy: Option<Strategy>,
}

impl StrategyBuilder {
    /// Create a new strategy builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the strategy
    pub fn with_strategy(mut self, strategy: Strategy) -> Self {
        self.strategy = Some(strategy);
        self
    }

    /// Build the strategy
    pub fn build(self) -> Result<Arc<dyn StrategyService>, StrategyError> {
        match self.strategy {
            Some(strategy) => match strategy {
                Strategy::Adaptive => Ok(Arc::new(AdaptiveStrategy::default())),
                Strategy::FastestResponseTime => {
                    Ok(Arc::new(FastestResponseTimeStrategy::default()))
                }
                Strategy::LeastConnections => {
                    Ok(Arc::new(LeastConnectionsStrategy::default()))
                }
                Strategy::RoundRobin => Ok(Arc::new(RoundRobinStrategy::default())),
                Strategy::WeightedRoundRobin => {
                    Ok(Arc::new(WeightedRoundRobinStrategy::default()))
                }
            },
            None => Err(StrategyError::NotFound("Strategy not found".to_string())),
        }
    }
}
