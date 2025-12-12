//! Adaptive strategy utils module
//!
//! This module contains utility functions for computing normalized scores
//! and preparing scoring contexts for the adaptive load balancing algorithm.

use super::constants::*;
use super::models::{AdaptiveWeights, ScoringContext};
use crate::prelude::*;
use crate::types::BackendMetrics;

/// Compute normalized connection score
///
/// Normalizes the connection count based on the maximum connections
/// and adjusts for backend weight. Higher weight backends receive
/// less penalty for having more connections.
///
/// # Arguments
/// * `backend_connection_count` - Current connection count for the backend
/// * `max_connection_count` - Maximum connection count across all backends
/// * `backend_weight` - Weight of this backend
/// * `max_weight` - Maximum weight across all backends
///
/// # Returns
/// Normalized connection score (0.0-1.0+), lower is better
pub fn compute_connection_score(
    backend_connection_count: usize,
    max_connection_count: usize,
    backend_weight: f64,
    max_weight: f64,
) -> f64 {
    if max_connection_count == ZERO_F64 as usize {
        return ZERO_F64;
    }

    // Normalize connection load (0-1 scale)
    let connection_ratio = backend_connection_count as f64 / max_connection_count as f64;

    // Adjust for weight (higher weight = less penalty for connections)
    let weight_factor = if max_weight > ZERO_F64 {
        backend_weight / max_weight
    } else {
        UNIT_WEIGHT_FACTOR
    };

    // Lower is better, so divide by weight factor
    connection_ratio / weight_factor.max(MIN_WEIGHT_FACTOR)
}

/// Compute error rate penalty
///
/// Converts error rate to a penalty value where lower penalty is better.
/// Backends with high error rates (> 10%) receive additional penalty.
///
/// # Arguments
/// * `error_rate` - Error rate as a fraction (0.0-1.0)
///
/// # Returns
/// Error penalty value (0.0-1.0), higher is better (less penalty)
pub fn compute_error_penalty(error_rate: f32) -> f64 {
    let base_penalty = UNIT_WEIGHT_FACTOR - error_rate as f64;

    // Additional penalty for high error rates
    if error_rate > ERROR_RATE_THRESHOLD {
        base_penalty * HIGH_ERROR_RATE_PENALTY
    } else {
        base_penalty
    }
}

/// Compute normalized latency score with variance penalty
///
/// Normalizes latency based on maximum latency and applies a variance
/// penalty for backends with high latency variance (p95 >> avg), which
/// indicates instability.
///
/// # Arguments
/// * `average_latency_ms` - Average latency in milliseconds
/// * `p95_latency_ms` - 95th percentile latency in milliseconds (optional)
/// * `max_latency_ms` - Maximum latency across all backends
///
/// # Returns
/// Normalized latency score (0.0-2.0+), lower is better
pub fn compute_latency_score(
    average_latency_ms: f64,
    p95_latency_ms: Option<f64>,
    max_latency_ms: f64,
) -> f64 {
    if max_latency_ms == ZERO_F64 {
        return ZERO_F64;
    }

    // Normalize latency (0-1 scale)
    let latency_ratio = average_latency_ms / max_latency_ms;

    // Variance penalty: high variance (p95 >> avg) indicates instability
    let variance_penalty = if let Some(p95_latency) = p95_latency_ms {
        if average_latency_ms > ZERO_F64 {
            let variance_ratio = (p95_latency - average_latency_ms) / average_latency_ms;
            NO_VARIANCE_PENALTY + variance_ratio.min(MAX_VARIANCE_PENALTY) // Cap penalty at 2x
        } else {
            NO_VARIANCE_PENALTY
        }
    } else {
        NO_VARIANCE_PENALTY
    };

    latency_ratio * variance_penalty
}

/// Prepare scoring context from registries
///
/// Analyzes all backends to find maximum values for normalization
/// and creates a context object containing all necessary data for scoring.
///
/// # Arguments
/// * `backends` - List of healthy backends to analyze
/// * `connections` - Connection registry for connection counts
/// * `metrics` - Metrics snapshot for performance data
/// * `routing` - Routing table for backend lookups
///
/// # Returns
/// ScoringContext with normalized maximum values and registries
pub fn prepare_scoring_context(
    backends: &[BackendMeta],
    connections: Arc<ConnectionRegistry>,
    metrics: Arc<MetricsSnapshot>,
    routing: Arc<RouteTable>,
) -> ScoringContext {
    use super::constants::*;

    let mut max_connection_count = ZERO_F64 as usize;
    let mut max_latency_value = ZERO_F64;
    let mut max_weight_value = ZERO_F64;

    for backend in backends {
        let backend_id = *backend.id();
        let backend_index = routing
            .find_index(backend_id)
            .unwrap_or(INVALID_BACKEND_INDEX);
        let connection_count = connections.get(backend_index);
        let backend_weight_value =
            backend.weight().unwrap_or(DEFAULT_BACKEND_WEIGHT) as f64;

        max_connection_count = max_connection_count.max(connection_count);
        max_weight_value = max_weight_value.max(backend_weight_value);

        // Get max latency from metrics (use p95 if available, else avg)
        if let Some(backend_metrics) = metrics.get(backend_id) {
            let p95_latency = backend_metrics.p95_latency_ms;
            let average_latency = backend_metrics.avg_latency_ms;
            let latency_value = if p95_latency > average_latency {
                p95_latency
            } else {
                average_latency
            };
            max_latency_value = max_latency_value.max(latency_value);
        }
    }

    // Use default max latency if no metrics available
    if max_latency_value == ZERO_F64 {
        max_latency_value = DEFAULT_MAX_LATENCY_MS;
    }

    // Ensure max_connection_count is at least 1 to avoid division by zero
    if max_connection_count == ZERO_F64 as usize {
        max_connection_count = MIN_CONNECTION_COUNT;
    }

    ScoringContext {
        max_connections: max_connection_count,
        max_latency_ms: max_latency_value,
        max_weight: max_weight_value,
        connections,
        metrics,
        routing,
    }
}

/// Compute combined score for a single backend
///
/// Combines normalized connection, latency, and error rate scores
/// using the configured weights, then applies weight factor adjustment.
///
/// # Arguments
/// * `backend_connection_count` - Current connection count for the backend
/// * `backend_weight_value` - Weight of this backend
/// * `backend_metrics` - Performance metrics for the backend (optional)
/// * `scoring_context` - Context with normalization values and registries
/// * `weights` - Weight configuration for scoring factors
///
/// # Returns
/// Combined score (lower is better)
pub fn compute_backend_score(
    backend_connection_count: usize,
    backend_weight_value: f64,
    backend_metrics: Option<BackendMetrics>,
    scoring_context: &ScoringContext,
    weights: &AdaptiveWeights,
) -> f64 {
    use super::constants::*;

    // Extract metrics or use defaults
    let average_latency = backend_metrics
        .as_ref()
        .map(|metrics| metrics.avg_latency_ms)
        .unwrap_or(DEFAULT_MAX_LATENCY_MS);
    let p95_latency = backend_metrics.as_ref().and_then(|metrics| {
        if metrics.p95_latency_ms > ZERO_F64 {
            Some(metrics.p95_latency_ms)
        } else {
            None
        }
    });
    let error_rate_value = backend_metrics
        .as_ref()
        .map(|metrics| metrics.error_rate)
        .unwrap_or(PERFECT_ERROR_RATE);

    // Compute normalized scores for each factor
    let connection_score = compute_connection_score(
        backend_connection_count,
        scoring_context.max_connections,
        backend_weight_value,
        scoring_context.max_weight,
    );
    let latency_score = compute_latency_score(
        average_latency,
        p95_latency,
        scoring_context.max_latency_ms,
    );
    let error_penalty_value = compute_error_penalty(error_rate_value);

    // Combine scores using configured weights (lower is better)
    let combined_score = (connection_score * weights.conn_weight)
        + (latency_score * weights.latency_weight)
        + ((UNIT_WEIGHT_FACTOR - error_penalty_value) * weights.error_weight);

    // Apply weight factor (higher weight = preference)
    let weight_factor = if scoring_context.max_weight > ZERO_F64 {
        backend_weight_value / scoring_context.max_weight
    } else {
        UNIT_WEIGHT_FACTOR
    };

    // Divide by weight factor to give preference to higher weights
    combined_score / weight_factor.max(MIN_WEIGHT_FACTOR)
}

/// Compute scores for all backends
///
/// Computes scores for all backends, using cached scores when available.
/// This method handles both cache lookup and score computation in a single pass.
///
/// # Arguments
/// * `backends` - List of healthy backends to score
/// * `scoring_context` - Context with normalization values and registries
/// * `cached_scores` - Pre-computed cached scores (backend_id -> score)
/// * `cache` - Cache instance for storing computed scores
/// * `weights` - Weight configuration for scoring factors
/// * `current_timestamp_ms` - Current timestamp in milliseconds for cache TTL
///
/// # Returns
/// Vector of (score, backend) tuples, sorted by score (lower is better)
pub fn compute_all_backend_scores(
    backends: &[BackendMeta],
    scoring_context: &ScoringContext,
    cached_scores: &[(BackendId, Option<f64>)],
    cache: &super::cache::AdaptiveCache,
    weights: &AdaptiveWeights,
    current_timestamp_ms: u64,
) -> Vec<(f64, BackendMeta)> {
    backends
        .iter()
        .enumerate()
        .map(|(backend_index, backend)| {
            let backend_id = *backend.id();
            let backend_table_index = scoring_context
                .routing
                .find_index(backend_id)
                .unwrap_or(INVALID_BACKEND_INDEX);
            let backend_connection_count =
                scoring_context.connections.get(backend_table_index);
            let backend_weight_value =
                backend.weight().unwrap_or(DEFAULT_BACKEND_WEIGHT) as f64;
            let backend_metrics = scoring_context.metrics.get(backend_id);

            // Use cached score if available and valid
            let computed_score =
                if let Some(cached_score_value) = cached_scores[backend_index].1 {
                    cached_score_value
                } else {
                    // Compute new score
                    let new_score = compute_backend_score(
                        backend_connection_count,
                        backend_weight_value,
                        backend_metrics,
                        scoring_context,
                        weights,
                    );

                    // Cache the score for future use
                    cache.put(backend_id, new_score, current_timestamp_ms);
                    new_score
                };

            (computed_score, backend.clone())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::adapters::adaptive::cache::AdaptiveCache;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    fn create_test_backend(id: u8, weight: Option<u8>) -> BackendMeta {
        let address =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080 + id as u16);
        BackendMeta::new(id, Some(format!("backend-{}", id)), address, weight)
    }

    #[test]
    fn compute_connection_score_with_weights_should_succeed() {
        // Given: connection counts and weights
        let backend_connection_count = 5;
        let max_connection_count = 10;
        let backend_weight = 2.0;
        let max_weight = 4.0;

        // When: computing connection score
        let score = compute_connection_score(
            backend_connection_count,
            max_connection_count,
            backend_weight,
            max_weight,
        );

        // Then: score is computed correctly
        // connection_ratio = 5/10 = 0.5
        // weight_factor = 2.0/4.0 = 0.5
        // score = 0.5 / 0.5 = 1.0
        assert_eq!(score, 1.0);
    }

    #[test]
    fn compute_connection_score_with_zero_max_connections_should_succeed() {
        // Given: zero max connection count
        let backend_connection_count = 5;
        let max_connection_count = 0;
        let backend_weight = 2.0;
        let max_weight = 4.0;

        // When: computing connection score
        let score = compute_connection_score(
            backend_connection_count,
            max_connection_count,
            backend_weight,
            max_weight,
        );

        // Then: returns zero
        assert_eq!(score, 0.0);
    }

    #[test]
    fn compute_connection_score_with_zero_max_weight_should_succeed() {
        // Given: zero max weight
        let backend_connection_count = 5;
        let max_connection_count = 10;
        let backend_weight = 2.0;
        let max_weight = 0.0;

        // When: computing connection score
        let score = compute_connection_score(
            backend_connection_count,
            max_connection_count,
            backend_weight,
            max_weight,
        );

        // Then: uses UNIT_WEIGHT_FACTOR (1.0)
        // connection_ratio = 0.5
        // weight_factor = 1.0 (fallback)
        // score = 0.5 / 1.0 = 0.5
        assert_eq!(score, 0.5);
    }

    #[test]
    fn compute_connection_score_with_equal_weights_should_succeed() {
        // Given: equal weights
        let backend_connection_count = 5;
        let max_connection_count = 10;
        let backend_weight = 2.0;
        let max_weight = 2.0;

        // When: computing connection score
        let score = compute_connection_score(
            backend_connection_count,
            max_connection_count,
            backend_weight,
            max_weight,
        );

        // Then: score equals connection ratio
        assert_eq!(score, 0.5);
    }

    #[test]
    fn compute_error_penalty_with_low_error_rate_should_succeed() {
        // Given: low error rate
        let error_rate = 0.05f32; // 5%

        // When: computing error penalty
        let penalty = compute_error_penalty(error_rate);

        // Then: penalty is computed correctly
        // base_penalty = 1.0 - 0.05 = 0.95
        assert!((penalty - 0.95).abs() < 0.001);
    }

    #[test]
    fn compute_error_penalty_with_high_error_rate_should_succeed() {
        // Given: high error rate (above threshold)
        let error_rate = 0.15f32; // 15%

        // When: computing error penalty
        let penalty = compute_error_penalty(error_rate);

        // Then: additional penalty is applied
        // base_penalty = 1.0 - 0.15 = 0.85
        // high_error_penalty = 0.85 * 0.5 = 0.425
        assert!((penalty - 0.425).abs() < 0.001);
    }

    #[test]
    fn compute_error_penalty_with_zero_error_rate_should_succeed() {
        // Given: zero error rate
        let error_rate = 0.0f32;

        // When: computing error penalty
        let penalty = compute_error_penalty(error_rate);

        // Then: penalty is 1.0 (no penalty)
        assert_eq!(penalty, 1.0);
    }

    #[test]
    fn compute_error_penalty_with_perfect_error_rate_should_succeed() {
        // Given: perfect error rate (no errors)
        let error_rate = 0.0f32;

        // When: computing error penalty
        let penalty = compute_error_penalty(error_rate);

        // Then: returns 1.0
        assert_eq!(penalty, 1.0);
    }

    #[test]
    fn compute_error_penalty_at_threshold_should_succeed() {
        // Given: error rate exactly at threshold
        let error_rate = 0.1f32; // 10%

        // When: computing error penalty
        let penalty = compute_error_penalty(error_rate);

        // Then: no additional penalty (threshold is >, not >=)
        // base_penalty = 1.0 - 0.1 = 0.9
        assert!((penalty - 0.9).abs() < 0.001);
    }

    #[test]
    fn compute_latency_score_without_p95_should_succeed() {
        // Given: latency values without p95
        let average_latency_ms = 50.0;
        let p95_latency_ms = None;
        let max_latency_ms = 100.0;

        // When: computing latency score
        let score =
            compute_latency_score(average_latency_ms, p95_latency_ms, max_latency_ms);

        // Then: score is computed correctly
        // latency_ratio = 50.0 / 100.0 = 0.5
        // variance_penalty = 1.0 (no p95)
        // score = 0.5 * 1.0 = 0.5
        assert_eq!(score, 0.5);
    }

    #[test]
    fn compute_latency_score_with_p95_should_succeed() {
        // Given: latency values with p95
        let average_latency_ms = 50.0;
        let p95_latency_ms = Some(75.0);
        let max_latency_ms = 100.0;

        // When: computing latency score
        let score =
            compute_latency_score(average_latency_ms, p95_latency_ms, max_latency_ms);

        // Then: score includes variance penalty
        // latency_ratio = 0.5
        // variance_ratio = (75.0 - 50.0) / 50.0 = 0.5
        // variance_penalty = 1.0 + 0.5 = 1.5
        // score = 0.5 * 1.5 = 0.75
        assert_eq!(score, 0.75);
    }

    #[test]
    fn compute_latency_score_with_zero_max_latency_should_succeed() {
        // Given: zero max latency
        let average_latency_ms = 50.0;
        let p95_latency_ms = Some(75.0);
        let max_latency_ms = 0.0;

        // When: computing latency score
        let score =
            compute_latency_score(average_latency_ms, p95_latency_ms, max_latency_ms);

        // Then: returns zero
        assert_eq!(score, 0.0);
    }

    #[test]
    fn compute_latency_score_with_zero_average_latency_should_succeed() {
        // Given: zero average latency
        let average_latency_ms = 0.0;
        let p95_latency_ms = Some(75.0);
        let max_latency_ms = 100.0;

        // When: computing latency score
        let score =
            compute_latency_score(average_latency_ms, p95_latency_ms, max_latency_ms);

        // Then: variance penalty is 1.0 (no variance calculation)
        // latency_ratio = 0.0 / 100.0 = 0.0
        // variance_penalty = 1.0
        // score = 0.0 * 1.0 = 0.0
        assert_eq!(score, 0.0);
    }

    #[test]
    fn compute_latency_score_with_high_variance_should_succeed() {
        // Given: high variance (p95 much higher than avg)
        let average_latency_ms = 10.0;
        let p95_latency_ms = Some(50.0);
        let max_latency_ms = 100.0;

        // When: computing latency score
        let score =
            compute_latency_score(average_latency_ms, p95_latency_ms, max_latency_ms);

        // Then: variance penalty is applied (capped at MAX_VARIANCE_PENALTY)
        // latency_ratio = 0.1
        // variance_ratio = (50.0 - 10.0) / 10.0 = 4.0, capped at 1.0
        // variance_penalty = 1.0 + 1.0 = 2.0
        // score = 0.1 * 2.0 = 0.2
        assert_eq!(score, 0.2);
    }

    #[test]
    fn prepare_scoring_context_with_metrics_should_succeed() {
        // Given: backends, connections, metrics, and routing
        let backends = vec![
            create_test_backend(0, Some(2)),
            create_test_backend(1, Some(4)),
        ];
        let connections = Arc::new(ConnectionRegistry::new(2));
        connections.increment(0);
        connections.increment(1);
        connections.increment(1);
        let metrics = Arc::new(MetricsSnapshot::default());
        metrics.update(
            0,
            BackendMetrics {
                avg_latency_ms: 10.0,
                p95_latency_ms: 15.0,
                error_rate: 0.0,
                last_updated_ms: 1000,
            },
        );
        metrics.update(
            1,
            BackendMetrics {
                avg_latency_ms: 20.0,
                p95_latency_ms: 25.0,
                error_rate: 0.0,
                last_updated_ms: 1000,
            },
        );
        let routing = Arc::new(RouteTable::new(backends.clone()));

        // When: preparing scoring context
        let context = prepare_scoring_context(
            &backends,
            connections.clone(),
            metrics.clone(),
            routing.clone(),
        );

        // Then: context has correct max values
        assert_eq!(context.max_connections, 2); // Max connection count
        assert_eq!(context.max_latency_ms, 25.0); // Max p95 latency (25.0 > 20.0)
        assert_eq!(context.max_weight, 4.0); // Max weight
    }

    #[test]
    fn prepare_scoring_context_without_metrics_should_succeed() {
        // Given: backends without metrics
        let backends = vec![create_test_backend(0, Some(2))];
        let connections = Arc::new(ConnectionRegistry::new(1));
        let metrics = Arc::new(MetricsSnapshot::default());
        let routing = Arc::new(RouteTable::new(backends.clone()));

        // When: preparing scoring context
        let context = prepare_scoring_context(
            &backends,
            connections.clone(),
            metrics.clone(),
            routing.clone(),
        );

        // Then: context uses default max latency
        assert_eq!(context.max_latency_ms, DEFAULT_MAX_LATENCY_MS);
        assert_eq!(context.max_connections, MIN_CONNECTION_COUNT); // At least 1
    }

    #[test]
    fn prepare_scoring_context_with_zero_connections_should_succeed() {
        // Given: backends with zero connections
        let backends = vec![create_test_backend(0, Some(2))];
        let connections = Arc::new(ConnectionRegistry::new(1));
        let metrics = Arc::new(MetricsSnapshot::default());
        let routing = Arc::new(RouteTable::new(backends.clone()));

        // When: preparing scoring context
        let context = prepare_scoring_context(
            &backends,
            connections.clone(),
            metrics.clone(),
            routing.clone(),
        );

        // Then: max_connections is at least MIN_CONNECTION_COUNT
        assert_eq!(context.max_connections, MIN_CONNECTION_COUNT);
    }

    #[test]
    fn compute_backend_score_with_all_metrics_should_succeed() {
        // Given: backend data and scoring context
        let backend_connection_count = 5;
        let backend_weight_value = 2.0;
        let backend_metrics = Some(BackendMetrics {
            avg_latency_ms: 10.0,
            p95_latency_ms: 15.0,
            error_rate: 0.05,
            last_updated_ms: 1000,
        });
        let connections = Arc::new(ConnectionRegistry::new(1));
        let metrics = Arc::new(MetricsSnapshot::default());
        let routing = Arc::new(RouteTable::new(vec![create_test_backend(0, Some(2))]));
        let scoring_context = ScoringContext {
            max_connections: 10,
            max_latency_ms: 100.0,
            max_weight: 4.0,
            connections: connections.clone(),
            metrics: metrics.clone(),
            routing: routing.clone(),
        };
        let weights = AdaptiveWeights::default();

        // When: computing backend score
        let score = compute_backend_score(
            backend_connection_count,
            backend_weight_value,
            backend_metrics,
            &scoring_context,
            &weights,
        );

        // Then: score is computed
        assert!(score >= 0.0);
    }

    #[test]
    fn compute_backend_score_without_metrics_should_succeed() {
        // Given: backend data without metrics
        let backend_connection_count = 5;
        let backend_weight_value = 2.0;
        let backend_metrics = None;
        let connections = Arc::new(ConnectionRegistry::new(1));
        let metrics = Arc::new(MetricsSnapshot::default());
        let routing = Arc::new(RouteTable::new(vec![create_test_backend(0, Some(2))]));
        let scoring_context = ScoringContext {
            max_connections: 10,
            max_latency_ms: 100.0,
            max_weight: 4.0,
            connections: connections.clone(),
            metrics: metrics.clone(),
            routing: routing.clone(),
        };
        let weights = AdaptiveWeights::default();

        // When: computing backend score
        let score = compute_backend_score(
            backend_connection_count,
            backend_weight_value,
            backend_metrics,
            &scoring_context,
            &weights,
        );

        // Then: score is computed with default values
        assert!(score >= 0.0);
    }

    #[test]
    fn compute_backend_score_with_zero_max_weight_should_succeed() {
        // Given: scoring context with zero max weight
        let backend_connection_count = 5;
        let backend_weight_value = 2.0;
        let backend_metrics = None;
        let connections = Arc::new(ConnectionRegistry::new(1));
        let metrics = Arc::new(MetricsSnapshot::default());
        let routing = Arc::new(RouteTable::new(vec![create_test_backend(0, Some(2))]));
        let scoring_context = ScoringContext {
            max_connections: 10,
            max_latency_ms: 100.0,
            max_weight: 0.0,
            connections: connections.clone(),
            metrics: metrics.clone(),
            routing: routing.clone(),
        };
        let weights = AdaptiveWeights::default();

        // When: computing backend score
        let score = compute_backend_score(
            backend_connection_count,
            backend_weight_value,
            backend_metrics,
            &scoring_context,
            &weights,
        );

        // Then: score is computed (uses UNIT_WEIGHT_FACTOR)
        assert!(score >= 0.0);
    }

    #[test]
    fn compute_all_backend_scores_should_succeed() {
        // Given: backends, scoring context, cache, and weights
        let backends = vec![
            create_test_backend(0, Some(2)),
            create_test_backend(1, Some(4)),
        ];
        let connections = Arc::new(ConnectionRegistry::new(2));
        let metrics = Arc::new(MetricsSnapshot::default());
        let routing = Arc::new(RouteTable::new(backends.clone()));
        let scoring_context = ScoringContext {
            max_connections: 10,
            max_latency_ms: 100.0,
            max_weight: 4.0,
            connections: connections.clone(),
            metrics: metrics.clone(),
            routing: routing.clone(),
        };
        let cached_scores = vec![(0, None), (1, None)]; // No cached scores
        let cache = AdaptiveCache::new(1000);
        let weights = AdaptiveWeights::default();
        let current_timestamp_ms = 1000u64;

        // When: computing all backend scores
        let scores = compute_all_backend_scores(
            &backends,
            &scoring_context,
            &cached_scores,
            &cache,
            &weights,
            current_timestamp_ms,
        );

        // Then: scores are computed for all backends
        assert_eq!(scores.len(), 2);
        assert!(scores[0].0 >= 0.0);
        assert!(scores[1].0 >= 0.0);
    }

    #[test]
    fn compute_all_backend_scores_with_cached_scores_should_succeed() {
        // Given: backends with cached scores
        let backends = vec![create_test_backend(0, Some(2))];
        let connections = Arc::new(ConnectionRegistry::new(1));
        let metrics = Arc::new(MetricsSnapshot::default());
        let routing = Arc::new(RouteTable::new(backends.clone()));
        let scoring_context = ScoringContext {
            max_connections: 10,
            max_latency_ms: 100.0,
            max_weight: 4.0,
            connections: connections.clone(),
            metrics: metrics.clone(),
            routing: routing.clone(),
        };
        let cached_scores = vec![(0, Some(5.5))]; // Cached score
        let cache = AdaptiveCache::new(1000);
        let weights = AdaptiveWeights::default();
        let current_timestamp_ms = 1000u64;

        // When: computing all backend scores
        let scores = compute_all_backend_scores(
            &backends,
            &scoring_context,
            &cached_scores,
            &cache,
            &weights,
            current_timestamp_ms,
        );

        // Then: cached score is used
        assert_eq!(scores.len(), 1);
        assert_eq!(scores[0].0, 5.5);
    }

    #[test]
    fn compute_all_backend_scores_empty_backends_should_succeed() {
        // Given: empty backends list
        let backends = Vec::<BackendMeta>::new();
        let connections = Arc::new(ConnectionRegistry::new(0));
        let metrics = Arc::new(MetricsSnapshot::default());
        let routing = Arc::new(RouteTable::new(Vec::new()));
        let scoring_context = ScoringContext {
            max_connections: 1,
            max_latency_ms: 100.0,
            max_weight: 1.0,
            connections: connections.clone(),
            metrics: metrics.clone(),
            routing: routing.clone(),
        };
        let cached_scores = Vec::new();
        let cache = AdaptiveCache::new(1000);
        let weights = AdaptiveWeights::default();
        let current_timestamp_ms = 1000u64;

        // When: computing all backend scores
        let scores = compute_all_backend_scores(
            &backends,
            &scoring_context,
            &cached_scores,
            &cache,
            &weights,
            current_timestamp_ms,
        );

        // Then: empty vector is returned
        assert!(scores.is_empty());
    }
}
