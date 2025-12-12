//! Adaptive strategy constants
//!

/// Default weight for connection load factor in adaptive scoring
pub const DEFAULT_CONN_WEIGHT: f64 = 0.4;

/// Default weight for latency factor in adaptive scoring
pub const DEFAULT_LATENCY_WEIGHT: f64 = 0.4;

/// Default weight for error rate factor in adaptive scoring
pub const DEFAULT_ERROR_WEIGHT: f64 = 0.2;

/// Default maximum latency in milliseconds (used when no metrics available)
pub const DEFAULT_MAX_LATENCY_MS: f64 = 1000.0;

/// Error rate threshold for additional penalty (10%)
pub const ERROR_RATE_THRESHOLD: f32 = 0.1;

/// Error rate penalty multiplier for high error rates
pub const HIGH_ERROR_RATE_PENALTY: f64 = 0.5;

/// Minimum weight factor to avoid division by zero
pub const MIN_WEIGHT_FACTOR: f64 = 0.1;

/// Default cache TTL in milliseconds
pub const DEFAULT_CACHE_TTL_MS: u64 = 100;

/// Default backend weight when not specified
pub const DEFAULT_BACKEND_WEIGHT: u8 = 1;

/// Minimum connection count to avoid division by zero
pub const MIN_CONNECTION_COUNT: usize = 1;

/// Zero value for floating point comparisons
pub const ZERO_F64: f64 = 0.0;

/// Unit value for weight factor when no weights are specified
pub const UNIT_WEIGHT_FACTOR: f64 = 1.0;

/// Maximum variance penalty multiplier (caps penalty at 2x)
pub const MAX_VARIANCE_PENALTY: f64 = 1.0;

/// Unit value for variance penalty when no variance
pub const NO_VARIANCE_PENALTY: f64 = 1.0;

/// Perfect error rate (no errors)
pub const PERFECT_ERROR_RATE: f32 = 0.0;

/// Invalid backend index (used when backend not found)
pub const INVALID_BACKEND_INDEX: usize = usize::MAX;
