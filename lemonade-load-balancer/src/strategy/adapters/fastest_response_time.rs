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
        let routing = ctx.routing_table();
        let healthy = routing.healthy_backends();

        if healthy.is_empty() {
            return Err(StrategyError::NoBackendAvailable);
        }

        // Find backend with lowest average latency
        // Use first backend as fallback if no metrics available
        let backend = healthy
            .iter()
            .filter_map(|b| {
                let metrics = b.metrics_snapshot();
                if metrics.avg_latency_ms > 0.0 {
                    Some((metrics.avg_latency_ms, b))
                } else {
                    None
                }
            })
            .min_by(|(a, _), (b, _)| {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(_, b)| b)
            .or_else(|| healthy.first())
            .ok_or(StrategyError::NoBackendAvailable)?;

        Ok(BackendMeta::new(
            backend.id(),
            backend.name(),
            backend.address(),
            backend.weight(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::models::{Config, RuntimeConfig};
    use crate::health::models::HealthConfig;
    use crate::metrics::models::MetricsConfig;
    use crate::proxy::models::ProxyConfig;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    fn create_test_backend(id: u8) -> BackendMeta {
        let address =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080 + id as u16);
        BackendMeta::new(id, Some(format!("backend-{}", id)), address, Some(10u8))
    }

    fn create_test_config(backends: Vec<BackendMeta>) -> Config {
        // Convert BackendMeta to BackendConfig
        let backend_configs: Vec<BackendConfig> = backends
            .iter()
            .map(|meta| BackendConfig::from(meta.clone()))
            .collect();
        Config {
            source: ConfigSource::Environment,
            runtime: RuntimeConfig {
                metrics_cap: 100,
                health_cap: 50,
                drain_timeout_millis: 5000,
                background_timeout_millis: 1000,
                accept_timeout_millis: 2000,
                config_watch_interval_millis: 1000,
            },
            proxy: ProxyConfig {
                listen_address: SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                    3000,
                ),
                max_connections: Some(1000),
            },
            strategy: Strategy::FastestResponseTime,
            backends: backend_configs,
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
        let ctx = Arc::new(Context::new(config).expect("Failed to create context"));
        // Backends start healthy by default

        // Set metrics: backend 0 has 10ms, backend 1 has 5ms (lowest), backend 2 has 20ms
        let routing = ctx.routing_table();
        if let Some(backend0) = routing.get(0) {
            backend0.record_request(10, false);
            backend0.record_request(10, false);
        }
        if let Some(backend1) = routing.get(1) {
            backend1.record_request(5, false);
            backend1.record_request(5, false);
        }
        if let Some(backend2) = routing.get(2) {
            backend2.record_request(20, false);
            backend2.record_request(20, false);
        }

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
        let ctx = Arc::new(Context::new(config).expect("Failed to create context"));
        // Backends start healthy by default, no metrics set

        // When: picking backend
        let backend = strategy
            .pick_backend(ctx)
            .await
            .expect("Failed to pick backend");

        // Then: a backend is selected as fallback (could be any backend with no metrics)
        assert!(*backend.id() == 0 || *backend.id() == 1);
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
        let ctx = Arc::new(Context::new(config).expect("Failed to create context"));
        // Backends start healthy by default

        // Set metrics only for backend 0 and 1
        let routing = ctx.routing_table();
        if let Some(backend0) = routing.get(0) {
            backend0.record_request(20, false);
            backend0.record_request(20, false);
        }
        if let Some(backend1) = routing.get(1) {
            backend1.record_request(10, false);
            backend1.record_request(10, false);
        }
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
        let ctx = Arc::new(Context::new(config).expect("Failed to create context"));
        // Mark backend as unhealthy
        let routing = ctx.routing_table();
        if let Some(backend) = routing.get(0) {
            backend.set_health(false, 1000);
        }

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
        let ctx = Arc::new(Context::new(config).expect("Failed to create context"));
        // Backends start healthy by default

        // Set equal latencies
        let routing = ctx.routing_table();
        if let Some(backend0) = routing.get(0) {
            backend0.record_request(10, false);
            backend0.record_request(10, false);
        }
        if let Some(backend1) = routing.get(1) {
            backend1.record_request(10, false);
            backend1.record_request(10, false);
        }

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
