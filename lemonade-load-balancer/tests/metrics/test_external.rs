//! Tests for ExternalMetricsService
//!
use lemonade_load_balancer::prelude::*;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn external_metrics_service_new_should_succeed() {
    // Given: a MetricsConfig with external endpoints
    let config = MetricsConfig {
        interval: Duration::from_millis(1),
        timeout: Duration::from_millis(1),
    };

    // When: creating ExternalMetricsService
    let service = ExternalMetricsService::new(Arc::new(ArcSwap::from_pointee(config)));

    // Then: service is created successfully
    assert!(service.is_ok());
}

#[tokio::test]
async fn external_metrics_service_fetch_prometheus_should_succeed() {
    // Given: a service and a mock Prometheus server
    // When: fetching metrics
    // Then: metrics are parsed and returned
    // TODO: Implement when Prometheus support is added
}

#[tokio::test]
async fn external_metrics_service_fetch_otlp_should_succeed() {
    // Given: a service and a mock OTLP server
    // When: fetching metrics
    // Then: metrics are parsed and returned
    // TODO: Implement when OTLP support is added
}
