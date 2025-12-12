//! Tests for App module
//!
use lemonade_load_balancer::prelude::*;
use lemonade_load_balancer::{App, spawn_background_handle};
use std::sync::Arc;

use crate::common::fixtures::create_test_config_fast;

// Mock service implementations
struct MockConfigService {
    config: Config,
}

#[async_trait]
impl ConfigService for MockConfigService {
    fn snapshot(&self) -> Config {
        self.config.clone()
    }

    async fn start(&self, _ctx: Arc<Context>) -> Result<(), ConfigError> {
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), ConfigError> {
        Ok(())
    }
}

struct MockHealthService;

#[async_trait]
impl HealthService for MockHealthService {
    async fn start(&self, _ctx: Arc<Context>) -> Result<(), HealthError> {
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), HealthError> {
        Ok(())
    }
}

struct MockMetricsService;

#[async_trait]
impl MetricsService for MockMetricsService {
    async fn snapshot(&self) -> Result<MetricsSnapshot, MetricsError> {
        Ok(MetricsSnapshot::default())
    }

    async fn start(&self, _ctx: Arc<Context>) -> Result<(), MetricsError> {
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), MetricsError> {
        Ok(())
    }
}

struct MockProxyService;

#[async_trait]
impl ProxyService for MockProxyService {
    async fn accept_connections(&self, _ctx: &Arc<Context>) -> Result<(), ProxyError> {
        Ok(())
    }
}

#[tokio::test]
async fn app_new_should_succeed() {
    let config = create_test_config_fast(vec![], Strategy::RoundRobin);
    let config_service: Arc<dyn ConfigService> = Arc::new(MockConfigService {
        config: config.clone(),
    });
    let health_service: Arc<dyn HealthService> = Arc::new(MockHealthService);
    let metrics_service: Arc<dyn MetricsService> = Arc::new(MockMetricsService);
    let proxy_service: Arc<dyn ProxyService> = Arc::new(MockProxyService);

    let _app = App::new(
        config_service,
        health_service,
        metrics_service,
        proxy_service,
    )
    .await;
}

#[tokio::test]
async fn app_run_creates_context_and_spawns_handles_should_succeed() {
    let config = create_test_config_fast(vec![], Strategy::RoundRobin);
    let config_service: Arc<dyn ConfigService> = Arc::new(MockConfigService {
        config: config.clone(),
    });
    let health_service: Arc<dyn HealthService> = Arc::new(MockHealthService);
    let metrics_service: Arc<dyn MetricsService> = Arc::new(MockMetricsService);
    let proxy_service: Arc<dyn ProxyService> = Arc::new(MockProxyService);

    let _app = App::new(
        config_service.clone(),
        health_service.clone(),
        metrics_service.clone(),
        proxy_service.clone(),
    )
    .await;

    let config_snapshot = config_service.snapshot();
    let ctx = Arc::new(Context::new(&config_snapshot).expect("Failed to create context"));

    let config_handle = spawn_background_handle!(config_service, &ctx);
    let health_handle = spawn_background_handle!(health_service, &ctx);
    let metrics_handle = spawn_background_handle!(metrics_service, &ctx);
    let _accept_handle = proxy_service.accept_connections(&ctx);

    assert!(!config_handle.is_finished());
    assert!(!health_handle.is_finished());
    assert!(!metrics_handle.is_finished());

    config_handle.abort();
    health_handle.abort();
    metrics_handle.abort();
}

#[tokio::test]
async fn app_new_with_different_services_should_succeed() {
    let config1 = create_test_config_fast(vec![], Strategy::RoundRobin);
    let config2 = create_test_config_fast(vec![], Strategy::RoundRobin);
    let config_service1: Arc<dyn ConfigService> =
        Arc::new(MockConfigService { config: config1 });
    let config_service2: Arc<dyn ConfigService> =
        Arc::new(MockConfigService { config: config2 });
    let health_service: Arc<dyn HealthService> = Arc::new(MockHealthService);
    let metrics_service: Arc<dyn MetricsService> = Arc::new(MockMetricsService);
    let proxy_service: Arc<dyn ProxyService> = Arc::new(MockProxyService);

    let _app1 = App::new(
        config_service1,
        health_service.clone(),
        metrics_service.clone(),
        proxy_service.clone(),
    )
    .await;
    let _app2 = App::new(
        config_service2,
        health_service,
        metrics_service,
        proxy_service,
    )
    .await;
}

#[tokio::test]
async fn app_run_creates_context_should_succeed() {
    let config = create_test_config_fast(vec![], Strategy::RoundRobin);
    let config_service: Arc<dyn ConfigService> = Arc::new(MockConfigService {
        config: config.clone(),
    });
    let health_service: Arc<dyn HealthService> = Arc::new(MockHealthService);
    let metrics_service: Arc<dyn MetricsService> = Arc::new(MockMetricsService);
    let proxy_service: Arc<dyn ProxyService> = Arc::new(MockProxyService);

    let _app = App::new(
        config_service.clone(),
        health_service,
        metrics_service,
        proxy_service,
    )
    .await;

    let config_snapshot = config_service.snapshot();
    let ctx_result = Context::new(&config_snapshot);

    assert!(ctx_result.is_ok());
}
