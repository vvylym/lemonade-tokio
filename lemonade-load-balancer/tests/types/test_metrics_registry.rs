//! Tests for MetricsRegistry
//!
//! This module tests:
//! - MetricsSnapshot operations (update, get, remove)
//! - BackendMetrics operations
//! - Calculations (avg_latency, error_rate)
//! - Trait implementations (Debug, Clone, Default)

use lemonade_load_balancer::prelude::*;

/// Test MetricsSnapshot default
///
/// Given: default MetricsSnapshot
/// When: checking properties
/// Then: snapshot is empty
#[test]
fn test_metrics_snapshot_default() {
    let snapshot = MetricsSnapshot::default();
    assert!(snapshot.backend_ids().is_empty());
}

/// Test update metrics
///
/// Given: a MetricsSnapshot
/// When: updating metrics for a backend
/// Then: metrics are stored
#[test]
fn test_update() {
    let snapshot = MetricsSnapshot::default();
    let metrics = BackendMetrics {
        avg_latency_ms: 10.5,
        p95_latency_ms: 20.0,
        error_rate: 0.1,
        last_updated_ms: 1000,
    };
    snapshot.update(1, metrics.clone());
    assert!(snapshot.has_metrics(1));
    let retrieved = snapshot.get(1);
    assert!(retrieved.is_some());
    let retrieved_metrics = retrieved.expect("Metrics not found for backend 1");
    assert_eq!(retrieved_metrics.avg_latency_ms, 10.5);
    assert_eq!(retrieved_metrics.error_rate, 0.1);
}

/// Test update overwrite
///
/// Given: a MetricsSnapshot with existing metrics
/// When: updating with new metrics
/// Then: metrics are overwritten
#[test]
fn test_update_overwrite() {
    let snapshot = MetricsSnapshot::default();
    let metrics1 = BackendMetrics {
        avg_latency_ms: 10.0,
        p95_latency_ms: 20.0,
        error_rate: 0.1,
        last_updated_ms: 1000,
    };
    snapshot.update(1, metrics1);
    let metrics2 = BackendMetrics {
        avg_latency_ms: 15.0,
        p95_latency_ms: 25.0,
        error_rate: 0.2,
        last_updated_ms: 2000,
    };
    snapshot.update(1, metrics2);
    let retrieved = snapshot.get(1).expect("Metrics not found");
    assert_eq!(retrieved.avg_latency_ms, 15.0);
    assert_eq!(retrieved.error_rate, 0.2);
}

/// Test get existing metrics
///
/// Given: a MetricsSnapshot with metrics
/// When: getting metrics for existing backend
/// Then: metrics are returned
#[test]
fn test_get_existing() {
    let snapshot = MetricsSnapshot::default();
    let metrics = BackendMetrics {
        avg_latency_ms: 10.5,
        p95_latency_ms: 20.0,
        error_rate: 0.1,
        last_updated_ms: 1000,
    };
    snapshot.update(1, metrics.clone());
    let retrieved = snapshot.get(1);
    assert!(retrieved.is_some());
    let retrieved_metrics = retrieved.expect("Metrics not found for backend 1");
    assert_eq!(retrieved_metrics.avg_latency_ms, 10.5);
    assert_eq!(retrieved_metrics.p95_latency_ms, 20.0);
    assert_eq!(retrieved_metrics.error_rate, 0.1);
    assert_eq!(retrieved_metrics.last_updated_ms, 1000);
}

/// Test get non-existing metrics
///
/// Given: a MetricsSnapshot without metrics
/// When: getting metrics for non-existing backend
/// Then: None is returned
#[test]
fn test_get_non_existing() {
    let snapshot = MetricsSnapshot::default();
    assert!(snapshot.get(99).is_none());
}

/// Test backend_ids
///
/// Given: a MetricsSnapshot with multiple backends
/// When: getting all backend IDs
/// Then: all IDs are returned
#[test]
fn test_backend_ids() {
    let snapshot = MetricsSnapshot::default();
    snapshot.update(1, BackendMetrics::default());
    snapshot.update(2, BackendMetrics::default());
    snapshot.update(3, BackendMetrics::default());
    let ids = snapshot.backend_ids();
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&1));
    assert!(ids.contains(&2));
    assert!(ids.contains(&3));
}

/// Test backend_ids with empty snapshot
///
/// Given: an empty MetricsSnapshot
/// When: getting all backend IDs
/// Then: empty vector is returned
#[test]
fn test_backend_ids_empty() {
    let snapshot = MetricsSnapshot::default();
    assert!(snapshot.backend_ids().is_empty());
}

/// Test has_metrics with existing
///
/// Given: a MetricsSnapshot with metrics
/// When: checking if metrics exist
/// Then: returns true
#[test]
fn test_has_metrics_existing() {
    let snapshot = MetricsSnapshot::default();
    snapshot.update(1, BackendMetrics::default());
    assert!(snapshot.has_metrics(1));
}

/// Test has_metrics with non-existing
///
/// Given: a MetricsSnapshot without metrics
/// When: checking if metrics exist for non-existing backend
/// Then: returns false
#[test]
fn test_has_metrics_non_existing() {
    let snapshot = MetricsSnapshot::default();
    assert!(!snapshot.has_metrics(99));
}

/// Test avg_latency with existing metrics
///
/// Given: a MetricsSnapshot with metrics
/// When: getting average latency
/// Then: latency is returned
#[test]
fn test_avg_latency_existing() {
    let snapshot = MetricsSnapshot::default();
    let metrics = BackendMetrics {
        avg_latency_ms: 25.5,
        p95_latency_ms: 50.0,
        error_rate: 0.05,
        last_updated_ms: 1000,
    };
    snapshot.update(1, metrics);
    assert_eq!(snapshot.avg_latency(1), Some(25.5));
}

/// Test avg_latency with non-existing metrics
///
/// Given: a MetricsSnapshot without metrics
/// When: getting average latency for non-existing backend
/// Then: None is returned
#[test]
fn test_avg_latency_non_existing() {
    let snapshot = MetricsSnapshot::default();
    assert_eq!(snapshot.avg_latency(99), None);
}

/// Test error_rate with existing metrics
///
/// Given: a MetricsSnapshot with metrics
/// When: getting error rate
/// Then: error rate is returned
#[test]
fn test_error_rate_existing() {
    let snapshot = MetricsSnapshot::default();
    let metrics = BackendMetrics {
        avg_latency_ms: 10.0,
        p95_latency_ms: 20.0,
        error_rate: 0.15,
        last_updated_ms: 1000,
    };
    snapshot.update(1, metrics);
    assert_eq!(snapshot.error_rate(1), Some(0.15));
}

/// Test error_rate with non-existing metrics
///
/// Given: a MetricsSnapshot without metrics
/// When: getting error rate for non-existing backend
/// Then: None is returned
#[test]
fn test_error_rate_non_existing() {
    let snapshot = MetricsSnapshot::default();
    assert_eq!(snapshot.error_rate(99), None);
}

/// Test remove metrics
///
/// Given: a MetricsSnapshot with metrics
/// When: removing metrics for a backend
/// Then: metrics are removed
#[test]
fn test_remove() {
    let snapshot = MetricsSnapshot::default();
    snapshot.update(1, BackendMetrics::default());
    snapshot.update(2, BackendMetrics::default());
    snapshot.remove(1);
    assert!(!snapshot.has_metrics(1));
    assert!(snapshot.has_metrics(2));
    assert!(snapshot.get(1).is_none());
}

/// Test remove non-existing metrics
///
/// Given: a MetricsSnapshot without metrics
/// When: removing metrics for non-existing backend
/// Then: no error occurs
#[test]
fn test_remove_non_existing() {
    let snapshot = MetricsSnapshot::default();
    snapshot.remove(99);
    assert!(!snapshot.has_metrics(99));
}

/// Test multiple backends
///
/// Given: a MetricsSnapshot with multiple backends
/// When: getting metrics for different backends
/// Then: each backend has correct metrics
#[test]
fn test_multiple_backends() {
    let snapshot = MetricsSnapshot::default();
    let metrics1 = BackendMetrics {
        avg_latency_ms: 10.0,
        p95_latency_ms: 20.0,
        error_rate: 0.1,
        last_updated_ms: 1000,
    };
    let metrics2 = BackendMetrics {
        avg_latency_ms: 20.0,
        p95_latency_ms: 40.0,
        error_rate: 0.2,
        last_updated_ms: 2000,
    };
    snapshot.update(1, metrics1);
    snapshot.update(2, metrics2);
    assert_eq!(snapshot.avg_latency(1), Some(10.0));
    assert_eq!(snapshot.error_rate(1), Some(0.1));
    assert_eq!(snapshot.avg_latency(2), Some(20.0));
    assert_eq!(snapshot.error_rate(2), Some(0.2));
}

/// Test BackendMetrics default
///
/// Given: default BackendMetrics
/// When: checking properties
/// Then: all fields are zero
#[test]
fn test_backend_metrics_default() {
    let metrics = BackendMetrics::default();
    assert_eq!(metrics.avg_latency_ms, 0.0);
    assert_eq!(metrics.p95_latency_ms, 0.0);
    assert_eq!(metrics.error_rate, 0.0);
    assert_eq!(metrics.last_updated_ms, 0);
}

/// Test BackendMetrics clone
///
/// Given: a BackendMetrics instance
/// When: cloning the metrics
/// Then: cloned metrics have same values
#[test]
fn test_backend_metrics_clone() {
    let metrics = BackendMetrics {
        avg_latency_ms: 10.0,
        p95_latency_ms: 20.0,
        error_rate: 0.1,
        last_updated_ms: 1000,
    };
    let cloned = metrics.clone();
    assert_eq!(cloned.avg_latency_ms, metrics.avg_latency_ms);
    assert_eq!(cloned.p95_latency_ms, metrics.p95_latency_ms);
    assert_eq!(cloned.error_rate, metrics.error_rate);
    assert_eq!(cloned.last_updated_ms, metrics.last_updated_ms);
}

/// Test BackendMetrics Debug
///
/// Given: a BackendMetrics instance
/// When: formatting with Debug
/// Then: debug string is not empty
#[test]
fn test_backend_metrics_debug() {
    let metrics = BackendMetrics {
        avg_latency_ms: 10.0,
        p95_latency_ms: 20.0,
        error_rate: 0.1,
        last_updated_ms: 1000,
    };
    let debug_str = format!("{:?}", metrics);
    assert!(!debug_str.is_empty());
}

/// Test MetricsSnapshot Debug
///
/// Given: a MetricsSnapshot
/// When: formatting with Debug
/// Then: debug string is not empty
#[test]
fn test_metrics_snapshot_debug() {
    let snapshot = MetricsSnapshot::default();
    let debug_str = format!("{:?}", snapshot);
    assert!(!debug_str.is_empty());
}
