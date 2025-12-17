//! Tests for App module
//!
use lemonade_load_balancer::App;
use lemonade_load_balancer::prelude::*;
use std::sync::Arc;

use crate::common::fixtures::create_test_config_fast;

// Mock service implementations
struct MockConfigService;

#[async_trait]
impl ConfigService for MockConfigService {
    async fn watch_config(&self, ctx: Arc<Context>) {
        // Just wait for shutdown
        let mut shutdown_rx = ctx.channels().shutdown_rx();
        let _ = shutdown_rx.recv().await;
    }
}

struct MockHealthService;

#[async_trait]
impl HealthService for MockHealthService {
    async fn check_health(&self, ctx: Arc<Context>) {
        // Just wait for shutdown
        let mut shutdown_rx = ctx.channels().shutdown_rx();
        let _ = shutdown_rx.recv().await;
    }
}

struct MockMetricsService;

#[async_trait]
impl MetricsService for MockMetricsService {
    async fn collect_metrics(&self, ctx: Arc<Context>) {
        // Just wait for shutdown
        let mut shutdown_rx = ctx.channels().shutdown_rx();
        let _ = shutdown_rx.recv().await;
    }
}

struct MockProxyService;

#[async_trait]
impl ProxyService for MockProxyService {
    async fn accept_connections(&self, ctx: Arc<Context>) -> Result<(), ProxyError> {
        // Wait for shutdown
        let mut shutdown_rx = ctx.channels().shutdown_rx();
        let _ = shutdown_rx.recv().await;
        Ok(())
    }
}

#[tokio::test]
async fn app_new_should_succeed() {
    let config_service: Arc<dyn ConfigService> = Arc::new(MockConfigService);
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
    let ctx = Arc::new(Context::new(config).expect("Failed to create context"));

    let config_service: Arc<dyn ConfigService> = Arc::new(MockConfigService);
    let health_service: Arc<dyn HealthService> = Arc::new(MockHealthService);
    let metrics_service: Arc<dyn MetricsService> = Arc::new(MockMetricsService);
    let proxy_service: Arc<dyn ProxyService> = Arc::new(MockProxyService);

    let app = App::new(
        config_service.clone(),
        health_service.clone(),
        metrics_service.clone(),
        proxy_service.clone(),
    )
    .await;

    // Spawn app.run() in background since it waits for shutdown
    let ctx_clone = ctx.clone();
    let app_handle = tokio::spawn(async move {
        let _ = app.run(ctx_clone).await;
    });

    // Wait a bit for services to start
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Verify app is running (not finished)
    assert!(!app_handle.is_finished());

    // Send shutdown signal
    let _ = ctx.channels().shutdown_tx().send(());

    // Wait a bit for shutdown
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Cleanup
    app_handle.abort();
}

#[tokio::test]
async fn app_new_with_different_services_should_succeed() {
    let config_service1: Arc<dyn ConfigService> = Arc::new(MockConfigService);
    let config_service2: Arc<dyn ConfigService> = Arc::new(MockConfigService);
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
    let ctx_result = Context::new(config);

    assert!(ctx_result.is_ok());
}

#[tokio::test]
async fn app_run_spawns_proxy_service_should_succeed() {
    let config = create_test_config_fast(vec![], Strategy::RoundRobin);
    let ctx = Arc::new(Context::new(config).expect("Failed to create context"));

    let config_service: Arc<dyn ConfigService> = Arc::new(MockConfigService);
    let health_service: Arc<dyn HealthService> = Arc::new(MockHealthService);
    let metrics_service: Arc<dyn MetricsService> = Arc::new(MockMetricsService);
    let proxy_service: Arc<dyn ProxyService> = Arc::new(MockProxyService);

    let app = App::new(
        config_service.clone(),
        health_service,
        metrics_service,
        proxy_service.clone(),
    )
    .await;

    // Spawn app.run() in background since it waits for shutdown
    let ctx_clone = ctx.clone();
    let app_handle = tokio::spawn(async move {
        let _ = app.run(ctx_clone).await;
    });

    // Wait a bit for services to start
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Verify app is running (not finished)
    assert!(!app_handle.is_finished());

    // Send shutdown signal
    let _ = ctx.channels().shutdown_tx().send(());

    // Wait a bit for shutdown
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Cleanup
    app_handle.abort();
}
