//! Adaptive strategy models module
//!
use crate::prelude::*;

/// Configuration for adaptive strategy scoring weights
///
/// These weights determine the relative importance of each factor
/// in the adaptive scoring algorithm. All weights should sum to 1.0
/// for optimal results, though this is not enforced.
#[derive(Debug, Clone)]
pub struct AdaptiveWeights {
    /// Weight for connection load factor (0.0-1.0)
    /// Higher values prioritize backends with fewer connections
    pub conn_weight: f64,
    /// Weight for latency factor (0.0-1.0)
    /// Higher values prioritize backends with lower latency
    pub latency_weight: f64,
    /// Weight for error rate factor (0.0-1.0)
    /// Higher values penalize backends with higher error rates
    pub error_weight: f64,
}

impl Default for AdaptiveWeights {
    fn default() -> Self {
        use super::constants::*;
        Self {
            conn_weight: DEFAULT_CONN_WEIGHT,
            latency_weight: DEFAULT_LATENCY_WEIGHT,
            error_weight: DEFAULT_ERROR_WEIGHT,
        }
    }
}

/// Scoring context containing normalized maximum values
///
/// This context is prepared once per backend selection and contains
/// all the necessary data for computing scores without repeated lookups.
pub struct ScoringContext {
    /// Maximum connection count across all backends (for normalization)
    pub max_connections: usize,
    /// Maximum latency across all backends (for normalization)
    pub max_latency_ms: f64,
    /// Maximum weight across all backends (for normalization)
    pub max_weight: f64,
    /// Routing table for looking up backends
    pub routing: Arc<RouteTable>,
}
