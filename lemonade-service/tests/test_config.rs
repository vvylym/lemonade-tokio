//! Tests for the config module
//!
use lemonade_service::config::{Config, ConfigBuilder, WorkerAddress};
use proptest::prelude::*;
use std::time::Duration;

mod common;
use common::*;

#[test]
fn config_new_with_valid_fields() {
    let config = Config::new(
        WorkerAddress::parse("127.0.0.1:8080").unwrap(),
        "test-service",
        Duration::from_millis(1),
    );

    assert_eq!(config.service_name(), "test-service");
    assert_eq!(config.work_delay(), Duration::from_millis(1));
}

#[test]
fn config_round_trip_serialization_json() {
    let config = Config::new(
        WorkerAddress::parse("127.0.0.1:8080").unwrap(),
        "test-service",
        Duration::from_millis(1),
    );

    let json = serde_json::to_string(&config).expect("Serialization should succeed");
    let deserialized: Config =
        serde_json::from_str(&json).expect("Deserialization should succeed");

    assert_eq!(config.service_name(), deserialized.service_name());
    assert_eq!(config.work_delay(), deserialized.work_delay());
}

#[test]
fn config_round_trip_serialization_toml() {
    let config = Config::new(
        WorkerAddress::parse("127.0.0.1:8080").unwrap(),
        "test-service",
        Duration::from_millis(1),
    );

    let toml = toml::to_string(&config).expect("Serialization should succeed");
    let deserialized: Config =
        toml::from_str(&toml).expect("Deserialization should succeed");

    assert_eq!(config.service_name(), deserialized.service_name());
    assert_eq!(config.work_delay(), deserialized.work_delay());
}

#[test]
fn config_builder_from_env() {
    // Test that from_env works with defaults when env vars are not set
    // Note: We can't safely remove env vars in tests, so we test the default behavior
    let config = ConfigBuilder::from_env().expect("Should load from env");
    // Verify it returns a valid config (either from env or defaults)
    assert!(!config.service_name().is_empty());
    assert!(!config.work_delay().is_zero());
}

// Property-based tests
proptest! {
    #[test]
    fn config_round_trip_serialization_property(config in config_strategy()) {
        let json = serde_json::to_string(&config)
            .expect("Serialization should succeed");
        let deserialized: Config = serde_json::from_str(&json)
            .expect("Deserialization should succeed");
        prop_assert_eq!(config.service_name(), deserialized.service_name());
        prop_assert_eq!(config.work_delay(), deserialized.work_delay());
    }

    #[test]
    fn config_clone_property(config in config_strategy()) {
        let cloned = config.clone();
        prop_assert_eq!(config.service_name(), cloned.service_name());
        prop_assert_eq!(config.work_delay(), cloned.work_delay());
    }
}
