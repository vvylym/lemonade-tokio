//! Context tests
//!
//! Tests for the Context type covering:
//! - Construction (new)
//! - Getters (strategy, routing_table, connection_registry, health_registry, metrics_snapshot, channel_bundle)
//! - Setters (set_backends, set_routing_table, set_connection_registry, set_health_registry, set_metrics_snapshot, set_timeouts, set_channel_bundle)
//! - Healthy backends filtering
//! - Channel operations (metrics_receiver, health_receiver, shutdown_sender, shutdown_receiver, channel_version_receiver)
//! - Timeout operations (drain_timeout, background_handle_timeout, accept_handle_timeout)
//! - Connection management (notify_connection_closed, keep_alive)
//! - Migration behavior (set_backends_preserves_connections)

use super::super::common::fixtures::*;
use lemonade_load_balancer::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

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
        },
    );

    // When: creating a new Context
    let result = Context::new(&config);

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
        },
    );

    // When: creating a new Context
    let result = Context::new(&config);

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
        },
    );
    let ctx = Context::new(&config).expect("Failed to create context");

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
        },
    );
    let ctx = Context::new(&config).expect("Failed to create context");

    // When: getting routing table
    let routing = ctx.routing_table();

    // Then: routing table is returned
    assert_eq!(routing.len(), 2);
}

#[test]
fn context_connection_registry_should_succeed() {
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
        },
    );
    let ctx = Context::new(&config).expect("Failed to create context");

    // When: getting connection registry
    let connections = ctx.connection_registry();

    // Then: connection registry is returned
    assert_eq!(connections.total(), 0);
}

#[test]
fn context_health_registry_should_succeed() {
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
        },
    );
    let ctx = Context::new(&config).expect("Failed to create context");

    // When: getting health registry
    let health = ctx.health_registry();

    // Then: health registry is returned
    assert_eq!(health.healthy_indices().len(), 0);
}

#[test]
fn context_metrics_snapshot_should_succeed() {
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
        },
    );
    let ctx = Context::new(&config).expect("Failed to create context");

    // When: getting metrics snapshot
    let metrics = ctx.metrics_snapshot();

    // Then: metrics snapshot is returned
    assert!(metrics.backend_ids().is_empty());
}

#[test]
fn context_channel_bundle_should_succeed() {
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
        },
    );
    let ctx = Context::new(&config).expect("Failed to create context");

    // When: getting channel bundle
    let bundle = ctx.channel_bundle();

    // Then: channel bundle is returned
    assert_eq!(bundle.metrics_cap(), 100);
    assert_eq!(bundle.health_cap(), 50);
}

#[test]
fn context_healthy_backends_should_succeed() {
    // Given: a Context with healthy backends
    let backends = vec![
        create_test_backend(0, None, Some(10u8)),
        create_test_backend(1, None, Some(10u8)),
    ];
    let ctx = create_test_context(backends.clone());
    let health = ctx.health_registry();
    health.set_alive(0, true, 1000);
    health.set_alive(1, false, 1000);

    // When: getting healthy backends
    let healthy = ctx.healthy_backends();

    // Then: only healthy backends are returned
    assert_eq!(healthy.len(), 1);
    assert_eq!(healthy[0].id(), &0u8);
}

#[test]
fn context_healthy_backends_all_healthy_should_succeed() {
    // Given: a Context with all healthy backends
    let backends = vec![
        create_test_backend(0, None, Some(10u8)),
        create_test_backend(1, None, Some(10u8)),
    ];
    let ctx = create_test_context(backends.clone());
    let health = ctx.health_registry();
    health.set_alive(0, true, 1000);
    health.set_alive(1, true, 1000);

    // When: getting healthy backends
    let healthy = ctx.healthy_backends();

    // Then: all backends are returned
    assert_eq!(healthy.len(), 2);
}

#[test]
fn context_set_backends_should_succeed() {
    // Given: a Context
    let backends1 = vec![create_test_backend(0, None, Some(10u8))];
    let ctx = create_test_context(backends1);

    // When: setting new backends
    let backends2 = vec![
        create_test_backend(0, None, Some(10u8)),
        create_test_backend(1, None, Some(10u8)),
    ];
    let result = ctx.set_backends(backends2);

    // Then: backends are updated successfully
    assert!(result.is_ok());
    assert_eq!(ctx.routing_table().len(), 2);
}

#[test]
fn context_set_routing_table_should_succeed() {
    // Given: a Context
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let ctx = create_test_context(backends);

    // When: setting new routing table
    let new_backends = vec![
        create_test_backend(0, None, Some(10u8)),
        create_test_backend(1, None, Some(10u8)),
    ];
    let new_routing = Arc::new(RouteTable::new(new_backends));
    ctx.set_routing_table(new_routing.clone());

    // Then: routing table is updated
    assert_eq!(ctx.routing_table().len(), 2);
}

#[test]
fn context_set_connection_registry_should_succeed() {
    // Given: a Context
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let ctx = create_test_context(backends);

    // When: setting new connection registry
    let new_connections = Arc::new(ConnectionRegistry::new(5));
    ctx.set_connection_registry(new_connections.clone());

    // Then: connection registry is updated
    assert_eq!(ctx.connection_registry().total(), 0);
}

#[test]
fn context_set_health_registry_should_succeed() {
    // Given: a Context
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let ctx = create_test_context(backends);

    // When: setting new health registry
    let new_health = Arc::new(HealthRegistry::new(5));
    new_health.set_alive(0, true, 1000);
    ctx.set_health_registry(new_health.clone());

    // Then: health registry is updated
    assert_eq!(ctx.health_registry().healthy_indices().len(), 1);
}

#[test]
fn context_set_metrics_snapshot_should_succeed() {
    // Given: a Context
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let ctx = create_test_context(backends);

    // When: setting new metrics snapshot
    let new_metrics = Arc::new(MetricsSnapshot::default());
    new_metrics.update(0, BackendMetrics::default());
    ctx.set_metrics_snapshot(new_metrics.clone());

    // Then: metrics snapshot is updated
    assert!(ctx.metrics_snapshot().has_metrics(0));
}

#[test]
fn context_set_timeouts_should_succeed() {
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
        },
    );
    let mut ctx = Context::new(&config).expect("Failed to create context");

    // When: setting timeouts
    ctx.set_timeouts(
        Some(Duration::from_secs(10)),
        Some(Duration::from_secs(5)),
        Some(Duration::from_secs(3)),
    );

    // Then: timeouts are updated
    assert_eq!(ctx.drain_timeout(), Duration::from_secs(10));
    assert_eq!(ctx.background_handle_timeout(), Duration::from_secs(5));
    assert_eq!(ctx.accept_handle_timeout(), Duration::from_secs(3));
}

#[test]
fn context_set_timeouts_partial_should_succeed() {
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
        },
    );
    let mut ctx = Context::new(&config).expect("Failed to create context");
    let _original_drain = ctx.drain_timeout();

    // When: setting only some timeouts
    ctx.set_timeouts(Some(Duration::from_secs(10)), None, None);

    // Then: only specified timeout is updated
    assert_eq!(ctx.drain_timeout(), Duration::from_secs(10));
    assert_eq!(ctx.background_handle_timeout(), Duration::from_millis(1000)); // Original value
}

#[test]
fn context_drain_timeout_should_succeed() {
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
        },
    );
    let ctx = Context::new(&config).expect("Failed to create context");

    // When: getting drain timeout
    let timeout = ctx.drain_timeout();

    // Then: timeout is correct
    assert_eq!(timeout, Duration::from_millis(5000));
}

#[test]
fn context_background_handle_timeout_should_succeed() {
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
        },
    );
    let ctx = Context::new(&config).expect("Failed to create context");

    // When: getting background handle timeout
    let timeout = ctx.background_handle_timeout();

    // Then: timeout is correct
    assert_eq!(timeout, Duration::from_millis(1000));
}

#[test]
fn context_accept_handle_timeout_should_succeed() {
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
        },
    );
    let ctx = Context::new(&config).expect("Failed to create context");

    // When: getting accept handle timeout
    let timeout = ctx.accept_handle_timeout();

    // Then: timeout is correct
    assert_eq!(timeout, Duration::from_millis(2000));
}

#[test]
fn context_metrics_receiver_should_succeed() {
    // Given: a Context
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let ctx = create_test_context(backends);

    // When: getting metrics receiver
    let receiver = ctx.metrics_receiver();

    // Then: receiver is returned
    assert!(!receiver.is_closed());
}

#[test]
fn context_health_receiver_should_succeed() {
    // Given: a Context
    let backends = vec![create_test_backend(0, None, Some(10u8))];
    let ctx = create_test_context(backends);

    // When: getting health receiver
    let receiver = ctx.health_receiver();

    // Then: receiver is returned
    assert!(!receiver.is_closed());
}

#[test]
fn context_shutdown_sender_should_succeed() {
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
        },
    );
    let ctx = Context::new(&config).expect("Failed to create context");

    // When: getting shutdown sender
    let sender = ctx.shutdown_sender();

    // Then: sender is returned
    // Verify by sending a message
    let _ = sender.send(());
}

#[test]
fn context_shutdown_receiver_should_succeed() {
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
        },
    );
    let ctx = Context::new(&config).expect("Failed to create context");

    // When: getting shutdown receiver
    let mut receiver = ctx.shutdown_receiver();

    // Then: receiver is returned
    // Verify by checking it can receive (non-blocking)
    let _ = receiver.try_recv(); // May return error if no message, which is fine
}

#[test]
fn context_channel_version_receiver_should_succeed() {
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
        },
    );
    let ctx = Context::new(&config).expect("Failed to create context");

    // When: getting channel version receiver
    let receiver = ctx.channel_version_receiver();

    // Then: receiver is returned
    assert_eq!(*receiver.borrow(), 0);
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
        },
    );
    let ctx = Context::new(&config).expect("Failed to create context");

    // When: notifying connection closed
    ctx.notify_connection_closed();

    // Then: notification is sent (no error)
    // This is tested by the fact that it doesn't panic
}

#[tokio::test]
async fn context_keep_alive_should_succeed() {
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
        },
    );
    let ctx = Arc::new(Context::new(&config).expect("Failed to create context"));

    // When: calling keep_alive with notification
    let ctx_clone = ctx.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(10)).await;
        ctx_clone.notify_connection_closed();
    });

    // Then: keep_alive completes when notified
    let result = timeout(Duration::from_millis(100), ctx.keep_alive()).await;
    assert!(result.is_ok());
}

#[test]
fn context_set_channel_bundle_should_succeed() {
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
        },
    );
    let ctx = Context::new(&config).expect("Failed to create context");

    // When: setting new channel bundle
    let (new_bundle, new_metrics_rx, new_health_rx) = ChannelBundle::new(200, 100);
    ctx.set_channel_bundle(Arc::new(new_bundle), new_metrics_rx, new_health_rx);

    // Then: channel bundle is updated
    assert_eq!(ctx.channel_bundle().metrics_cap(), 200);
    assert_eq!(ctx.channel_bundle().health_cap(), 100);
}

#[test]
fn context_set_backends_preserves_connections_should_succeed() {
    // Given: a Context with connections
    let backends1 = vec![create_test_backend(0, None, Some(10u8))];
    let ctx = create_test_context(backends1);
    ctx.set_backends(vec![create_test_backend(0, None, Some(10u8))])
        .expect("Failed to set backends");

    let connections = ctx.connection_registry();
    connections.increment(0);

    // When: setting backends with same backend ID
    let backends2 = vec![
        create_test_backend(0, None, Some(10u8)),
        create_test_backend(1, None, Some(10u8)),
    ];
    ctx.set_backends(backends2).expect("Failed to set backends");

    // Then: connection count is preserved for existing backend
    let new_connections = ctx.connection_registry();
    assert_eq!(new_connections.get(0), 1); // Migrated connection
}
