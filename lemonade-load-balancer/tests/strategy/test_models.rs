//! Strategy models tests
//!
use lemonade_load_balancer::prelude::*;
use rstest::*;

#[rstest]
#[case("adaptive", Strategy::Adaptive)]
#[case("fastest_response_time", Strategy::FastestResponseTime)]
#[case("least_connections", Strategy::LeastConnections)]
#[case("round_robin", Strategy::RoundRobin)]
#[case("weighted_round_robin", Strategy::WeightedRoundRobin)]
fn strategy_from_str_should_succeed(#[case] input: &str, #[case] expected: Strategy) {
    let result = input.parse::<Strategy>();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected);
}

#[rstest]
#[case("invalid_strategy")]
#[case("")]
#[case("ADAPTIVE")] // case sensitive
fn strategy_from_str_invalid_should_fail(#[case] input: &str) {
    let result = input.parse::<Strategy>();
    assert!(result.is_err());
}

#[rstest]
#[case(Strategy::Adaptive, "adaptive")]
#[case(Strategy::FastestResponseTime, "fastest_response_time")]
#[case(Strategy::LeastConnections, "least_connections")]
#[case(Strategy::RoundRobin, "round_robin")]
#[case(Strategy::WeightedRoundRobin, "weighted_round_robin")]
fn strategy_as_ref_should_succeed(#[case] strategy: Strategy, #[case] expected: &str) {
    assert_eq!(strategy.as_ref(), expected);
}

#[rstest]
#[case(Strategy::Adaptive)]
#[case(Strategy::FastestResponseTime)]
#[case(Strategy::LeastConnections)]
#[case(Strategy::RoundRobin)]
#[case(Strategy::WeightedRoundRobin)]
fn strategy_clone_should_succeed(#[case] strategy: Strategy) {
    let cloned = strategy.clone();
    assert_eq!(strategy, cloned);
}

#[test]
fn strategy_debug_should_succeed() {
    let strategy = Strategy::Adaptive;
    let debug_str = format!("{:?}", strategy);
    assert!(!debug_str.is_empty());
}

#[test]
fn strategy_round_trip_parse_all_variants() {
    let strategies = vec![
        Strategy::Adaptive,
        Strategy::FastestResponseTime,
        Strategy::LeastConnections,
        Strategy::RoundRobin,
        Strategy::WeightedRoundRobin,
    ];

    for strategy in strategies {
        let s = strategy.as_ref();
        let parsed = s.parse::<Strategy>().expect("Should parse");
        assert_eq!(strategy, parsed);
    }
}
