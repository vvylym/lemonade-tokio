//! Tests for RoundRobin strategy
//!
use lemonade_load_balancer::prelude::*;

use crate::common::fixtures::{create_test_backend, create_test_context};

#[test]
fn round_robin_strategy_strategy_should_succeed() {
    let strategy = RoundRobinStrategy::default();
    assert!(matches!(strategy.strategy(), Strategy::RoundRobin));
}

#[tokio::test]
async fn round_robin_strategy_pick_backend_with_multiple_backends_should_succeed() {
    let strategy = RoundRobinStrategy::default();
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

    let backend1 = strategy
        .pick_backend(ctx.clone())
        .await
        .expect("Failed to pick backend");
    let backend2 = strategy
        .pick_backend(ctx.clone())
        .await
        .expect("Failed to pick backend");
    let backend3 = strategy
        .pick_backend(ctx.clone())
        .await
        .expect("Failed to pick backend");
    let backend4 = strategy
        .pick_backend(ctx.clone())
        .await
        .expect("Failed to pick backend");

    assert_eq!(backend1.id(), &0u8);
    assert_eq!(backend2.id(), &1u8);
    assert_eq!(backend3.id(), &2u8);
    assert_eq!(backend4.id(), &0u8); // Wraps around
}

#[tokio::test]
async fn round_robin_strategy_pick_backend_with_empty_healthy_should_fail() {
    let strategy = RoundRobinStrategy::default();
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let ctx = create_test_context(backends);

    let result = strategy.pick_backend(ctx).await;
    assert!(result.is_err());
    assert!(matches!(
        result.expect_err("Expected error"),
        StrategyError::NoBackendAvailable
    ));
}

#[tokio::test]
async fn round_robin_strategy_pick_backend_single_backend_should_succeed() {
    let strategy = RoundRobinStrategy::default();
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let ctx = create_test_context(backends);

    let health = ctx.health_registry();
    health.set_alive(0, true, 1000);

    let backend1 = strategy
        .pick_backend(ctx.clone())
        .await
        .expect("Failed to pick backend");
    let backend2 = strategy
        .pick_backend(ctx.clone())
        .await
        .expect("Failed to pick backend");

    assert_eq!(backend1.id(), &0u8);
    assert_eq!(backend2.id(), &0u8);
}

#[test]
fn round_robin_strategy_default_should_succeed() {
    let strategy = RoundRobinStrategy::default();
    assert!(matches!(strategy.strategy(), Strategy::RoundRobin));
}
