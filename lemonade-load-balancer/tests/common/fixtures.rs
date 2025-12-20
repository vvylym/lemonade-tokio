//! Shared test fixtures
//!
//! Provides reusable fixtures for creating test data (BackendMeta, Config, Context)
//! to eliminate code duplication across test files.

use lemonade_load_balancer::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

/// Create a test backend with specified parameters
///
/// Given: backend id, optional name, optional weight
/// When: creating BackendMeta
/// Then: returns BackendMeta with address 127.0.0.1:(8080 + id)
#[rstest::fixture]
pub fn create_test_backend(
    #[default(0)] id: u8,
    #[default(None)] name: Option<String>,
    #[default(Some(10u8))] weight: Option<u8>,
) -> BackendMeta {
    let address =
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080 + id as u16);
    let backend_name = name.unwrap_or_else(|| format!("backend-{}", id));
    BackendMeta::new(
        id,
        Some(backend_name),
        BackendAddress::from(address),
        weight,
    )
}

/// Create a test backend with custom name and port
///
/// Given: backend id, name, port
/// When: creating BackendMeta
/// Then: returns BackendMeta with specified name and port
#[rstest::fixture]
pub fn create_test_backend_with_details(
    #[default(0)] id: u8,
    #[default("backend-0")] name: &str,
    #[default(8080u16)] port: u16,
) -> BackendMeta {
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    BackendMeta::new(
        id,
        Some(name.to_string()),
        BackendAddress::from(address),
        Some(10u8),
    )
}

/// Create a test configuration with specified parameters
///
/// Given: backends, optional strategy, optional runtime config
/// When: creating Config
/// Then: returns Config with sensible defaults
#[rstest::fixture]
pub fn create_test_config(
    #[default(Vec::new())] backends: Vec<BackendMeta>,
    #[default(Strategy::RoundRobin)] strategy: Strategy,
    #[default(RuntimeConfig {
        metrics_cap: 100,
        health_cap: 50,
        drain_timeout_millis: 5000,
        background_timeout_millis: 1000,
        accept_timeout_millis: 2000,
        config_watch_interval_millis: 1000,
    })]
    runtime: RuntimeConfig,
) -> Config {
    // Convert BackendMeta to BackendConfig
    let backend_configs: Vec<BackendConfig> = backends
        .iter()
        .map(|meta| BackendConfig::from(meta.clone()))
        .collect();

    Config {
        source: ConfigSource::Environment,
        runtime: runtime.clone(),
        proxy: ProxyConfig {
            listen_address: SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                3000,
            ),
            max_connections: Some(1000),
        },
        strategy,
        backends: backend_configs,
        health: HealthConfig {
            interval: Duration::from_secs(5),
            timeout: Duration::from_secs(1),
        },
        metrics: MetricsConfig {
            interval: Duration::from_secs(10),
            timeout: Duration::from_secs(2),
        },
        otlp_protocol: None,
        otlp_endpoint: None,
    }
}

/// Create a test configuration with minimal timeouts (for faster tests)
///
/// Given: backends, optional strategy
/// When: creating Config
/// Then: returns Config with short timeouts for test execution
#[rstest::fixture]
pub fn create_test_config_fast(
    #[default(Vec::new())] backends: Vec<BackendMeta>,
    #[default(Strategy::RoundRobin)] strategy: Strategy,
) -> Config {
    create_test_config(
        backends,
        strategy,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 100,
            background_timeout_millis: 50,
            accept_timeout_millis: 50,
            config_watch_interval_millis: 100,
        },
    )
}

/// Create a test context with specified backends
///
/// Given: backends
/// When: creating Context
/// Then: returns Arc<Context> with backends configured
#[rstest::fixture]
pub fn create_test_context(
    #[default(Vec::new())] backend_list: Vec<BackendMeta>,
) -> Arc<Context> {
    let config = create_test_config(
        backend_list.clone(),
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 5000,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 100,
        },
    );
    Arc::new(Context::new(config).expect("Failed to create context"))
}

/// Create a test context with a single backend
///
/// Given: backend id
/// When: creating Context
/// Then: returns Arc<Context> with single backend
#[rstest::fixture]
pub fn create_test_context_single(#[default(0)] id: u8) -> Arc<Context> {
    let backend = create_test_backend(id, None, Some(10u8));
    create_test_context(vec![backend])
}

/// Create multiple test backends
///
/// Given: count of backends
/// When: creating backends
/// Then: returns Vec<BackendMeta> with sequential ids
#[rstest::fixture]
pub fn create_test_backends(#[default(3)] count: usize) -> Vec<BackendMeta> {
    (0..count as u8)
        .map(|id| create_test_backend(id, None, Some(10u8)))
        .collect()
}
