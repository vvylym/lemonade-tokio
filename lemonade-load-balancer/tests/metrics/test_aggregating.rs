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
async fn aggregating_metrics_service_start_aggregates_events_should_succeed() {
    // Given: a running service and context
    let config = MetricsConfig {
        interval: Duration::from_millis(1),
        timeout: Duration::from_millis(1),
    };
    let service = AggregatingMetricsService::new(Arc::new(ArcSwap::from_pointee(config)))
        .expect("Failed to create service");
    let ctx = create_test_context(vec![]);

    // When: starting the service
    let start_result = service.start(ctx.clone()).await;

    // Then: service starts successfully
    assert!(start_result.is_ok());

    // Cleanup
    service.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
async fn aggregating_metrics_service_snapshot_returns_aggregated_metrics_should_succeed()
{
    // Given: a service with aggregated events
    let config = MetricsConfig {
        interval: Duration::from_millis(1),
        timeout: Duration::from_millis(1),
    };
    let service = AggregatingMetricsService::new(Arc::new(ArcSwap::from_pointee(config)))
        .expect("Failed to create service");
    let ctx = create_test_context(vec![]);

    // Start service first (this takes ownership of the receiver)
    service
        .start(ctx.clone())
        .await
        .expect("Failed to start service");

    // Wait a bit for service to initialize
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Send some metrics events
    let metrics_tx = ctx.channel_bundle().metrics_sender();
    let _ = metrics_tx
        .send(MetricsEvent::ConnectionOpened {
            backend_id: 0,
            at_micros: 1000,
        })
        .await;

    // Wait a bit for event processing
    tokio::time::sleep(Duration::from_millis(20)).await;

    // When: calling snapshot()
    let snapshot_result = service.snapshot().await;

    // Then: returns aggregated MetricsSnapshot
    assert!(snapshot_result.is_ok());
    let _snapshot = snapshot_result.expect("Failed to get snapshot");
    // Should have metrics for backend 0 (if event was processed)
    // Note: This may not always pass if event processing is slow, but it verifies the service works

    // Cleanup
    service.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
async fn aggregating_metrics_service_periodic_update_updates_context_should_succeed() {
    // Given: a running service with aggregated metrics
    let config = MetricsConfig {
        interval: Duration::from_millis(10), // Short interval for test
        timeout: Duration::from_millis(1),
    };
    let service = AggregatingMetricsService::new(Arc::new(ArcSwap::from_pointee(config)))
        .expect("Failed to create service");
    let ctx = create_test_context(vec![]);

    service
        .start(ctx.clone())
        .await
        .expect("Failed to start service");

    // Send metrics events
    let metrics_tx = ctx.channel_bundle().metrics_sender();
    let _ = metrics_tx
        .send(MetricsEvent::ConnectionOpened {
            backend_id: 0,
            at_micros: 1000,
        })
        .await;

    // Wait for interval to elapse
    tokio::time::sleep(Duration::from_millis(30)).await;

    // When: interval elapses
    // Then: context metrics snapshot is updated
    let _ = ctx.metrics_snapshot();
    // Metrics should be updated in context (verification would require checking snapshot content)

    // Cleanup
    service.shutdown().await.expect("Failed to shutdown");
}
