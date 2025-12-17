//! Tests for ConfigBuilder

use lemonade_load_balancer::prelude::{ConfigBuilder, ConfigSource, Strategy};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn config_builder_from_env_with_custom_values_should_succeed() {
    let result = ConfigBuilder::from_env();
    assert!(
        result.is_ok(),
        "Should build with custom values: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.source, ConfigSource::Environment);
    assert_eq!(config.proxy.listen_address.to_string(), "0.0.0.0:8080");
    assert_eq!(config.strategy, Strategy::LeastConnections);
    // Note: backends from env are NOT parsed - this is expected behavior
    // The from_env() method is meant for basic config, backends should come from file
    assert_eq!(config.backends.len(), 0, "Backends from env not supported");
    assert_eq!(config.proxy.max_connections, Some(50000));
    assert_eq!(config.runtime.metrics_cap, 5000);
    assert_eq!(config.runtime.health_cap, 500);
    assert_eq!(config.runtime.drain_timeout_millis, 10000);
    assert_eq!(config.runtime.background_timeout_millis, 5000);
    assert_eq!(config.runtime.accept_timeout_millis, 3000);
    assert_eq!(config.health.interval.as_millis(), 15000);
    assert_eq!(config.health.timeout.as_millis(), 4000);
    assert_eq!(config.metrics.interval.as_millis(), 20000);
    assert_eq!(config.metrics.timeout.as_millis(), 5000);
}

#[test]
fn config_builder_from_file_toml_should_succeed() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let config_content = r#"
strategy = "adaptive"

[runtime]
metrics_cap = 2000
health_cap = 200
drain_timeout_millis = 8000
background_timeout_millis = 4000
accept_timeout_millis = 2000
config_watch_interval_millis = 500

[proxy]
listen_address = "127.0.0.1:9000"
max_connections = 20000

[[backends]]
id = 0
name = "test-backend-1"
address = "127.0.0.1:10001"
weight = 5

[[backends]]
id = 1
name = "test-backend-2"
address = "127.0.0.1:10002"
weight = 10

[health]
interval = 20000
timeout = 5000

[metrics]
interval = 15000
timeout = 3000
"#;

    fs::write(&config_path, config_content).unwrap();

    let result = ConfigBuilder::from_file(Some(config_path));
    if let Err(ref e) = result {
        eprintln!("Config load error: {:?}", e);
    }
    assert!(
        result.is_ok(),
        "Should load TOML config: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.source, ConfigSource::File);
    assert_eq!(config.runtime.metrics_cap, 2000);
    assert_eq!(config.runtime.health_cap, 200);
    assert_eq!(config.proxy.listen_address.to_string(), "127.0.0.1:9000");
    assert_eq!(config.strategy, Strategy::Adaptive);
    assert_eq!(config.backends.len(), 2);
    assert_eq!(config.backends[0].name, Some("test-backend-1".to_string()));
    assert_eq!(config.backends[0].weight, Some(5));
    assert_eq!(config.backends[1].weight, Some(10));
}

#[test]
fn config_builder_from_file_json_should_succeed() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let config_content = r#"{
  "runtime": {
    "metrics_cap": 3000,
    "health_cap": 300,
    "drain_timeout_millis": 12000,
    "background_timeout_millis": 6000,
    "accept_timeout_millis": 3000,
    "config_watch_interval_millis": 1000
  },
  "proxy": {
    "listen_address": "0.0.0.0:7000",
    "max_connections": 30000
  },
  "strategy": "fastest_response_time",
  "backends": [
    {
      "id": 0,
      "name": "json-backend-1",
      "address": "127.0.0.1:11001"
    },
    {
      "id": 1,
      "address": "127.0.0.1:11002"
    }
  ],
  "health": {
    "interval": 25000,
    "timeout": 6000
  },
  "metrics": {
    "interval": 18000,
    "timeout": 4000
  }
}"#;

    fs::write(&config_path, config_content).unwrap();

    let result = ConfigBuilder::from_file(Some(config_path));
    if let Err(ref e) = result {
        eprintln!("JSON Config load error: {:?}", e);
    }
    assert!(
        result.is_ok(),
        "Should load JSON config: {:?}",
        result.err()
    );

    let config = result.unwrap();
    assert_eq!(config.source, ConfigSource::File);
    assert_eq!(config.runtime.metrics_cap, 3000);
    assert_eq!(config.strategy, Strategy::FastestResponseTime);
    assert_eq!(config.backends.len(), 2);
    assert_eq!(config.backends[0].name, Some("json-backend-1".to_string()));
    assert_eq!(config.backends[1].name, None);
}

#[test]
fn config_builder_from_file_nonexistent_should_fail() {
    let result =
        ConfigBuilder::from_file(Some(PathBuf::from("/nonexistent/path/config.toml")));
    assert!(result.is_err(), "Should fail for nonexistent file");
}

#[test]
fn config_builder_from_file_invalid_toml_should_fail() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.toml");

    fs::write(&config_path, "invalid toml content {]").unwrap();

    let result = ConfigBuilder::from_file(Some(config_path));
    assert!(result.is_err(), "Should fail for invalid TOML");
}

#[test]
fn config_builder_from_file_invalid_json_should_fail() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.json");

    fs::write(&config_path, "{invalid json}").unwrap();

    let result = ConfigBuilder::from_file(Some(config_path));
    assert!(result.is_err(), "Should fail for invalid JSON");
}

#[test]
fn config_builder_from_file_none_defaults_should_succeed() {
    // When None is passed, it should fall back to environment variables
    let result = ConfigBuilder::from_file(None::<PathBuf>);
    assert!(
        result.is_ok(),
        "Should succeed with None and use env defaults"
    );

    let config = result.unwrap();
    // Should use environment-based defaults since no file was provided
    assert_eq!(config.source, ConfigSource::Environment);
}
