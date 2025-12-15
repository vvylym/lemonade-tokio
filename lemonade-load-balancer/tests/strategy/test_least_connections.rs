//! Tests for LeastConnections strategy
//!
use lemonade_load_balancer::prelude::*;

use crate::common::fixtures::{create_test_backend, create_test_context};

#[test]
fn least_connections_strategy_strategy_should_succeed() {
    let strategy = LeastConnectionsStrategy::default();
    assert!(matches!(strategy.strategy(), Strategy::LeastConnections));
}

#[tokio::test]
async fn least_connections_strategy_pick_backend_with_least_connections_should_succeed() {
    let strategy = LeastConnectionsStrategy::default();
    let backends = vec![
        create_test_backend(0, None, Some(10u8)),
        create_test_backend(1, None, Some(10u8)),
        create_test_backend(2, None, Some(10u8)),
    ];
    let ctx = create_test_context(backends.clone());
    // Backends start healthy by default

    // Set connection counts: backend 0 has 5, backend 1 has 2, backend 2 has 10
    let routing = ctx.routing_table();
    if let Some(backend0) = routing.get(0) {
        for _ in 0..5 {
            backend0.increment_connection();
        }
    }
    if let Some(backend1) = routing.get(1) {
        for _ in 0..2 {
            backend1.increment_connection();
        }
    }
    if let Some(backend2) = routing.get(2) {
        for _ in 0..10 {
            backend2.increment_connection();
        }
    }

    let backend = strategy
        .pick_backend(ctx)
        .await
        .expect("Failed to pick backend");

    // Backend 1 has least connections (2)
    assert_eq!(backend.id(), &1u8);
}

#[tokio::test]
async fn least_connections_strategy_pick_backend_with_equal_connections_should_succeed() {
    let strategy = LeastConnectionsStrategy::default();
    let backends = vec![
        create_test_backend(0, None, Some(10u8)),
        create_test_backend(1, None, Some(10u8)),
    ];
    let ctx = create_test_context(backends.clone());
    // Backends start healthy by default

    let routing = ctx.routing_table();
    if let Some(backend0) = routing.get(0) {
        backend0.increment_connection();
    }
    if let Some(backend1) = routing.get(1) {
        backend1.increment_connection();
    }

    let backend = strategy
        .pick_backend(ctx)
        .await
        .expect("Failed to pick backend");

    // Either backend can be selected (tie-breaking)
    assert!(*backend.id() == 0 || *backend.id() == 1);
}

#[tokio::test]
async fn least_connections_strategy_pick_backend_with_empty_healthy_should_fail() {
    let strategy = LeastConnectionsStrategy::default();
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let ctx = create_test_context(backends);
    // Mark backend as unhealthy
    let routing = ctx.routing_table();
    if let Some(backend) = routing.get(0) {
        backend.set_health(false, 1000);
    }

    let result = strategy.pick_backend(ctx).await;
    assert!(result.is_err());
    assert!(matches!(
        result.expect_err("Expected error"),
        StrategyError::NoBackendAvailable
    ));
}

#[tokio::test]
async fn least_connections_strategy_pick_backend_with_zero_connections_should_succeed() {
    let strategy = LeastConnectionsStrategy::default();
    let backends = vec![
        create_test_backend(0, None, Some(10u8)),
        create_test_backend(1, None, Some(10u8)),
    ];
    let ctx = create_test_context(backends.clone());
    // Backends start healthy by default, no connections set (all zero)

    let backend = strategy
        .pick_backend(ctx)
        .await
        .expect("Failed to pick backend");

    // Either backend can be selected (both have zero connections)
    assert!(*backend.id() == 0 || *backend.id() == 1);
}

#[test]
fn least_connections_strategy_default_should_succeed() {
    let strategy = LeastConnectionsStrategy::default();
    assert!(matches!(strategy.strategy(), Strategy::LeastConnections));
}
