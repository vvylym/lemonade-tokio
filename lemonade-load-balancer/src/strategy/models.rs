//! Strategy models module
//!
use super::constants::*;
use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// Strategy enum
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    /// Adaptive strategy
    Adaptive,
    /// Fastest response time strategy
    FastestResponseTime,
    /// Least connections strategy
    LeastConnections,
    /// Round robin strategy
    RoundRobin,
    /// Weighted round robin strategy
    WeightedRoundRobin,
}

impl std::str::FromStr for Strategy {
    type Err = StrategyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            STRATEGY_ADAPTIVE => Ok(Strategy::Adaptive),
            STRATEGY_FASTEST_RESPONSE_TIME => Ok(Strategy::FastestResponseTime),
            STRATEGY_LEAST_CONNECTIONS => Ok(Strategy::LeastConnections),
            STRATEGY_ROUND_ROBIN => Ok(Strategy::RoundRobin),
            STRATEGY_WEIGHTED_ROUND_ROBIN => Ok(Strategy::WeightedRoundRobin),
            _ => Err(StrategyError::NotFound(s.to_string())),
        }
    }
}

impl AsRef<str> for Strategy {
    fn as_ref(&self) -> &str {
        match self {
            Strategy::Adaptive => STRATEGY_ADAPTIVE,
            Strategy::FastestResponseTime => STRATEGY_FASTEST_RESPONSE_TIME,
            Strategy::LeastConnections => STRATEGY_LEAST_CONNECTIONS,
            Strategy::RoundRobin => STRATEGY_ROUND_ROBIN,
            Strategy::WeightedRoundRobin => STRATEGY_WEIGHTED_ROUND_ROBIN,
        }
    }
}
