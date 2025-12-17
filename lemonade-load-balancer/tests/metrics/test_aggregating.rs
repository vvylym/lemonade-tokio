//! Tests for AggregatingMetricsService
//!
use lemonade_load_balancer::prelude::*;
use std::sync::Arc;
use std::time::Duration;

use crate::common::fixtures::create_test_context;

#[tokio::test]
async fn aggregating_metrics_service_new_should_succeed() {
    // Given: a MetricsConfig
    let config = MetricsConfig {
        interval: Duration::from_millis(1),
        timeout: Duration::from_millis(1),
    };

    // When: creating AggregatingMetricsService
    let service = AggregatingMetricsService::new(Arc::new(ArcSwap::from_pointee(config)));

    // Then: service is created successfully
    assert!(service.is_ok());
}

#[tokio::test]
async fn aggregating_metrics_service_collect_metrics_should_succeed() {
    // Given: a running service and context
    let config = MetricsConfig {
        interval: Duration::from_millis(10),
        timeout: Duration::from_millis(1),
    };
    let service = Arc::new(
        AggregatingMetricsService::new(Arc::new(ArcSwap::from_pointee(config)))
            .expect("Failed to create service"),
    );
    let ctx = create_test_context(vec![]);

    // Spawn collect_metrics in background
    let service = Arc::new(service);
    let service_clone = service.clone();
    let ctx_clone = ctx.clone();
    let metrics_handle = tokio::spawn(async move {
        service_clone.collect_metrics(ctx_clone).await;
    });

    // Wait a bit for service to start
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Send shutdown signal
    let _ = ctx.channels().shutdown_tx().send(());

    // Wait for service to stop
    let _ = tokio::time::timeout(Duration::from_millis(100), metrics_handle).await;
}

#[tokio::test]
async fn aggregating_metrics_service_processes_events_should_succeed() {
    // Given: a service with context and backends
    let config = MetricsConfig {
        interval: Duration::from_millis(10),
        timeout: Duration::from_millis(1),
    };
    let service = Arc::new(
        AggregatingMetricsService::new(Arc::new(ArcSwap::from_pointee(config)))
            .expect("Failed to create service"),
    );

    let backend = BackendMeta::new(
        0u8,
        Some("test"),
        "127.0.0.1:8080".parse::<std::net::SocketAddr>().unwrap(),
        Some(10u8),
    );
    let ctx = create_test_context(vec![backend]);

    // Spawn collect_metrics in background
    let service = Arc::new(service);
    let service_clone = service.clone();
    let ctx_clone = ctx.clone();
    let metrics_handle = tokio::spawn(async move {
        service_clone.collect_metrics(ctx_clone).await;
    });

    // Wait a bit for service to start
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Send metrics events
    let metrics_tx = ctx.channels().metrics_tx();
    let _ = metrics_tx
        .send(MetricsEvent::ConnectionOpened {
            backend_id: 0,
            at_micros: 1000,
        })
        .await;
    let _ = metrics_tx
        .send(MetricsEvent::ConnectionClosed {
            backend_id: 0,
            duration_micros: 5000,
            bytes_in: 100,
            bytes_out: 200,
        })
        .await;

    // Wait a bit for event processing
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Check that backend metrics were updated
    let routing = ctx.routing_table();
    if let Some(backend) = routing.get(0) {
        let metrics = backend.metrics_snapshot();
        // Check that metrics were updated (either latency or error rate)
        assert!(metrics.avg_latency_ms > 0.0 || metrics.error_rate > 0.0);
    }

    // Send shutdown signal
    let _ = ctx.channels().shutdown_tx().send(());

    // Wait for service to stop
    let _ = tokio::time::timeout(Duration::from_millis(100), metrics_handle).await;
}
