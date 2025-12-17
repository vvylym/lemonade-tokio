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
    // Backends start healthy by default

    // Pick backends multiple times to test round robin
    let mut picked_ids = Vec::new();
    for _ in 0..6 {
        let backend = strategy
            .pick_backend(ctx.clone())
            .await
            .expect("Failed to pick backend");
        picked_ids.push(*backend.id());
    }

    // Then: backends are selected in round-robin fashion
    // All backends should be represented
    assert!(picked_ids.contains(&0), "Backend 0 should be selected");
    assert!(picked_ids.contains(&1), "Backend 1 should be selected");
    assert!(picked_ids.contains(&2), "Backend 2 should be selected");

    // The pattern should repeat (after 3 picks, the 4th should match the 1st, etc.)
    assert_eq!(
        picked_ids[0], picked_ids[3],
        "Round robin should repeat after all backends"
    );
    assert_eq!(
        picked_ids[1], picked_ids[4],
        "Round robin should repeat after all backends"
    );
    assert_eq!(
        picked_ids[2], picked_ids[5],
        "Round robin should repeat after all backends"
    );
}

#[tokio::test]
async fn round_robin_strategy_pick_backend_with_empty_healthy_should_fail() {
    let strategy = RoundRobinStrategy::default();
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
async fn round_robin_strategy_pick_backend_single_backend_should_succeed() {
    let strategy = RoundRobinStrategy::default();
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let ctx = create_test_context(backends);
    // Backends start healthy by default

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
