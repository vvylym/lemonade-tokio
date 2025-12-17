//! Tests for BackendHealthService
//!
use lemonade_load_balancer::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use crate::common::fixtures::create_test_context;

#[tokio::test]
async fn backend_health_service_new_should_succeed() {
    // Given: a HealthConfig
    let config = HealthConfig {
        interval: Duration::from_millis(1),
        timeout: Duration::from_millis(1),
    };

    // When: creating BackendHealthService
    let service = BackendHealthService::new(Arc::new(ArcSwap::from_pointee(config)));

    // Then: service is created successfully
    assert!(service.is_ok());
}

#[tokio::test]
async fn backend_health_service_check_health_performs_health_checks_should_succeed() {
    // Given: a BackendHealthService, context with backends, and a test server
    let config = HealthConfig {
        interval: Duration::from_millis(10),
        timeout: Duration::from_millis(100),
    };
    let service = Arc::new(
        BackendHealthService::new(Arc::new(ArcSwap::from_pointee(config)))
            .expect("Failed to create service"),
    );

    // Create a test server first on a random port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server");
    let server_addr = listener.local_addr().expect("Failed to get server address");

    // Create backend with the server's address
    let backend = BackendMeta::new(0u8, Some("test"), server_addr, Some(10u8));
    let ctx = create_test_context(vec![backend.clone()]);
    let server_handle = tokio::spawn(async move {
        let _ = listener.accept().await;
    });

    // Spawn check_health in background
    let service = Arc::new(service);
    let service_clone = service.clone();
    let ctx_clone = ctx.clone();
    let health_handle = tokio::spawn(async move {
        service_clone.check_health(ctx_clone).await;
    });

    // Wait a bit for health check to run
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Send shutdown signal
    let _ = ctx.channels().shutdown_tx().send(());

    // Wait for service to stop
    let _ = tokio::time::timeout(Duration::from_millis(100), health_handle).await;
    server_handle.abort();
}

#[tokio::test]
async fn backend_health_service_healthy_backend_updates_state_should_succeed() {
    // Given: a running service and a healthy backend
    let config = HealthConfig {
        interval: Duration::from_millis(10),
        timeout: Duration::from_millis(100),
    };
    let service = Arc::new(
        BackendHealthService::new(Arc::new(ArcSwap::from_pointee(config)))
            .expect("Failed to create service"),
    );

    // Create a test server first on a random port that keeps accepting connections
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind server");
    let server_addr = listener.local_addr().expect("Failed to get server address");

    // Create backend with the server's address
    let backend = BackendMeta::new(0u8, Some("test"), server_addr, Some(10u8));
    let ctx = create_test_context(vec![backend.clone()]);

    // Keep server accepting connections in a loop
    let server_handle = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            drop(stream); // Accept and close immediately
        }
    });

    // Spawn check_health in background
    let service = Arc::new(service);
    let service_clone = service.clone();
    let ctx_clone = ctx.clone();
    let health_handle = tokio::spawn(async move {
        service_clone.check_health(ctx_clone).await;
    });

    // Wait for health check to run
    tokio::time::sleep(Duration::from_millis(100)).await;

    // When: health check succeeds
    // Then: backend state is updated with healthy status
    let routing = ctx.routing_table();
    if let Some(backend) = routing.get(0) {
        // Backend should be healthy (starts healthy, and health check should maintain it)
        assert!(backend.is_alive());
    }

    // Send shutdown signal
    let _ = ctx.channels().shutdown_tx().send(());

    // Wait for service to stop
    let _ = tokio::time::timeout(Duration::from_millis(100), health_handle).await;
    server_handle.abort();
}

#[tokio::test]
async fn backend_health_service_respects_backend_load_should_succeed() {
    // Given: a service and a backend with high connection count
    let config = HealthConfig {
        interval: Duration::from_millis(10),
        timeout: Duration::from_millis(100),
    };
    let service = Arc::new(
        BackendHealthService::new(Arc::new(ArcSwap::from_pointee(config)))
            .expect("Failed to create service"),
    );

    let backend = BackendMeta::new(
        0u8,
        Some("test"),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9999),
        Some(10u8),
    );
    let ctx = create_test_context(vec![backend.clone()]);

    // Set high connection count
    let routing = ctx.routing_table();
    if let Some(backend) = routing.get(0) {
        for _ in 0..150 {
            backend.increment_connection();
        }
        // Backend should not have capacity for health check
        assert!(!backend.has_capacity_for_health_check(100));
    }

    // Spawn check_health in background
    let service = Arc::new(service);
    let service_clone = service.clone();
    let ctx_clone = ctx.clone();
    let health_handle = tokio::spawn(async move {
        service_clone.check_health(ctx_clone).await;
    });

    // Wait a bit
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Send shutdown signal
    let _ = ctx.channels().shutdown_tx().send(());

    // Wait for service to stop
    let _ = tokio::time::timeout(Duration::from_millis(100), health_handle).await;
}

#[tokio::test]
async fn backend_health_service_handles_failure_events_should_succeed() {
    // Given: a service and context
    let config = HealthConfig {
        interval: Duration::from_millis(100),
        timeout: Duration::from_millis(100),
    };
    let service = Arc::new(
        BackendHealthService::new(Arc::new(ArcSwap::from_pointee(config)))
            .expect("Failed to create service"),
    );

    let backend = BackendMeta::new(
        0u8,
        Some("test"),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9999),
        Some(10u8),
    );
    let ctx = create_test_context(vec![backend.clone()]);

    // Spawn check_health in background
    let service = Arc::new(service);
    let service_clone = service.clone();
    let ctx_clone = ctx.clone();
    let health_handle = tokio::spawn(async move {
        service_clone.check_health(ctx_clone).await;
    });

    // Wait a bit for service to start
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Send a failure event
    let failure_tx = ctx.channels().backend_failure_tx();
    let _ = failure_tx
        .send(BackendFailureEvent::ConnectionRefused { backend_id: 0 })
        .await;

    // Wait a bit for event processing
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Check that backend was marked unhealthy
    let routing = ctx.routing_table();
    if let Some(backend) = routing.get(0) {
        assert!(!backend.is_alive(), "Backend should be marked unhealthy");
    }

    // Send shutdown signal
    let _ = ctx.channels().shutdown_tx().send(());

    // Wait for service to stop
    let _ = tokio::time::timeout(Duration::from_millis(100), health_handle).await;
}
