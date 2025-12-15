//! Tests for WeightedRoundRobin strategy
//!
use lemonade_load_balancer::prelude::*;

use crate::common::fixtures::{create_test_backend, create_test_context};

#[test]
fn weighted_round_robin_strategy_strategy_should_succeed() {
    let strategy = WeightedRoundRobinStrategy::default();
    assert!(matches!(strategy.strategy(), Strategy::WeightedRoundRobin));
}

#[tokio::test]
async fn weighted_round_robin_strategy_pick_backend_with_weights_should_succeed() {
    let strategy = WeightedRoundRobinStrategy::default();
    let backends = vec![
        create_test_backend(0, None, Some(3)), // weight 3
        create_test_backend(1, None, Some(1)), // weight 1
        create_test_backend(2, None, Some(2)), // weight 2
    ];
    let ctx = create_test_context(backends.clone());
    // Backends start healthy by default

    // Pick backends multiple times
    let mut selected = Vec::new();
    for _ in 0..6 {
        let backend = strategy
            .pick_backend(ctx.clone())
            .await
            .expect("Failed to pick backend");
        selected.push(*backend.id());
    }

    // With weights 3:1:2, backend 0 should be selected more
    let count_0 = selected.iter().filter(|&&id| id == 0).count();
    let count_1 = selected.iter().filter(|&&id| id == 1).count();
    let count_2 = selected.iter().filter(|&&id| id == 2).count();
    assert!(count_0 >= count_1);
    assert!(count_0 >= count_2);
}

#[tokio::test]
async fn weighted_round_robin_strategy_pick_backend_with_equal_weights_should_succeed() {
    let strategy = WeightedRoundRobinStrategy::default();
    let backends = vec![
        create_test_backend(0, None, Some(1)),
        create_test_backend(1, None, Some(1)),
    ];
    let ctx = create_test_context(backends.clone());
    // Backends start healthy by default

    let mut selected = Vec::new();
    for _ in 0..10 {
        let backend = strategy
            .pick_backend(ctx.clone())
            .await
            .expect("Failed to pick backend");
        selected.push(*backend.id());
    }

    // Should be approximately equal
    let count_0 = selected.iter().filter(|&&id| id == 0).count();
    let count_1 = selected.iter().filter(|&&id| id == 1).count();
    assert!((count_0 as i32 - count_1 as i32).abs() <= 2);
}

#[tokio::test]
async fn weighted_round_robin_strategy_pick_backend_with_zero_weights_should_fail() {
    let strategy = WeightedRoundRobinStrategy::default();
    let backends = vec![
        create_test_backend(0, None, Some(0)),
        create_test_backend(1, None, Some(0)),
    ];
    let ctx = create_test_context(backends.clone());
    // Backends start healthy by default

    let result = strategy.pick_backend(ctx).await;
    assert!(result.is_err());
    assert!(matches!(
        result.expect_err("Expected error"),
        StrategyError::NoBackendAvailable
    ));
}

#[tokio::test]
async fn weighted_round_robin_strategy_pick_backend_with_empty_healthy_should_fail() {
    let strategy = WeightedRoundRobinStrategy::default();
    let backends = vec![create_test_backend(0, None, Some(1))];
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
async fn weighted_round_robin_strategy_pick_backend_with_none_weights_should_succeed() {
    let strategy = WeightedRoundRobinStrategy::default();
    let backends = vec![
        create_test_backend(0, None, None),
        create_test_backend(1, None, None),
    ];
    let ctx = create_test_context(backends.clone());
    // Backends start healthy by default

    let backend1 = strategy
        .pick_backend(ctx.clone())
        .await
        .expect("Failed to pick backend");
    let backend2 = strategy
        .pick_backend(ctx.clone())
        .await
        .expect("Failed to pick backend");

    // None weight defaults to 1
    assert!(*backend1.id() == 0 || *backend1.id() == 1);
    assert!(*backend2.id() == 0 || *backend2.id() == 1);
}

#[test]
fn weighted_round_robin_strategy_default_should_succeed() {
    let strategy = WeightedRoundRobinStrategy::default();
    assert!(matches!(strategy.strategy(), Strategy::WeightedRoundRobin));
}
