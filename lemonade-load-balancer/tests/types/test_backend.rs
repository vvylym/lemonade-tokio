//! Tests for Backend
//!
use lemonade_load_balancer::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

fn create_test_backend_config() -> BackendConfig {
    BackendConfig {
        id: 1,
        name: Some("test-backend".to_string()),
        address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080).into(),
        weight: Some(10),
    }
}

#[test]
fn test_backend_new_starts_healthy() {
    let config = create_test_backend_config();
    let backend = Backend::new(config);

    // Backend should start healthy by default
    assert!(backend.is_alive());
    assert_eq!(backend.active_connections(), 0);
    assert!(backend.is_active());
    assert!(!backend.is_draining());
}

#[test]
fn test_backend_metadata_getters() {
    let config = create_test_backend_config();
    let backend = Backend::new(config);

    assert_eq!(backend.id(), 1);
    assert_eq!(backend.name(), Some("test-backend"));
    assert_eq!(backend.address().as_str(), "127.0.0.1:8080");
    assert_eq!(backend.weight(), Some(10));
}

#[test]
fn test_backend_health_set_and_get() {
    let config = create_test_backend_config();
    let backend = Backend::new(config);

    // Initially healthy
    assert!(backend.is_alive());

    // Set unhealthy
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    backend.set_health(false, now_ms);
    assert!(!backend.is_alive());
    assert_eq!(backend.last_health_check(), now_ms);

    // Set healthy again
    let now_ms2 = now_ms + 1000;
    backend.set_health(true, now_ms2);
    assert!(backend.is_alive());
    assert_eq!(backend.last_health_check(), now_ms2);
}

#[test]
fn test_backend_connection_tracking() {
    let config = create_test_backend_config();
    let backend = Backend::new(config);

    // Initially no connections
    assert_eq!(backend.active_connections(), 0);

    // Increment connections
    backend.increment_connection();
    assert_eq!(backend.active_connections(), 1);

    backend.increment_connection();
    assert_eq!(backend.active_connections(), 2);

    // Decrement connections
    backend.decrement_connection();
    assert_eq!(backend.active_connections(), 1);

    backend.decrement_connection();
    assert_eq!(backend.active_connections(), 0);
}

#[test]
fn test_backend_has_capacity_for_health_check() {
    let config = create_test_backend_config();
    let backend = Backend::new(config);

    // Initially has capacity
    assert!(backend.has_capacity_for_health_check(10));

    // Add connections up to threshold
    for _ in 0..9 {
        backend.increment_connection();
    }
    assert!(backend.has_capacity_for_health_check(10));
    assert_eq!(backend.active_connections(), 9);

    // At threshold, no capacity
    backend.increment_connection();
    assert!(!backend.has_capacity_for_health_check(10));
    assert_eq!(backend.active_connections(), 10);
}

#[test]
fn test_backend_metrics_recording() {
    let config = create_test_backend_config();
    let backend = Backend::new(config);

    // Record successful requests
    backend.record_request(100, false); // 100ms latency, no error
    backend.record_request(200, false);
    backend.record_request(150, false);

    let metrics = backend.metrics_snapshot();
    assert_eq!(metrics.avg_latency_ms, 150.0); // (100 + 200 + 150) / 3
    assert_eq!(metrics.error_rate, 0.0);
    assert_eq!(metrics.p95_latency_ms, 225.0); // 150 * 1.5

    // Record failed requests
    backend.record_request(50, true); // error
    backend.record_request(75, true); // error

    let metrics = backend.metrics_snapshot();
    // Total latency: 100 + 200 + 150 + 50 + 75 = 575
    // Total requests: 5
    // Average: 575 / 5 = 115.0
    assert_eq!(metrics.avg_latency_ms, 115.0);
    assert_eq!(metrics.error_rate, 0.4); // 2 errors / 5 requests
    assert_eq!(metrics.p95_latency_ms, 172.5); // 115.0 * 1.5
}

#[test]
fn test_backend_metrics_snapshot_empty() {
    let config = create_test_backend_config();
    let backend = Backend::new(config);

    let metrics = backend.metrics_snapshot();
    assert_eq!(metrics.avg_latency_ms, 0.0);
    assert_eq!(metrics.error_rate, 0.0);
    assert_eq!(metrics.p95_latency_ms, 0.0);
}

#[test]
fn test_backend_draining_state() {
    let config = create_test_backend_config();
    let backend = Backend::new(config);

    // Initially active
    assert!(backend.is_active());
    assert!(!backend.is_draining());

    // Mark as draining
    backend.mark_draining();
    assert!(!backend.is_active());
    assert!(backend.is_draining());
}

#[test]
fn test_backend_can_accept_new_connections() {
    let config = create_test_backend_config();
    let backend = Backend::new(config);

    // Healthy and active - can accept
    assert!(backend.can_accept_new_connections());

    // Unhealthy but active - cannot accept
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    backend.set_health(false, now_ms);
    assert!(!backend.can_accept_new_connections());

    // Healthy but draining - cannot accept
    backend.set_health(true, now_ms);
    backend.mark_draining();
    assert!(!backend.can_accept_new_connections());
}

#[test]
fn test_backend_concurrent_operations() {
    let config = create_test_backend_config();
    let backend = Arc::new(Backend::new(config));

    use std::thread;

    // Spawn multiple threads to test concurrent operations
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let backend = backend.clone();
            thread::spawn(move || {
                for _ in 0..100 {
                    backend.increment_connection();
                    backend.record_request(100, false);
                    backend.decrement_connection();
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // All connections should be decremented
    assert_eq!(backend.active_connections(), 0);
    // Should have 1000 requests recorded (each with 100ms latency)
    // Total latency: 100 * 1000 = 100000ms
    // Average: 100000 / 1000 = 100.0
    let metrics = backend.metrics_snapshot();
    assert_eq!(metrics.avg_latency_ms, 100.0);
    assert_eq!(metrics.error_rate, 0.0);
}

#[test]
fn test_backend_config_from_backend_meta() {
    let meta = BackendMeta::new(
        5u8,
        Some("test-backend"),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 9090),
        Some(20u8),
    );

    let config: BackendConfig = meta.into();
    assert_eq!(config.id, 5);
    assert_eq!(config.name, Some("test-backend".to_string()));
    assert_eq!(config.address.as_str(), "192.168.1.1:9090");
    assert_eq!(config.weight, Some(20));
}

#[test]
fn test_backend_metrics_timestamp() {
    let config = create_test_backend_config();
    let backend = Backend::new(config);

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    backend.update_metrics_timestamp(now_ms);
    let metrics = backend.metrics_snapshot();
    assert_eq!(metrics.last_updated_ms, now_ms);
}

#[test]
fn test_backend_concurrent_connections() {
    use std::sync::Arc;
    use std::thread;

    let backend_config = BackendConfig {
        id: 0,
        name: Some("concurrent-test".to_string()),
        address: BackendAddress::parse("127.0.0.1:8080").unwrap(),
        weight: Some(10),
    };
    let backend = Arc::new(Backend::new(backend_config));

    // Spawn multiple threads incrementing connections
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let backend_clone = backend.clone();
            thread::spawn(move || {
                for _ in 0..10 {
                    backend_clone.increment_connection();
                    backend_clone.decrement_connection();
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // All increments and decrements should cancel out
    assert_eq!(backend.active_connections(), 0);
}

#[test]
fn test_backend_metrics_timestamp_update() {
    let backend_config = BackendConfig {
        id: 0,
        name: Some("timestamp-test".to_string()),
        address: BackendAddress::parse("127.0.0.1:8080").unwrap(),
        weight: Some(10),
    };
    let backend = Backend::new(backend_config);

    backend.update_metrics_timestamp(5000);
    let metrics = backend.metrics_snapshot();
    assert_eq!(metrics.last_updated_ms, 5000);
}
