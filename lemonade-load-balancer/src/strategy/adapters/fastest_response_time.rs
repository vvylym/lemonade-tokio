use crate::prelude::*;

/// Fastest response time strategy
#[derive(Default)]
pub struct FastestResponseTimeStrategy {}

#[async_trait]
impl StrategyService for FastestResponseTimeStrategy {
    fn strategy(&self) -> Strategy {
        Strategy::FastestResponseTime
    }

    async fn pick_backend(
        &self,
        ctx: Arc<Context>,
    ) -> Result<BackendMeta, StrategyError> {
        let healthy = ctx.healthy_backends();

        if healthy.is_empty() {
            return Err(StrategyError::NoBackendAvailable);
        }

        // Load metrics once
        let metrics = ctx.metrics_snapshot();

        // Find backend with lowest average latency
        // Use first backend as fallback if no metrics available
        let backend = healthy
            .iter()
            .filter_map(|b| metrics.avg_latency(*b.id()).map(|lat| (lat, b)))
            .min_by(|(a, _), (b, _)| {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(_, b)| b)
            .or_else(|| healthy.first())
            .ok_or(StrategyError::NoBackendAvailable)?;

        Ok(backend.clone())
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

    fn create_test_backend(id: u8) -> BackendMeta {
        let address =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080 + id as u16);
        BackendMeta::new(id, Some(format!("backend-{}", id)), address, Some(10u8))
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
            strategy: Strategy::FastestResponseTime,
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
    fn fastest_response_time_strategy_strategy_should_succeed() {
        // Given: a FastestResponseTimeStrategy
        let strategy = FastestResponseTimeStrategy::default();

        // When: getting strategy type
        let strategy_type = strategy.strategy();

        // Then: returns FastestResponseTime
        assert!(matches!(strategy_type, Strategy::FastestResponseTime));
    }

    #[tokio::test]
    async fn fastest_response_time_strategy_pick_backend_with_lowest_latency_should_succeed()
     {
        // Given: a FastestResponseTimeStrategy and Context with backends having different latencies
        let strategy = FastestResponseTimeStrategy::default();
        let backends = vec![
            create_test_backend(0),
            create_test_backend(1),
            create_test_backend(2),
        ];
        let config = create_test_config(backends.clone());
        let ctx = Arc::new(Context::new(&config).expect("Failed to create context"));
        ctx.set_backends(backends).expect("Failed to set backends");

        let health = ctx.health_registry();
        let routing = ctx.routing_table();
        for i in 0..routing.len() {
            health.set_alive(i, true, 1000);
        }

        // Set metrics: backend 0 has 10ms, backend 1 has 5ms (lowest), backend 2 has 20ms
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
                avg_latency_ms: 5.0,
                p95_latency_ms: 8.0,
                error_rate: 0.0,
                last_updated_ms: 1000,
            },
        );
        metrics.update(
            2,
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

        // Then: backend with lowest latency (backend 1) is selected
        assert_eq!(backend.id(), &1u8);
    }

    #[tokio::test]
    async fn fastest_response_time_strategy_pick_backend_with_no_metrics_should_succeed()
    {
        // Given: a FastestResponseTimeStrategy and Context with no metrics
        let strategy = FastestResponseTimeStrategy::default();
        let backends = vec![create_test_backend(0), create_test_backend(1)];
        let config = create_test_config(backends.clone());
        let ctx = Arc::new(Context::new(&config).expect("Failed to create context"));
        ctx.set_backends(backends).expect("Failed to set backends");

        let health = ctx.health_registry();
        health.set_alive(0, true, 1000);
        health.set_alive(1, true, 1000);
        // No metrics set

        // When: picking backend
        let backend = strategy
            .pick_backend(ctx)
            .await
            .expect("Failed to pick backend");

        // Then: first backend is selected as fallback
        assert_eq!(backend.id(), &0u8);
    }

    #[tokio::test]
    async fn fastest_response_time_strategy_pick_backend_with_partial_metrics_should_succeed()
     {
        // Given: a FastestResponseTimeStrategy and Context with partial metrics
        let strategy = FastestResponseTimeStrategy::default();
        let backends = vec![
            create_test_backend(0),
            create_test_backend(1),
            create_test_backend(2),
        ];
        let config = create_test_config(backends.clone());
        let ctx = Arc::new(Context::new(&config).expect("Failed to create context"));
        ctx.set_backends(backends).expect("Failed to set backends");

        let health = ctx.health_registry();
        let routing = ctx.routing_table();
        for i in 0..routing.len() {
            health.set_alive(i, true, 1000);
        }

        // Set metrics only for backend 0 and 1
        let metrics = ctx.metrics_snapshot();
        metrics.update(
            0,
            BackendMetrics {
                avg_latency_ms: 20.0,
                p95_latency_ms: 25.0,
                error_rate: 0.0,
                last_updated_ms: 1000,
            },
        );
        metrics.update(
            1,
            BackendMetrics {
                avg_latency_ms: 10.0,
                p95_latency_ms: 15.0,
                error_rate: 0.0,
                last_updated_ms: 1000,
            },
        );
        // Backend 2 has no metrics

        // When: picking backend
        let backend = strategy
            .pick_backend(ctx)
            .await
            .expect("Failed to pick backend");

        // Then: backend with lowest latency among those with metrics (backend 1) is selected
        assert_eq!(backend.id(), &1u8);
    }

    #[tokio::test]
    async fn fastest_response_time_strategy_pick_backend_with_empty_healthy_should_fail()
    {
        // Given: a FastestResponseTimeStrategy and Context with no healthy backends
        let strategy = FastestResponseTimeStrategy::default();
        let backends = vec![create_test_backend(0)];
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
    async fn fastest_response_time_strategy_pick_backend_with_equal_latencies_should_succeed()
     {
        // Given: a FastestResponseTimeStrategy and Context with backends having equal latencies
        let strategy = FastestResponseTimeStrategy::default();
        let backends = vec![create_test_backend(0), create_test_backend(1)];
        let config = create_test_config(backends.clone());
        let ctx = Arc::new(Context::new(&config).expect("Failed to create context"));
        ctx.set_backends(backends).expect("Failed to set backends");

        let health = ctx.health_registry();
        health.set_alive(0, true, 1000);
        health.set_alive(1, true, 1000);

        // Set equal latencies
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
                avg_latency_ms: 10.0,
                p95_latency_ms: 15.0,
                error_rate: 0.0,
                last_updated_ms: 1000,
            },
        );

        // When: picking backend
        let backend = strategy
            .pick_backend(ctx)
            .await
            .expect("Failed to pick backend");

        // Then: a backend is selected (tie-breaking behavior - first in iteration)
        assert!(*backend.id() == 0 || *backend.id() == 1);
    }

    #[test]
    fn fastest_response_time_strategy_default_should_succeed() {
        // Given: default FastestResponseTimeStrategy
        // When: creating with default
        let strategy = FastestResponseTimeStrategy::default();

        // Then: strategy is created
        assert!(matches!(strategy.strategy(), Strategy::FastestResponseTime));
    }
}
