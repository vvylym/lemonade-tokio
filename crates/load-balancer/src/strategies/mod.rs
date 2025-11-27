//! Strategies module
//!
mod adaptive;
mod least_connections;
mod round_robin;
mod weighted_round_robin;

pub use adaptive::AdaptiveStrategy;
pub use least_connections::LeastConnectionsStrategy;
pub use round_robin::RoundRobinStrategy;
pub use weighted_round_robin::WeightedRoundRobinStrategy;

use std::str::FromStr;
use crate::{
    backend::BackendId,
    error::{Error, Result},
};

/// Strategy Trait
/// 
pub trait Strategy: Send + Sync + 'static {
    /// Execute the strategy
    /// 
    fn execute_strategy(&self) -> impl Future<Output = Result<BackendId>> + Send;
}

/// Load balancing Strategy module
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyType {
    /// Adaptive: Distributes connections using any of the other Strategys for better performance.
    Adaptive,
    /// Round-robin: Distributes connections evenly by cycling through healthy backends in order.
    RoundRobin,
    /// Least connections: Routes to the backend with the fewest active connections.
    LeastConnections,
    /// Weighted round-robin: Distributes connections proportionally based on backend weights.
    WeightedRoundRobin,
}

impl FromStr for StrategyType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "round-robin" => Ok(Self::RoundRobin),
            "least-connections" => Ok(Self::LeastConnections),
            "weighted-round-robin" => Ok(Self::WeightedRoundRobin),
            "adaptive" => Ok(Self::Adaptive),
            _ => Err(Error::InvalidStrategy(s.into())),
        }
    }
}

impl TryFrom<u8> for StrategyType {
    type Error = crate::error::Error;
    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Adaptive),
            1 => Ok(Self::RoundRobin),
            2 => Ok(Self::LeastConnections),
            3 => Ok(Self::WeightedRoundRobin),
            _ => Err(Error::InvalidStrategy(format!("Numeric value: {}", value))),
        }
    }
}

impl From<StrategyType> for u8 {
    fn from(value: StrategyType) -> Self {
        match value {
            StrategyType::Adaptive => 0,
            StrategyType::RoundRobin => 1,
            StrategyType::LeastConnections => 2,
            StrategyType::WeightedRoundRobin => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_adaptive() {
        assert_eq!(StrategyType::try_from(0).unwrap(), StrategyType::Adaptive);
        assert_eq!(u8::from(StrategyType::Adaptive), 0);
        assert_eq!(
            StrategyType::from_str("adaptive").unwrap(),
            StrategyType::Adaptive
        );
    }

    #[test]
    fn test_strategy_round_robin() {
        assert_eq!(StrategyType::try_from(1).unwrap(), StrategyType::RoundRobin);
        assert_eq!(u8::from(StrategyType::RoundRobin), 1);
        assert_eq!(
            StrategyType::from_str("round-robin").unwrap(),
            StrategyType::RoundRobin
        );
    }

    #[test]
    fn test_strategy_least_connections() {
        assert_eq!(StrategyType::try_from(2).unwrap(), StrategyType::LeastConnections);
        assert_eq!(u8::from(StrategyType::LeastConnections), 2);
        assert_eq!(
            StrategyType::from_str("least-connections").unwrap(),
            StrategyType::LeastConnections
        );
    }

    #[test]
    fn test_strategy_weighted_round_robin() {
        assert_eq!(
            StrategyType::try_from(3).unwrap(),
            StrategyType::WeightedRoundRobin
        );
        assert_eq!(u8::from(StrategyType::WeightedRoundRobin), 3);
        assert_eq!(
            StrategyType::from_str("weighted-round-robin").unwrap(),
            StrategyType::WeightedRoundRobin
        );
    }

    #[test]
    fn test_strategy_invalid() {
        assert!(StrategyType::try_from(4).is_err());
        assert!(StrategyType::from_str("invalid").is_err());
    }
}
