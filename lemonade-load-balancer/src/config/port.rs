//! Config Port module
//!
use crate::prelude::*;

/// Config service trait
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ConfigService: Send + Sync + 'static {
    /// Get config snapshot
    fn snapshot(&self) -> Config;

    /// Start the config service
    async fn start(&self, ctx: Arc<Context>) -> Result<(), ConfigError>;

    /// Shutdown the config service
    async fn shutdown(&self) -> Result<(), ConfigError>;
}

#[cfg(test)]
mockall::mock! {
    pub MockConfigServiceSuccess {}

    #[async_trait]
    impl ConfigService for MockConfigServiceSuccess {
        fn snapshot(&self) -> Config {
            use crate::config::models::RuntimeConfig;
            use crate::health::models::HealthConfig;
            use crate::metrics::models::MetricsConfig;
            use crate::proxy::models::ProxyConfig;
            use std::net::{IpAddr, Ipv4Addr, SocketAddr};
            use std::time::Duration;

            Config {
                runtime: RuntimeConfig {
                    metrics_cap: 100,
                    health_cap: 50,
                    drain_timeout_millis: 5000,
                    background_timeout_millis: 1000,
                    accept_timeout_millis: 2000,
                },
                proxy: ProxyConfig {
                    listen_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000),
                    max_connections: Some(1000),
                },
                strategy: Strategy::RoundRobin,
                backends: Vec::new(),
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
        async fn start(&self, _ctx: Arc<Context>) -> Result<(), ConfigError> {
            Ok(())
        }
        async fn shutdown(&self) -> Result<(), ConfigError> {
            Ok(())
        }
    }
}
