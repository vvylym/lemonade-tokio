//! Load Balancing Algorithms module
//!
use crate::error::*;
use std::str::FromStr;

/// Load balancing algorithm module
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    /// Adaptive: Distributes connections using any of the other algorithms for better performance.
    Adaptive,
    /// Round-robin: Distributes connections evenly by cycling through healthy backends in order.
    RoundRobin,
    /// Least connections: Routes to the backend with the fewest active connections.
    LeastConnections,
    /// Weighted round-robin: Distributes connections proportionally based on backend weights.
    WeightedRoundRobin,
}

impl FromStr for Algorithm {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "round-robin" => Ok(Self::RoundRobin),
            "least-connections" => Ok(Self::LeastConnections),
            "weighted-round-robin" => Ok(Self::WeightedRoundRobin),
            "adaptive" => Ok(Self::Adaptive),
            _ => Err(Error::InvalidAlgorithm(s.into())),
        }
    }
}

impl TryFrom<u8> for Algorithm {
    type Error = crate::error::Error;
    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Adaptive),
            1 => Ok(Self::RoundRobin),
            2 => Ok(Self::LeastConnections),
            3 => Ok(Self::WeightedRoundRobin),
            _ => Err(Error::InvalidAlgorithm(format!("Numeric value: {}", value))),
        }
    }
}

impl From<Algorithm> for u8 {
    fn from(value: Algorithm) -> Self {
        match value {
            Algorithm::Adaptive => 0,
            Algorithm::RoundRobin => 1,
            Algorithm::LeastConnections => 2,
            Algorithm::WeightedRoundRobin => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_adaptive() {
        assert_eq!(Algorithm::try_from(0).unwrap(), Algorithm::Adaptive);
        assert_eq!(u8::from(Algorithm::Adaptive), 0);
        assert_eq!(
            Algorithm::from_str("adaptive").unwrap(),
            Algorithm::Adaptive
        );
    }

    #[test]
    fn test_algorithm_round_robin() {
        assert_eq!(Algorithm::try_from(1).unwrap(), Algorithm::RoundRobin);
        assert_eq!(u8::from(Algorithm::RoundRobin), 1);
        assert_eq!(
            Algorithm::from_str("round-robin").unwrap(),
            Algorithm::RoundRobin
        );
    }

    #[test]
    fn test_algorithm_least_connections() {
        assert_eq!(Algorithm::try_from(2).unwrap(), Algorithm::LeastConnections);
        assert_eq!(u8::from(Algorithm::LeastConnections), 2);
        assert_eq!(
            Algorithm::from_str("least-connections").unwrap(),
            Algorithm::LeastConnections
        );
    }

    #[test]
    fn test_algorithm_weighted_round_robin() {
        assert_eq!(
            Algorithm::try_from(3).unwrap(),
            Algorithm::WeightedRoundRobin
        );
        assert_eq!(u8::from(Algorithm::WeightedRoundRobin), 3);
        assert_eq!(
            Algorithm::from_str("weighted-round-robin").unwrap(),
            Algorithm::WeightedRoundRobin
        );
    }

    #[test]
    fn test_algorithm_invalid() {
        assert!(Algorithm::try_from(4).is_err());
        assert!(Algorithm::from_str("invalid").is_err());
    }
}
