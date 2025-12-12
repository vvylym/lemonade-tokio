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
async fn notify_config_service_snapshot_should_return_config() {
    // Given: a NotifyConfigService with loaded config
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.json");
    let config = create_test_config_fast(vec![], Strategy::RoundRobin);
    fs::write(&config_path, serde_json::to_string(&config).unwrap())
        .expect("Failed to write config");

    let service =
        NotifyConfigService::new(Some(config_path)).expect("Failed to create service");

    // When: calling snapshot()
    let snapshot = service.snapshot();

    // Then: returns current config
    assert_eq!(snapshot.strategy, config.strategy);
    assert_eq!(snapshot.backends.len(), config.backends.len());
}

#[tokio::test]
async fn notify_config_service_start_watches_file_should_succeed() {
    // Given: a NotifyConfigService and a config file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.json");
    let config = create_test_config_fast(vec![], Strategy::RoundRobin);
    fs::write(&config_path, serde_json::to_string(&config).unwrap())
        .expect("Failed to write config");

    let service = NotifyConfigService::new(Some(config_path.clone()))
        .expect("Failed to create service");
    let ctx = Arc::new(Context::new(&config).expect("Failed to create context"));

    // When: starting the service
    let start_result = service.start(ctx.clone()).await;

    // Then: file watcher is active
    assert!(start_result.is_ok());

    // Cleanup
    service.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
async fn notify_config_service_file_change_updates_context_should_succeed() {
    // Given: a running NotifyConfigService and context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.json");
    let initial_config = create_test_config_fast(vec![], Strategy::RoundRobin);
    fs::write(
        &config_path,
        serde_json::to_string(&initial_config).unwrap(),
    )
    .expect("Failed to write config");

    let service = NotifyConfigService::new(Some(config_path.clone()))
        .expect("Failed to create service");
    let ctx = Arc::new(Context::new(&initial_config).expect("Failed to create context"));

    service
        .start(ctx.clone())
        .await
        .expect("Failed to start service");

    // Wait a bit for watcher to initialize
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // When: config file changes
    let new_backend = create_test_backend(0, None, Some(10u8));
    let mut new_config = initial_config.clone();
    new_config.backends.push(new_backend);
    fs::write(&config_path, serde_json::to_string(&new_config).unwrap())
        .expect("Failed to write new config");

    // Wait for file change to be detected and processed
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Then: context is updated with new config
    let routing = ctx.routing_table();
    assert_eq!(routing.len(), 1); // Should have one backend now

    // Cleanup
    service.shutdown().await.expect("Failed to shutdown");
}

#[tokio::test]
async fn notify_config_service_shutdown_should_succeed() {
    // Given: a running NotifyConfigService
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.json");
    let config = create_test_config_fast(vec![], Strategy::RoundRobin);
    fs::write(&config_path, serde_json::to_string(&config).unwrap())
        .expect("Failed to write config");

    let service =
        NotifyConfigService::new(Some(config_path)).expect("Failed to create service");
    let ctx = Arc::new(Context::new(&config).expect("Failed to create context"));
    service.start(ctx).await.expect("Failed to start service");

    // When: calling shutdown()
    let shutdown_result = service.shutdown().await;

    // Then: service stops gracefully
    assert!(shutdown_result.is_ok());
}

#[tokio::test]
async fn notify_config_service_invalid_file_should_fail() {
    // Given: a non-existent config file path
    let config_path = PathBuf::from("/nonexistent/path/config.json");

    // When: creating NotifyConfigService
    let service = NotifyConfigService::new(Some(config_path));

    // Then: service creation fails
    assert!(service.is_err());
}

// Helper function for tests
use crate::common::fixtures::create_test_backend;
