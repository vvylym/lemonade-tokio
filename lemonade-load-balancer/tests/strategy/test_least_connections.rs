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

    let health = ctx.health_registry();
    let routing = ctx.routing_table();
    for i in 0..routing.len() {
        health.set_alive(i, true, 1000);
    }

    // Set connection counts: backend 0 has 5, backend 1 has 2, backend 2 has 10
    let connections = ctx.connection_registry();
    for _ in 0..5 {
        connections.increment(0);
    }
    for _ in 0..2 {
        connections.increment(1);
    }
    for _ in 0..10 {
        connections.increment(2);
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

    let health = ctx.health_registry();
    health.set_alive(0, true, 1000);
    health.set_alive(1, true, 1000);

    let connections = ctx.connection_registry();
    connections.increment(0);
    connections.increment(1);

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
    // All backends are unhealthy by default

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

    let health = ctx.health_registry();
    health.set_alive(0, true, 1000);
    health.set_alive(1, true, 1000);
    // No connections set (all zero)

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
