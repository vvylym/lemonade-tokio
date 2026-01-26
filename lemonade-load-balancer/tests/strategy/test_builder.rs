//! Strategy builder tests
//!
//! Tests for the StrategyBuilder covering:
//! - Construction (new, default)
//! - Builder methods (with_strategy)
//! - Building strategies (all strategy types)
//! - Error handling (building without strategy)
//! - Method chaining
//! - Trait implementations (Debug)

use lemonade_load_balancer::prelude::*;

#[test]
fn strategy_builder_with_strategy_should_succeed() {
    // Given: a StrategyBuilder
    let builder = StrategyBuilder::new();

    // When: setting strategy
    let builder = builder.with_strategy(Strategy::RoundRobin);

    // Then: builder can be used (verified by build)
    let result = builder.build();
    assert!(result.is_ok());
}

#[test]
fn strategy_builder_build_adaptive_should_succeed() {
    // Given: a StrategyBuilder with Adaptive strategy
    let builder = StrategyBuilder::new().with_strategy(Strategy::Adaptive);

    // When: building the strategy
    let result = builder.build();

    // Then: build succeeds with Adaptive strategy
    assert!(result.is_ok());
    let strategy_service = result.expect("Failed to build strategy");
    assert!(matches!(strategy_service.strategy(), Strategy::Adaptive));
}

#[test]
fn strategy_builder_build_fastest_response_time_should_succeed() {
    // Given: a StrategyBuilder with FastestResponseTime strategy
    let builder = StrategyBuilder::new().with_strategy(Strategy::FastestResponseTime);

    // When: building the strategy
    let result = builder.build();

    // Then: build succeeds with FastestResponseTime strategy
    assert!(result.is_ok());
    let strategy_service = result.expect("Failed to build strategy");
    assert!(matches!(
        strategy_service.strategy(),
        Strategy::FastestResponseTime
    ));
}

#[test]
fn strategy_builder_build_least_connections_should_succeed() {
    // Given: a StrategyBuilder with LeastConnections strategy
    let builder = StrategyBuilder::new().with_strategy(Strategy::LeastConnections);

    // When: building the strategy
    let result = builder.build();

    // Then: build succeeds with LeastConnections strategy
    assert!(result.is_ok());
    let strategy_service = result.expect("Failed to build strategy");
    assert!(matches!(
        strategy_service.strategy(),
        Strategy::LeastConnections
    ));
}

#[test]
fn strategy_builder_build_round_robin_should_succeed() {
    // Given: a StrategyBuilder with RoundRobin strategy
    let builder = StrategyBuilder::new().with_strategy(Strategy::RoundRobin);

    // When: building the strategy
    let result = builder.build();

    // Then: build succeeds with RoundRobin strategy
    assert!(result.is_ok());
    let strategy_service = result.expect("Failed to build strategy");
    assert!(matches!(strategy_service.strategy(), Strategy::RoundRobin));
}

#[test]
fn strategy_builder_build_weighted_round_robin_should_succeed() {
    // Given: a StrategyBuilder with WeightedRoundRobin strategy
    let builder = StrategyBuilder::new().with_strategy(Strategy::WeightedRoundRobin);

    // When: building the strategy
    let result = builder.build();

    // Then: build succeeds with WeightedRoundRobin strategy
    assert!(result.is_ok());
    let strategy_service = result.expect("Failed to build strategy");
    assert!(matches!(
        strategy_service.strategy(),
        Strategy::WeightedRoundRobin
    ));
}

#[test]
fn strategy_builder_build_without_strategy_should_fail() {
    // Given: a StrategyBuilder without strategy
    let builder = StrategyBuilder::new();

    // When: building without strategy
    let result = builder.build();

    // Then: build fails with NotFound error
    assert!(result.is_err());
    if let Err(StrategyError::NotFound(msg)) = result {
        assert_eq!(msg, "Strategy not found");
    } else {
        panic!("Expected NotFound error");
    }
}

#[test]
fn strategy_builder_chaining_should_succeed() {
    // Given: a StrategyBuilder
    // When: chaining with_strategy then build (backends come from Context at runtime)
    let result = StrategyBuilder::new()
        .with_strategy(Strategy::RoundRobin)
        .build();

    // Then: builder builds successfully
    assert!(result.is_ok());
}

#[test]
fn strategy_builder_debug_should_succeed() {
    // Given: a StrategyBuilder
    let builder = StrategyBuilder::new().with_strategy(Strategy::RoundRobin);

    // When: formatting with Debug
    let debug_str = format!("{:?}", builder);

    // Then: debug string is not empty
    assert!(!debug_str.is_empty());
}
