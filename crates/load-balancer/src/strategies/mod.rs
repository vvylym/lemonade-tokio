//! Strategies module
//!
mod constants;
pub mod adaptive;
pub mod error;
pub mod least_connections;
pub mod round_robin;
pub mod weighted_round_robin;

use std::str::FromStr;

use crate::backend::BackendId;
use error::{StrategyError, StrategyResult};

use constants::{
    STRATEGY_ADAPTIVE, STRATEGY_LEAST_CONNECTIONS, STRATEGY_ROUND_ROBIN,
    STRATEGY_WEIGHTED_ROUND_ROBIN,
};

/// Strategy Trait
///
pub trait Strategy: Send + Sync + 'static {
    /// Strategy type
    ///
    fn strategy_type(&self) -> StrategyType;

    /// Select the backend for a connection
    ///
    fn select_backend(&self) -> impl Future<Output = StrategyResult<BackendId>> + Send;
}

/// Load balancing Strategy module
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyType {
    /// Adaptive: Distributes connections using any of the other strategies for better performance.
    ///
    /// This strategy is used to distribute connections using any of the other strategies for better performance.
    Adaptive,
    /// Round-robin: Distributes connections evenly by cycling through healthy backends in order.
    ///
    /// This strategy is used to distribute connections evenly by cycling through healthy backends in order.
    RoundRobin,
    /// Least connections: Routes to the backend with the fewest active connections.
    ///
    /// This strategy is used to route to the backend with the fewest active connections.
    LeastConnections,
    /// Weighted round-robin: Distributes connections proportionally based on backend weights.
    ///
    /// This strategy is used to distribute connections proportionally based on backend weights.
    WeightedRoundRobin,
}

impl FromStr for StrategyType {
    type Err = StrategyError;

    fn from_str(s: &str) -> StrategyResult<Self> {
        match s {
            STRATEGY_ROUND_ROBIN => Ok(Self::RoundRobin),
            STRATEGY_LEAST_CONNECTIONS => Ok(Self::LeastConnections),
            STRATEGY_WEIGHTED_ROUND_ROBIN => Ok(Self::WeightedRoundRobin),
            STRATEGY_ADAPTIVE => Ok(Self::Adaptive),
            _ => Err(StrategyError::InvalidStrategyType(s.into())),
        }
    }
}

impl AsRef<str> for StrategyType {
    fn as_ref(&self) -> &str {
        match self {
            StrategyType::Adaptive => STRATEGY_ADAPTIVE,
            StrategyType::RoundRobin => STRATEGY_ROUND_ROBIN,
            StrategyType::LeastConnections => STRATEGY_LEAST_CONNECTIONS,
            StrategyType::WeightedRoundRobin => STRATEGY_WEIGHTED_ROUND_ROBIN,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_adaptive() {
        assert_eq!(
            StrategyType::from_str(STRATEGY_ADAPTIVE).unwrap(),
            StrategyType::Adaptive
        );
    }

    #[test]
    fn test_strategy_round_robin() {
        assert_eq!(
            StrategyType::from_str(STRATEGY_ROUND_ROBIN).unwrap(),
            StrategyType::RoundRobin
        );
    }

    #[test]
    fn test_strategy_least_connections() {
        assert_eq!(
            StrategyType::from_str(STRATEGY_LEAST_CONNECTIONS).unwrap(),
            StrategyType::LeastConnections
        );
    }

    #[test]
    fn test_strategy_weighted_round_robin() {
        assert_eq!(
            StrategyType::from_str(STRATEGY_WEIGHTED_ROUND_ROBIN).unwrap(),
            StrategyType::WeightedRoundRobin
        );
    }

    #[test]
    fn test_strategy_invalid() {
        assert!(StrategyType::from_str("invalid").is_err());
    }
}
