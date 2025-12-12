//! Common test utilities and fixtures
//!
use lemonade_service::config::Config;
use proptest::prelude::*;
use std::{net::SocketAddr, time::Duration};

/// Generate a valid service name strategy
pub fn service_name_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_-]{5,10}"
}

/// Generate a valid duration strategy (non-zero, 1-5ms for fast tests)
pub fn duration_strategy() -> impl Strategy<Value = Duration> {
    (1u64..=5u64).prop_map(Duration::from_millis)
}

/// Generate a valid Config strategy
pub fn config_strategy() -> impl Strategy<Value = Config> {
    (service_name_strategy(), duration_strategy()).prop_map(|(name, delay)| {
        Config::new(SocketAddr::from(([127, 0, 0, 1], 8080)), name, delay)
    })
}

/// Generate a status string strategy
#[allow(dead_code)] // Used in proptest! macros
pub fn status_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_-]{1,20}"
}

/// Generate a service name string strategy for responses
#[allow(dead_code)] // Used in proptest! macros
pub fn response_service_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_-]{5,10}"
}

/// Generate a duration_ms strategy (u64, 1-5ms for fast tests)
#[allow(dead_code)] // Used in proptest! macros
pub fn duration_ms_strategy() -> impl Strategy<Value = u64> {
    1u64..=5u64
}

#[test]
fn test_config_strategy_generates_valid_configs() {
    let mut runner = proptest::test_runner::TestRunner::default();
    runner
        .run(&config_strategy(), |config| {
            prop_assert!(!config.service_name().is_empty());
            prop_assert!(!config.work_delay().is_zero());
            Ok(())
        })
        .unwrap();
}
