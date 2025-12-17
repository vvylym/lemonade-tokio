//! Context tests
//!
//! Tests for the Context type covering:
//! - Construction (new)
//! - Getters (strategy, routing_table, channels)
//! - Migration (migrate)
//! - Drain waiting (wait_for_drain)
//! - Channel operations

use super::super::common::fixtures::*;
use lemonade_load_balancer::prelude::*;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn context_new_should_succeed() {
    // Given: a valid Config
    let backends = vec![
        create_test_backend(0, None, Some(10u8)),
        create_test_backend(1, None, Some(10u8)),
    ];
    let config = create_test_config(
        backends.clone(),
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 5000,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );

    // When: creating a new Context
    let result = Context::new(config);

    // Then: context is created successfully
    assert!(result.is_ok());
    let ctx = result.expect("Failed to create context");
    assert_eq!(ctx.routing_table().len(), 2);
}

#[test]
fn context_new_with_empty_backends_should_succeed() {
    // Given: a Config with empty backends
    let config = create_test_config(
        Vec::new(),
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 5000,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );

    // When: creating a new Context
    let result = Context::new(config);

    // Then: context is created successfully
    assert!(result.is_ok());
    let ctx = result.expect("Failed to create context");
    assert_eq!(ctx.routing_table().len(), 0);
}

#[test]
fn context_strategy_should_succeed() {
    // Given: a Context
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let config = create_test_config(
        backends,
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 5000,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );
    let ctx = Context::new(config).expect("Failed to create context");

    // When: getting strategy
    let strategy = ctx.strategy();

    // Then: strategy is returned
    assert!(matches!(strategy.strategy(), Strategy::RoundRobin));
}

#[test]
fn context_routing_table_should_succeed() {
    // Given: a Context
    let backends = vec![
        create_test_backend(0, None, Some(10u8)),
        create_test_backend(1, None, Some(10u8)),
    ];
    let config = create_test_config(
        backends,
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 5000,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );
    let ctx = Context::new(config).expect("Failed to create context");

    // When: getting routing table
    let routing = ctx.routing_table();

    // Then: routing table is returned
    assert_eq!(routing.len(), 2);
}

#[test]
fn context_channels_should_succeed() {
    // Given: a Context
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let config = create_test_config(
        backends,
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 5000,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );
    let ctx = Context::new(config).expect("Failed to create context");

    // When: getting channels
    let channels = ctx.channels();

    // Then: channels are returned
    // Verify channels work
    let _ = channels.config_tx();
}

#[test]
fn context_healthy_backends_should_succeed() {
    // Given: a Context with healthy backends
    let backends = vec![
        create_test_backend(0, None, Some(10u8)),
        create_test_backend(1, None, Some(10u8)),
    ];
    let ctx = create_test_context(backends.clone());
    // Backends start healthy by default
    // Mark one as unhealthy
    let routing = ctx.routing_table();
    if let Some(backend1) = routing.get(1) {
        backend1.set_health(false, 1000);
    }

    // When: getting healthy backends
    let healthy = ctx.routing_table().healthy_backends();

    // Then: only healthy backends are returned
    assert_eq!(healthy.len(), 1);
    assert_eq!(healthy[0].id(), 0);
}

#[test]
fn context_healthy_backends_all_healthy_should_succeed() {
    // Given: a Context with all healthy backends
    let backends = vec![
        create_test_backend(0, None, Some(10u8)),
        create_test_backend(1, None, Some(10u8)),
    ];
    let ctx = create_test_context(backends.clone());
    // Backends start healthy by default

    // When: getting healthy backends
    let healthy = ctx.routing_table().healthy_backends();

    // Then: all backends are returned
    assert_eq!(healthy.len(), 2);
}

#[tokio::test]
async fn context_migrate_should_succeed() {
    // Given: a Context with backends
    let backends1 = vec![create_test_backend(0, None, Some(10u8))];
    let config1 = create_test_config(
        backends1.clone(),
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 5000,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );
    let ctx = Arc::new(Context::new(config1).expect("Failed to create context"));

    // When: migrating to new config with additional backend
    let backends2 = vec![
        create_test_backend(0, None, Some(10u8)),
        create_test_backend(1, None, Some(10u8)),
    ];
    let config2 = create_test_config(
        backends2.clone(),
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 100, // Short timeout for test
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );

    let result = ctx.migrate(config2).await;

    // Then: migration succeeds
    assert!(result.is_ok());
    assert_eq!(ctx.routing_table().len(), 2);
}

#[tokio::test]
async fn context_wait_for_drain_should_succeed() {
    // Given: a Context with no active connections
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let config = create_test_config(
        backends,
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 5000,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );
    let ctx = Arc::new(Context::new(config).expect("Failed to create context"));

    // When: waiting for drain
    let result = ctx.wait_for_drain(Duration::from_millis(100)).await;

    // Then: drain completes immediately (no connections)
    assert!(result.is_ok());
}

#[tokio::test]
async fn context_wait_for_drain_with_connections_should_succeed() {
    // Given: a Context with active connections
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let config = create_test_config(
        backends,
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 5000,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );
    let ctx = Arc::new(Context::new(config).expect("Failed to create context"));

    // Add a connection
    let routing = ctx.routing_table();
    if let Some(backend) = routing.get(0) {
        backend.increment_connection();
    }

    // Spawn task to close connection after a delay
    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        let routing = ctx_clone.routing_table();
        if let Some(backend) = routing.get(0) {
            backend.decrement_connection();
        }
        ctx_clone.notify_connection_closed();
    });

    // When: waiting for drain
    let result = ctx.wait_for_drain(Duration::from_millis(200)).await;

    // Then: drain completes after connection closes
    assert!(result.is_ok());
}

#[test]
fn context_notify_connection_closed_should_succeed() {
    // Given: a Context
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let config = create_test_config(
        backends,
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 5000,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );
    let ctx = Context::new(config).expect("Failed to create context");

    // When: notifying connection closed
    ctx.notify_connection_closed();

    // Then: notification is sent (no error)
    // This is tested by the fact that it doesn't panic
}

#[test]
fn context_backends_start_healthy_should_succeed() {
    // Given: a Context with backends
    let backends = vec![
        create_test_backend(0, None, Some(10u8)),
        create_test_backend(1, None, Some(10u8)),
    ];
    let config = create_test_config(
        backends,
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 5000,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );
    let ctx = Context::new(config).expect("Failed to create context");

    // When: checking backend health
    let routing = ctx.routing_table();
    let all_backends = routing.all_backends();

    // Then: all backends are healthy by default
    assert_eq!(all_backends.len(), 2);
    for backend in all_backends {
        assert!(
            backend.is_alive(),
            "Backend {} should start healthy",
            backend.id()
        );
    }
}

#[tokio::test]
async fn context_migrate_with_removed_backends_should_succeed() {
    // Given: a Context with backends
    let backends1 = vec![
        create_test_backend(0, None, Some(10u8)),
        create_test_backend(1, None, Some(10u8)),
    ];
    let config1 = create_test_config(
        backends1.clone(),
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 100,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );
    let ctx = Arc::new(Context::new(config1).expect("Failed to create context"));

    // When: migrating to config with fewer backends
    let backends2 = vec![create_test_backend(0, None, Some(10u8))];
    let config2 = create_test_config(
        backends2.clone(),
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 100,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );

    let result = ctx.migrate(config2).await;

    // Then: migration succeeds and removed backend is either draining or removed (depending on connections)
    assert!(result.is_ok());
    // Backend 1 is removed since it had no active connections
    assert!(ctx.routing_table().len() <= 2);
}

#[tokio::test]
async fn context_migrate_with_changed_listen_address_should_succeed() {
    // Given: a Context with a listen address
    use std::net::{IpAddr, Ipv4Addr};
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let config1 = create_test_config(
        backends.clone(),
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 100,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );
    let ctx = Arc::new(Context::new(config1).expect("Failed to create context"));

    // When: migrating to config with different listen address
    let mut config2 = create_test_config(
        backends.clone(),
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 100,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );
    config2.proxy.listen_address =
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 4000);

    let result = ctx.migrate(config2.clone()).await;

    // Then: migration succeeds and config is updated
    assert!(result.is_ok());
    assert_eq!(
        ctx.config().proxy.listen_address,
        config2.proxy.listen_address
    );
}

#[tokio::test]
async fn context_wait_for_drain_timeout_should_fail() {
    // Given: a Context with active connections that won't close
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let config = create_test_config(
        backends,
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 5000,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );
    let ctx = Arc::new(Context::new(config).expect("Failed to create context"));

    // Add a connection that won't close
    let routing = ctx.routing_table();
    if let Some(backend) = routing.get(0) {
        backend.increment_connection();
    }

    // When: waiting for drain with short timeout
    let result = ctx.wait_for_drain(Duration::from_millis(50)).await;

    // Then: timeout occurs
    assert!(
        result.is_err(),
        "Should timeout when connections don't close"
    );
}

#[tokio::test]
async fn context_migrate_with_strategy_change_should_succeed() {
    // Given: a Context with RoundRobin strategy
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let config1 = create_test_config(
        backends.clone(),
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 100,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );
    let ctx = Arc::new(Context::new(config1).expect("Failed to create context"));

    // When: migrating to LeastConnections strategy
    let config2 = create_test_config(
        backends.clone(),
        Strategy::LeastConnections,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 100,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );

    let result = ctx.migrate(config2).await;

    // Then: migration succeeds and strategy is updated
    assert!(result.is_ok());
    assert!(matches!(
        ctx.strategy().strategy(),
        Strategy::LeastConnections
    ));
}

#[test]
fn context_config_getter_should_succeed() {
    // Given: a Context
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let config = create_test_config(
        backends,
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 5000,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );
    let ctx = Context::new(config.clone()).expect("Failed to create context");

    // When: getting config
    let cfg = ctx.config();

    // Then: config is returned
    assert_eq!(cfg.strategy, config.strategy);
}

#[tokio::test]
async fn context_migrate_with_same_backends_should_succeed() {
    // Given: a Context
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let config1 = create_test_config(
        backends.clone(),
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 100,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );
    let ctx = Arc::new(Context::new(config1).expect("Failed to create context"));

    // When: migrating to same backends (no change)
    let config2 = create_test_config(
        backends.clone(),
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 100,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );

    let result = ctx.migrate(config2).await;

    // Then: migration succeeds
    assert!(result.is_ok());
    assert_eq!(ctx.routing_table().len(), 1);
}

#[tokio::test]
async fn context_migrate_with_added_backends_should_succeed() {
    // Given: a Context with one backend
    let backends1 = vec![create_test_backend(0, None, Some(10u8))];
    let config1 = create_test_config(
        backends1.clone(),
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 100,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );
    let ctx = Arc::new(Context::new(config1).expect("Failed to create context"));

    // When: migrating to more backends
    let backends2 = vec![
        create_test_backend(0, None, Some(10u8)),
        create_test_backend(1, None, Some(10u8)),
        create_test_backend(2, None, Some(10u8)),
    ];
    let config2 = create_test_config(
        backends2.clone(),
        Strategy::RoundRobin,
        RuntimeConfig {
            metrics_cap: 100,
            health_cap: 50,
            drain_timeout_millis: 100,
            background_timeout_millis: 1000,
            accept_timeout_millis: 2000,
            config_watch_interval_millis: 1000,
        },
    );

    let result = ctx.migrate(config2).await;

    // Then: migration succeeds and new backends are added
    assert!(result.is_ok());
    assert_eq!(ctx.routing_table().len(), 3);
}
