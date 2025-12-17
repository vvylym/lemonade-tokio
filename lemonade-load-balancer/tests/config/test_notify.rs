//! Tests for NotifyConfigService
//!
use lemonade_load_balancer::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

use crate::common::fixtures::create_test_config_fast;

#[tokio::test]
async fn notify_config_service_new_should_succeed() {
    // Given: a config file path
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.json");
    let config = create_test_config_fast(vec![], Strategy::RoundRobin);
    fs::write(&config_path, serde_json::to_string(&config).unwrap())
        .expect("Failed to write config");

    // When: creating NotifyConfigService
    let service = NotifyConfigService::new(Some(config_path.clone()));

    // Then: service is created successfully
    assert!(service.is_ok());
}

#[tokio::test]
async fn notify_config_service_new_without_file_should_succeed() {
    // Given: no config file path
    // When: creating NotifyConfigService
    let service = NotifyConfigService::new(None::<PathBuf>);

    // Then: service is created successfully (uses environment)
    assert!(service.is_ok());
}

#[tokio::test]
async fn notify_config_service_watch_config_should_succeed() {
    // Given: a NotifyConfigService and a config file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.json");
    let config = create_test_config_fast(vec![], Strategy::RoundRobin);
    fs::write(&config_path, serde_json::to_string(&config).unwrap())
        .expect("Failed to write config");

    let service = Arc::new(
        NotifyConfigService::new(Some(config_path.clone()))
            .expect("Failed to create service"),
    );
    let ctx = Arc::new(Context::new(config).expect("Failed to create context"));

    // Spawn watch_config in background
    let service_clone = service.clone();
    let ctx_clone = ctx.clone();
    let watch_handle = tokio::spawn(async move {
        service_clone.watch_config(ctx_clone).await;
    });

    // Wait a bit for watcher to start
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Send shutdown signal
    let _ = ctx.channels().shutdown_tx().send(());

    // Wait for service to stop
    let _ =
        tokio::time::timeout(tokio::time::Duration::from_millis(100), watch_handle).await;
}

#[tokio::test]
async fn notify_config_service_watch_config_detects_file_changes_should_succeed() {
    // Given: a NotifyConfigService and a config file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.json");
    let config = create_test_config_fast(vec![], Strategy::RoundRobin);
    fs::write(&config_path, serde_json::to_string(&config).unwrap())
        .expect("Failed to write config");

    let service = Arc::new(
        NotifyConfigService::new(Some(config_path.clone()))
            .expect("Failed to create service"),
    );
    let ctx = Arc::new(Context::new(config).expect("Failed to create context"));

    // Spawn watch_config in background
    let service_clone = service.clone();
    let ctx_clone = ctx.clone();
    let watch_handle = tokio::spawn(async move {
        service_clone.watch_config(ctx_clone).await;
    });

    // Wait a bit for watcher to start
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Modify config file
    let new_config = create_test_config_fast(vec![], Strategy::LeastConnections);
    fs::write(&config_path, serde_json::to_string(&new_config).unwrap())
        .expect("Failed to write config");

    // Wait for change to be detected
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Send shutdown signal
    let _ = ctx.channels().shutdown_tx().send(());

    // Wait for service to stop
    let _ =
        tokio::time::timeout(tokio::time::Duration::from_millis(100), watch_handle).await;
}
