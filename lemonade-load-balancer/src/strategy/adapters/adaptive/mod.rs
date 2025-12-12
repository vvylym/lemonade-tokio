//! Adaptive strategy implementation
//!
//! The adaptive strategy uses a multi-factor scoring algorithm that considers:
//! - Connection load (normalized by weight)
//! - Response latency (with variance penalty)
//! - Error rates (with additional penalty for high error rates)
//!
//! All factors are normalized and combined using configurable weights.
//! The strategy includes caching for performance optimization.

use crate::prelude::*;

mod cache;
mod constants;
mod models;
mod utils;

use cache::AdaptiveCache;
use models::AdaptiveWeights;
use utils::*;

/// Adaptive strategy implementation with multi-factor scoring
///
/// This strategy provides superior performance by considering multiple
/// factors simultaneously and adapting to changing conditions. It uses
/// caching to minimize computation overhead.
pub struct AdaptiveStrategy {
    /// Cache for computed scores to avoid recalculation
    cache: AdaptiveCache,
    /// Weights for scoring factors (connection, latency, error rate)
    weights: AdaptiveWeights,
}

impl AdaptiveStrategy {
    /// Create a new adaptive strategy with default weights from constants
    ///
    /// Uses default weights defined in the constants module:
    /// - Connection weight: 0.4
    /// - Latency weight: 0.4
    /// - Error rate weight: 0.2
    pub fn new() -> Self {
        Self {
            cache: AdaptiveCache::default(),
            weights: AdaptiveWeights::default(),
        }
    }

    /// Create a new adaptive strategy with optional custom weights
    ///
    /// # Arguments
    /// * `custom_weights` - Optional custom weight configuration.
    ///   If None, uses default weights from constants.
    pub fn with_weights(custom_weights: Option<AdaptiveWeights>) -> Self {
        Self {
            cache: AdaptiveCache::default(),
            weights: custom_weights.unwrap_or_default(),
        }
    }
}

impl Default for AdaptiveStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StrategyService for AdaptiveStrategy {
    fn strategy(&self) -> Strategy {
        Strategy::Adaptive
    }

    /// Pick the best backend using adaptive multi-factor scoring
    ///
    /// This method:
    /// 1. Gets healthy backends from context
    /// 2. Prepares scoring context with normalized values
    /// 3. Checks cache for existing scores
    /// 4. Computes scores for backends (using cache when available)
    /// 5. Returns backend with lowest score (best performance)
    ///
    /// # Arguments
    /// * `ctx` - Runtime context containing registries and state
    ///
    /// # Returns
    /// Selected backend metadata, or error if no backends available
    async fn pick_backend(
        &self,
        ctx: Arc<Context>,
    ) -> Result<BackendMeta, StrategyError> {
        let healthy_backends = ctx.healthy_backends();

        // Early exit optimization: single backend
        if healthy_backends.len() == 1 {
            return Ok(healthy_backends[0].clone());
        }

        if healthy_backends.is_empty() {
            return Err(StrategyError::NoBackendAvailable);
        }

        // Load all registries once (single read optimization)
        let connection_registry = ctx.connection_registry();
        let metrics_snapshot = ctx.metrics_snapshot();
        let routing_table = ctx.routing_table();

        // Get current timestamp for cache TTL validation
        let current_timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Prepare scoring context with normalized maximum values
        let scoring_context = prepare_scoring_context(
            &healthy_backends,
            connection_registry,
            metrics_snapshot,
            routing_table,
        );

        // Check cache for all backends in parallel
        let cached_scores: Vec<(BackendId, Option<f64>)> = healthy_backends
            .iter()
            .map(|backend| {
                (
                    *backend.id(),
                    self.cache.get(*backend.id(), current_timestamp_ms),
                )
            })
            .collect();

        // Compute scores for all backends (uses cache when available)
        let scored_backends = compute_all_backend_scores(
            &healthy_backends,
            &scoring_context,
            &cached_scores,
            &self.cache,
            &self.weights,
            current_timestamp_ms,
        );

        // Find backend with lowest score (best performance)
        let (_best_score, best_backend) = scored_backends
            .iter()
            .min_by(|(first_score, _), (second_score, _)| {
                first_score
                    .partial_cmp(second_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or(StrategyError::NoBackendAvailable)?;

        Ok(best_backend.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::models::{Config, RuntimeConfig};
    use crate::health::models::HealthConfig;
    use crate::metrics::models::MetricsConfig;
    use crate::proxy::models::ProxyConfig;
    use crate::types::BackendMetrics;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    fn create_test_backend(id: u8, weight: Option<u8>) -> BackendMeta {
        let address =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080 + id as u16);
        BackendMeta::new(id, Some(format!("backend-{}", id)), address, weight)
    }

    fn create_test_config(backends: Vec<BackendMeta>) -> Config {
        Config {
            runtime: RuntimeConfig {
                metrics_cap: 100,
                health_cap: 50,
                drain_timeout_millis: 5000,
                background_timeout_millis: 1000,
                accept_timeout_millis: 2000,
            },
            proxy: ProxyConfig {
                listen_address: SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                    3000,
                ),
                max_connections: Some(1000),
            },
            strategy: Strategy::Adaptive,
            backends: backends.clone(),
            health: HealthConfig {
                interval: Duration::from_secs(5),
                timeout: Duration::from_secs(1),
            },
            metrics: MetricsConfig {
                interval: Duration::from_secs(10),
                timeout: Duration::from_secs(2),
            },
        }
    }

    #[test]
    fn adaptive_strategy_new_should_succeed() {
        // Given: a new AdaptiveStrategy
        // When: creating a new strategy
        let strategy = AdaptiveStrategy::new();

        // Then: strategy is created successfully
        assert!(matches!(strategy.strategy(), Strategy::Adaptive));
    }

    #[test]
    fn adaptive_strategy_default_should_succeed() {
        // Given: default AdaptiveStrategy
        // When: creating with default
        let strategy = AdaptiveStrategy::default();

        // Then: strategy is created
        assert!(matches!(strategy.strategy(), Strategy::Adaptive));
    }

    #[test]
    fn adaptive_strategy_with_weights_none_should_succeed() {
        // Given: AdaptiveStrategy with None weights
        // When: creating with None weights
        let strategy = AdaptiveStrategy::with_weights(None);

        // Then: strategy is created with default weights
        assert!(matches!(strategy.strategy(), Strategy::Adaptive));
    }

    #[test]
    fn adaptive_strategy_with_weights_some_should_succeed() {
        // Given: custom weights
        let custom_weights = AdaptiveWeights {
            conn_weight: 0.5,
            latency_weight: 0.3,
            error_weight: 0.2,
        };

        // When: creating with custom weights
        let strategy = AdaptiveStrategy::with_weights(Some(custom_weights));

        // Then: strategy is created
        assert!(matches!(strategy.strategy(), Strategy::Adaptive));
    }

    #[test]
    fn adaptive_strategy_strategy_should_succeed() {
        // Given: an AdaptiveStrategy
        let strategy = AdaptiveStrategy::default();

        // When: getting strategy type
        let strategy_type = strategy.strategy();

        // Then: returns Adaptive
        assert!(matches!(strategy_type, Strategy::Adaptive));
    }

    #[tokio::test]
    async fn adaptive_strategy_pick_backend_with_single_backend_should_succeed() {
        // Given: an AdaptiveStrategy and Context with single healthy backend
        let strategy = AdaptiveStrategy::default();
        let backends = vec![create_test_backend(0, Some(1))];
        let config = create_test_config(backends.clone());
        let ctx = Arc::new(Context::new(&config).expect("Failed to create context"));
        ctx.set_backends(backends).expect("Failed to set backends");

        let health = ctx.health_registry();
        health.set_alive(0, true, 1000);

        // When: picking backend
        let backend = strategy
            .pick_backend(ctx)
            .await
            .expect("Failed to pick backend");

        // Then: single backend is returned (early exit optimization)
        assert_eq!(backend.id(), &0u8);
    }

    #[tokio::test]
    async fn adaptive_strategy_pick_backend_with_empty_healthy_should_fail() {
        // Given: an AdaptiveStrategy and Context with no healthy backends
        let strategy = AdaptiveStrategy::default();
        let backends = vec![create_test_backend(0, Some(1))];
        let config = create_test_config(backends);
        let ctx = Arc::new(Context::new(&config).expect("Failed to create context"));
        // All backends are unhealthy by default

        // When: picking backend
        let result = strategy.pick_backend(ctx).await;

        // Then: returns NoBackendAvailable error
        assert!(result.is_err());
        assert!(matches!(
            result.expect_err("Failed to pick backend"),
            StrategyError::NoBackendAvailable
        ));
    }

    #[tokio::test]
    async fn adaptive_strategy_pick_backend_with_multiple_backends_should_succeed() {
        // Given: an AdaptiveStrategy and Context with multiple healthy backends
        let strategy = AdaptiveStrategy::default();
        let backends = vec![
            create_test_backend(0, Some(1)),
            create_test_backend(1, Some(2)),
        ];
        let config = create_test_config(backends.clone());
        let ctx = Arc::new(Context::new(&config).expect("Failed to create context"));
        ctx.set_backends(backends).expect("Failed to set backends");

        let health = ctx.health_registry();
        let routing = ctx.routing_table();
        for i in 0..routing.len() {
            health.set_alive(i, true, 1000);
        }

        // When: picking backend
        let backend = strategy
            .pick_backend(ctx)
            .await
            .expect("Failed to pick backend");

        // Then: a backend is selected
        assert!(*backend.id() == 0 || *backend.id() == 1);
    }

    #[tokio::test]
    async fn adaptive_strategy_pick_backend_with_metrics_should_succeed() {
        // Given: an AdaptiveStrategy and Context with backends having metrics
        let strategy = AdaptiveStrategy::default();
        let backends = vec![
            create_test_backend(0, Some(1)),
            create_test_backend(1, Some(1)),
        ];
        let config = create_test_config(backends.clone());
        let ctx = Arc::new(Context::new(&config).expect("Failed to create context"));
        ctx.set_backends(backends).expect("Failed to set backends");

        let health = ctx.health_registry();
        health.set_alive(0, true, 1000);
        health.set_alive(1, true, 1000);

        // Set metrics: backend 0 has lower latency
        let metrics = ctx.metrics_snapshot();
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

        // When: picking backend
        let backend = strategy
            .pick_backend(ctx)
            .await
            .expect("Failed to pick backend");

        // Then: backend with better metrics (lower latency) is selected
        // Backend 0 should be selected due to lower latency
        assert_eq!(backend.id(), &0u8);
    }

    #[tokio::test]
    async fn adaptive_strategy_pick_backend_with_connections_should_succeed() {
        // Given: an AdaptiveStrategy and Context with backends having different connection counts
        let strategy = AdaptiveStrategy::default();
        let backends = vec![
            create_test_backend(0, Some(1)),
            create_test_backend(1, Some(1)),
        ];
        let config = create_test_config(backends.clone());
        let ctx = Arc::new(Context::new(&config).expect("Failed to create context"));
        ctx.set_backends(backends).expect("Failed to set backends");

        let health = ctx.health_registry();
        health.set_alive(0, true, 1000);
        health.set_alive(1, true, 1000);

        // Set connection counts: backend 0 has fewer connections
        let connections = ctx.connection_registry();
        connections.increment(0);
        connections.increment(1);
        connections.increment(1);

        // When: picking backend
        let backend = strategy
            .pick_backend(ctx)
            .await
            .expect("Failed to pick backend");

        // Then: backend with fewer connections is selected
        assert_eq!(backend.id(), &0u8);
    }
}
