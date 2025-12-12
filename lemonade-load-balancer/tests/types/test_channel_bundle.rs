//! Tests for ChannelBundle
//!
//! This module tests:
//! - ChannelBundle creation with various capacities
//! - Getter methods (metrics_cap, health_cap, senders)
//! - Sender functionality and cloning
//! - Trait implementations (Debug, Clone)

use lemonade_load_balancer::prelude::*;

/// Test ChannelBundle creation
///
/// Given: channel capacities
/// When: creating a new ChannelBundle
/// Then: bundle is created successfully
#[rstest::rstest]
#[case(100, 50)]
#[case(1, 1)]
#[case(1000, 500)]
#[test]
fn test_new(#[case] metrics_cap: usize, #[case] health_cap: usize) {
    let (bundle, metrics_rx, health_rx) = ChannelBundle::new(metrics_cap, health_cap);
    assert_eq!(bundle.metrics_cap(), metrics_cap);
    assert_eq!(bundle.health_cap(), health_cap);
    assert!(!metrics_rx.is_closed());
    assert!(!health_rx.is_closed());
}

/// Test metrics capacity getter
///
/// Given: a ChannelBundle
/// When: getting metrics capacity
/// Then: capacity is correct
#[test]
fn test_metrics_cap() {
    let (bundle, _, _) = ChannelBundle::new(100, 50);
    assert_eq!(bundle.metrics_cap(), 100);
}

/// Test health capacity getter
///
/// Given: a ChannelBundle
/// When: getting health capacity
/// Then: capacity is correct
#[test]
fn test_health_cap() {
    let (bundle, _, _) = ChannelBundle::new(100, 50);
    assert_eq!(bundle.health_cap(), 50);
}

/// Test metrics sender functionality
///
/// Given: a ChannelBundle
/// When: getting metrics sender and sending an event
/// Then: sender works and event is received
#[test]
fn test_metrics_sender() {
    let (bundle, mut metrics_rx, _) = ChannelBundle::new(100, 50);
    let sender = bundle.metrics_sender();
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
    let (bundle, _, mut health_rx) = ChannelBundle::new(100, 50);
    let sender = bundle.health_sender();
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

/// Test metrics sender cloning
///
/// Given: a ChannelBundle
/// When: cloning metrics sender multiple times
/// Then: both senders work independently
#[test]
fn test_metrics_sender_clone() {
    let (bundle, mut metrics_rx, _) = ChannelBundle::new(100, 50);
    let sender1 = bundle.metrics_sender();
    let sender2 = bundle.metrics_sender();
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
    let (bundle, _, mut health_rx) = ChannelBundle::new(100, 50);
    let sender1 = bundle.health_sender();
    let sender2 = bundle.health_sender();
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
    let (bundle, _, _) = ChannelBundle::new(100, 50);
    let debug_str = format!("{:?}", bundle);
    assert!(!debug_str.is_empty());
}

/// Test Clone trait implementation
///
/// Given: a ChannelBundle
/// When: cloning the bundle
/// Then: cloned bundle has same capacities
#[test]
fn test_clone() {
    let (bundle, _, _) = ChannelBundle::new(100, 50);
    let cloned = bundle.clone();
    assert_eq!(bundle.metrics_cap(), cloned.metrics_cap());
    assert_eq!(bundle.health_cap(), cloned.health_cap());
}

/// Test with minimum capacity
///
/// Given: minimum capacities (mpsc requires > 0)
/// When: creating a new ChannelBundle
/// Then: bundle is created successfully
#[test]
fn test_with_minimum_capacity() {
    let (bundle, _, _) = ChannelBundle::new(1, 1);
    assert_eq!(bundle.metrics_cap(), 1);
    assert_eq!(bundle.health_cap(), 1);
}

/// Test metrics sender sends all event types
///
/// Given: a ChannelBundle
/// When: sending all types of metrics events
/// Then: all events can be sent and received
#[test]
fn test_metrics_sender_all_event_types() {
    let (bundle, mut metrics_rx, _) = ChannelBundle::new(100, 50);
    let sender = bundle.metrics_sender();
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
    let (bundle, _, mut health_rx) = ChannelBundle::new(100, 50);
    let sender = bundle.health_sender();
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
