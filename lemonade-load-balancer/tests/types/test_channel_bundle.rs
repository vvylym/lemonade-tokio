//! Tests for ChannelBundle
//!
//! This module tests:
//! - ChannelBundle creation with various capacities
//! - Getter methods (config_tx, health_tx, metrics_tx, etc.)
//! - Sender functionality and cloning
//! - Receiver access

use lemonade_load_balancer::prelude::*;

/// Test ChannelBundle creation
///
/// Given: channel capacities
/// When: creating a new ChannelBundle
/// Then: bundle is created successfully
#[rstest::rstest]
#[case(100, 50, 100, 50)]
#[case(1, 1, 1, 1)]
#[case(1000, 500, 1000, 500)]
#[test]
fn test_new(
    #[case] metrics_cap: usize,
    #[case] health_cap: usize,
    #[case] connection_cap: usize,
    #[case] backend_failure_cap: usize,
) {
    let bundle =
        ChannelBundle::new(metrics_cap, health_cap, connection_cap, backend_failure_cap);
    // Note: capacities are not directly accessible, but we can test senders work
    // Verify config sender works (receiver_count is always >= 0)
    let _ = bundle.config_tx();
}

/// Test metrics sender functionality
///
/// Given: a ChannelBundle
/// When: getting metrics sender and sending an event
/// Then: sender works and event is received
#[test]
fn test_metrics_sender() {
    let bundle = ChannelBundle::new(100, 50, 100, 50);
    let mut metrics_rx = bundle
        .metrics_rx()
        .expect("Metrics receiver should be available");
    let sender = bundle.metrics_tx();
    let event = MetricsEvent::ConnectionOpened {
        backend_id: 1,
        at_micros: 1000,
    };
    let send_result = sender.try_send(event.clone());
    assert!(send_result.is_ok());
    let received = metrics_rx.try_recv();
    assert!(received.is_ok());
    match received.expect("Failed to receive event") {
        MetricsEvent::ConnectionOpened {
            backend_id,
            at_micros,
        } => {
            assert_eq!(backend_id, 1);
            assert_eq!(at_micros, 1000);
        }
        _ => panic!("Unexpected event type"),
    }
}

/// Test health sender functionality
///
/// Given: a ChannelBundle
/// When: getting health sender and sending an event
/// Then: sender works and event is received
#[test]
fn test_health_sender() {
    let bundle = ChannelBundle::new(100, 50, 100, 50);
    let mut health_rx = bundle
        .health_rx()
        .expect("Health receiver should be available");
    let sender = bundle.health_tx();
    let event = HealthEvent::BackendHealthy {
        backend_id: 2,
        rtt_micros: 500,
    };
    let send_result = sender.try_send(event.clone());
    assert!(send_result.is_ok());
    let received = health_rx.try_recv();
    assert!(received.is_ok());
    match received.expect("Failed to receive event") {
        HealthEvent::BackendHealthy {
            backend_id,
            rtt_micros,
        } => {
            assert_eq!(backend_id, 2);
            assert_eq!(rtt_micros, 500);
        }
        _ => panic!("Unexpected event type"),
    }
}

/// Test backend failure sender functionality
///
/// Given: a ChannelBundle
/// When: getting backend failure sender and sending an event
/// Then: sender works and event is received
#[test]
fn test_backend_failure_sender() {
    let bundle = ChannelBundle::new(100, 50, 100, 50);
    let mut failure_rx = bundle
        .backend_failure_rx()
        .expect("Backend failure receiver should be available");
    let sender = bundle.backend_failure_tx();
    let event = BackendFailureEvent::ConnectionRefused { backend_id: 1 };
    let send_result = sender.try_send(event.clone());
    assert!(send_result.is_ok());
    let received = failure_rx.try_recv();
    assert!(received.is_ok());
    match received.expect("Failed to receive event") {
        BackendFailureEvent::ConnectionRefused { backend_id } => {
            assert_eq!(backend_id, 1);
        }
        _ => panic!("Unexpected event type"),
    }
}

/// Test connection sender functionality
///
/// Given: a ChannelBundle
/// When: getting connection sender and sending an event
/// Then: sender works and event is received
#[test]
fn test_connection_sender() {
    let bundle = ChannelBundle::new(100, 50, 100, 50);
    let mut connection_rx = bundle
        .connection_rx()
        .expect("Connection receiver should be available");
    let sender = bundle.connection_tx();
    let event = ConnectionEvent::Opened { backend_id: 1 };
    let send_result = sender.try_send(event.clone());
    assert!(send_result.is_ok());
    let received = connection_rx.try_recv();
    assert!(received.is_ok());
    match received.expect("Failed to receive event") {
        ConnectionEvent::Opened { backend_id } => {
            assert_eq!(backend_id, 1);
        }
        _ => panic!("Unexpected event type"),
    }
}

/// Test config sender (broadcast)
///
/// Given: a ChannelBundle
/// When: getting config sender and sending an event
/// Then: sender works and event is received
#[test]
fn test_config_sender() {
    let bundle = ChannelBundle::new(100, 50, 100, 50);
    let mut config_rx = bundle.config_rx();
    let sender = bundle.config_tx();
    let event = ConfigEvent::Migrated;
    let send_result = sender.send(event);
    assert!(send_result.is_ok());
    let received = config_rx.try_recv();
    assert!(received.is_ok());
    match received.expect("Failed to receive event") {
        ConfigEvent::Migrated => {}
        _ => panic!("Unexpected event type"),
    }
}

/// Test shutdown sender (broadcast)
///
/// Given: a ChannelBundle
/// When: getting shutdown sender and sending signal
/// Then: signal is received
#[test]
fn test_shutdown_sender() {
    let bundle = ChannelBundle::new(100, 50, 100, 50);
    let mut shutdown_rx = bundle.shutdown_rx();
    let sender = bundle.shutdown_tx();
    let send_result = sender.send(());
    assert!(send_result.is_ok());
    let received = shutdown_rx.try_recv();
    assert!(received.is_ok());
}

/// Test metrics sender cloning
///
/// Given: a ChannelBundle
/// When: cloning metrics sender multiple times
/// Then: both senders work independently
#[test]
fn test_metrics_sender_clone() {
    let bundle = ChannelBundle::new(100, 50, 100, 50);
    let mut metrics_rx = bundle
        .metrics_rx()
        .expect("Metrics receiver should be available");
    let sender1 = bundle.metrics_tx();
    let sender2 = bundle.metrics_tx();
    let event1 = MetricsEvent::FlushSnapshot;
    let event2 = MetricsEvent::RequestCompleted {
        backend_id: 1,
        latency_micros: 100,
        status_code: 200,
    };
    assert!(sender1.try_send(event1).is_ok());
    assert!(sender2.try_send(event2).is_ok());
    assert!(metrics_rx.try_recv().is_ok());
    assert!(metrics_rx.try_recv().is_ok());
}

/// Test health sender cloning
///
/// Given: a ChannelBundle
/// When: cloning health sender multiple times
/// Then: both senders work independently
#[test]
fn test_health_sender_clone() {
    let bundle = ChannelBundle::new(100, 50, 100, 50);
    let mut health_rx = bundle
        .health_rx()
        .expect("Health receiver should be available");
    let sender1 = bundle.health_tx();
    let sender2 = bundle.health_tx();
    let event1 = HealthEvent::CheckBackend { backend_id: 1 };
    let event2 = HealthEvent::BackendUnhealthy {
        backend_id: 2,
        reason: HealthFailureReason::Timeout,
    };
    assert!(sender1.try_send(event1).is_ok());
    assert!(sender2.try_send(event2).is_ok());
    assert!(health_rx.try_recv().is_ok());
    assert!(health_rx.try_recv().is_ok());
}

/// Test Debug trait implementation
///
/// Given: a ChannelBundle
/// When: formatting with Debug
/// Then: debug string is not empty
#[test]
fn test_debug() {
    let bundle = ChannelBundle::new(100, 50, 100, 50);
    let debug_str = format!("{:?}", bundle);
    assert!(!debug_str.is_empty());
}

/// Test metrics sender sends all event types
///
/// Given: a ChannelBundle
/// When: sending all types of metrics events
/// Then: all events can be sent and received
#[test]
fn test_metrics_sender_all_event_types() {
    let bundle = ChannelBundle::new(100, 50, 100, 50);
    let mut metrics_rx = bundle
        .metrics_rx()
        .expect("Metrics receiver should be available");
    let sender = bundle.metrics_tx();
    let events = vec![
        MetricsEvent::ConnectionOpened {
            backend_id: 1,
            at_micros: 1000,
        },
        MetricsEvent::ConnectionClosed {
            backend_id: 1,
            duration_micros: 5000,
            bytes_in: 100,
            bytes_out: 200,
        },
        MetricsEvent::RequestCompleted {
            backend_id: 1,
            latency_micros: 100,
            status_code: 200,
        },
        MetricsEvent::RequestFailed {
            backend_id: 1,
            latency_micros: 50,
            error_class: MetricsErrorClass::Timeout,
        },
        MetricsEvent::FlushSnapshot,
    ];
    for event in events {
        assert!(sender.try_send(event).is_ok());
    }
    for _ in 0..5 {
        assert!(metrics_rx.try_recv().is_ok());
    }
}

/// Test health sender sends all event types
///
/// Given: a ChannelBundle
/// When: sending all types of health events
/// Then: all events can be sent and received
#[test]
fn test_health_sender_all_event_types() {
    let bundle = ChannelBundle::new(100, 50, 100, 50);
    let mut health_rx = bundle
        .health_rx()
        .expect("Health receiver should be available");
    let sender = bundle.health_tx();
    let events = vec![
        HealthEvent::CheckBackend { backend_id: 1 },
        HealthEvent::BackendHealthy {
            backend_id: 1,
            rtt_micros: 100,
        },
        HealthEvent::BackendUnhealthy {
            backend_id: 1,
            reason: HealthFailureReason::Timeout,
        },
        HealthEvent::HealthTransition {
            backend_id: 1,
            from: HealthStatus::Unhealthy,
            to: HealthStatus::Healthy,
        },
        HealthEvent::BackendConfigUpdated { backend_id: 1 },
    ];
    for event in events {
        assert!(sender.try_send(event).is_ok());
    }
    for _ in 0..5 {
        assert!(health_rx.try_recv().is_ok());
    }
}

/// Test receiver can only be taken once
///
/// Given: a ChannelBundle
/// When: taking receiver multiple times
/// Then: only first call succeeds
#[test]
fn test_receiver_single_consumer() {
    let bundle = ChannelBundle::new(100, 50, 100, 50);
    let rx1 = bundle.metrics_rx();
    assert!(rx1.is_some());
    let rx2 = bundle.metrics_rx();
    assert!(rx2.is_none(), "Receiver should only be available once");
}
