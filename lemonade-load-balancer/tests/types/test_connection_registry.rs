//! Tests for ConnectionRegistry
//!
//! This module tests:
//! - ConnectionRegistry creation
//! - Increment/decrement operations
//! - Capacity checks
//! - Migration functionality
//! - Trait implementations (Debug, Default)

use lemonade_load_balancer::prelude::*;

/// Test ConnectionRegistry creation with various capacities
///
/// Given: a capacity
/// When: creating a new ConnectionRegistry
/// Then: registry is created successfully
#[rstest::rstest]
#[case(5, vec![0, 0, 0, 0, 0])]
#[case(3, vec![0, 0, 0])]
#[case(1, vec![0])]
#[test]
fn test_new(#[case] cap: usize, #[case] expected_snapshot: Vec<usize>) {
    let registry = ConnectionRegistry::new(cap);
    assert_eq!(registry.total(), 0);
    assert_eq!(registry.snapshot_per_backend().len(), cap);
    assert_eq!(registry.snapshot_per_backend(), expected_snapshot);
}

/// Test ConnectionRegistry creation with zero capacity
///
/// Given: zero capacity
/// When: creating a new ConnectionRegistry
/// Then: registry is created successfully
#[test]
fn test_new_with_zero_cap() {
    let registry = ConnectionRegistry::new(0);
    assert_eq!(registry.total(), 0);
    assert_eq!(registry.snapshot_per_backend().len(), 0);
}

/// Test increment operations
///
/// Given: a ConnectionRegistry
/// When: incrementing connection counts
/// Then: counts are updated correctly
#[test]
fn test_increment() {
    let registry = ConnectionRegistry::new(3);
    registry.increment(0);
    registry.increment(1);
    registry.increment(1);
    assert_eq!(registry.total(), 3);
    assert_eq!(registry.get(0), 1);
    assert_eq!(registry.get(1), 2);
    assert_eq!(registry.get(2), 0);
}

/// Test increment with out-of-bounds index
///
/// Given: a ConnectionRegistry with capacity 2
/// When: incrementing with out-of-bounds index
/// Then: total is incremented but per-backend is not
#[test]
fn test_increment_out_of_bounds() {
    let registry = ConnectionRegistry::new(2);
    registry.increment(5);
    assert_eq!(registry.total(), 1);
    assert_eq!(registry.get(0), 0);
    assert_eq!(registry.get(1), 0);
}

/// Test decrement operations
///
/// Given: a ConnectionRegistry with some connections
/// When: decrementing connection counts
/// Then: counts are updated correctly
#[test]
fn test_decrement() {
    let registry = ConnectionRegistry::new(3);
    registry.increment(0);
    registry.increment(0);
    registry.increment(1);
    registry.decrement(0);
    registry.decrement(1);
    assert_eq!(registry.total(), 1);
    assert_eq!(registry.get(0), 1);
    assert_eq!(registry.get(1), 0);
}

/// Test decrement with out-of-bounds index
///
/// Given: a ConnectionRegistry with some connections
/// When: decrementing with out-of-bounds index
/// Then: total is decremented but per-backend is not
#[test]
fn test_decrement_out_of_bounds() {
    let registry = ConnectionRegistry::new(2);
    registry.increment(0);
    registry.increment(1);
    registry.decrement(5);
    assert_eq!(registry.total(), 1);
    assert_eq!(registry.get(0), 1);
    assert_eq!(registry.get(1), 1);
}

/// Test total getter
///
/// Given: a ConnectionRegistry with connections
/// When: getting total connections
/// Then: total is correct
#[test]
fn test_total() {
    let registry = ConnectionRegistry::new(3);
    registry.increment(0);
    registry.increment(1);
    registry.increment(2);
    assert_eq!(registry.total(), 3);
}

/// Test get method
///
/// Given: a ConnectionRegistry with connections
/// When: getting connection count for a backend
/// Then: count is correct
#[test]
fn test_get() {
    let registry = ConnectionRegistry::new(3);
    registry.increment(0);
    registry.increment(0);
    registry.increment(1);
    assert_eq!(registry.get(0), 2);
    assert_eq!(registry.get(1), 1);
    assert_eq!(registry.get(2), 0);
}

/// Test get with out-of-bounds index
///
/// Given: a ConnectionRegistry
/// When: getting connection count for out-of-bounds index
/// Then: returns 0
#[test]
fn test_get_out_of_bounds() {
    let registry = ConnectionRegistry::new(2);
    assert_eq!(registry.get(10), 0);
}

/// Test snapshot_per_backend
///
/// Given: a ConnectionRegistry with connections
/// When: getting snapshot
/// Then: snapshot is correct
#[test]
fn test_snapshot_per_backend() {
    let registry = ConnectionRegistry::new(4);
    registry.increment(0);
    registry.increment(0);
    registry.increment(1);
    registry.increment(3);
    let snapshot = registry.snapshot_per_backend();
    assert_eq!(snapshot, vec![2, 1, 0, 1]);
}

/// Test has_capacity with limit
///
/// Given: a ConnectionRegistry with connections and a limit
/// When: checking capacity
/// Then: capacity check is correct
#[test]
fn test_has_capacity_with_limit() {
    let registry = ConnectionRegistry::new(3);
    registry.increment(0);
    registry.increment(0);
    let max_connections = Some(3);
    assert!(registry.has_capacity(0, max_connections)); // 2 < 3
    assert!(registry.has_capacity(1, max_connections)); // 0 < 3
}

/// Test has_capacity at limit
///
/// Given: a ConnectionRegistry at capacity limit
/// When: checking capacity
/// Then: no capacity available
#[test]
fn test_has_capacity_at_limit() {
    let registry = ConnectionRegistry::new(2);
    registry.increment(0);
    registry.increment(0);
    registry.increment(0);
    let max_connections = Some(3);
    assert!(!registry.has_capacity(0, max_connections)); // 3 >= 3
}

/// Test has_capacity without limit
///
/// Given: a ConnectionRegistry with no limit
/// When: checking capacity with no limit
/// Then: always has capacity
#[test]
fn test_has_capacity_no_limit() {
    let registry = ConnectionRegistry::new(2);
    registry.increment(0);
    registry.increment(0);
    registry.increment(0);
    assert!(registry.has_capacity(0, None));
}

/// Test migrate functionality
///
/// Given: a ConnectionRegistry with connections and a mapping
/// When: migrating to new registry
/// Then: connections are migrated correctly
#[test]
fn test_migrate() {
    let old_registry = ConnectionRegistry::new(3);
    old_registry.increment(0);
    old_registry.increment(0);
    old_registry.increment(1);
    old_registry.increment(2);
    // Mapping: old[0] -> new[1], old[1] -> new[0], old[2] -> None (removed)
    let id_mapping = vec![Some(1), Some(0), None];
    let new_registry = old_registry.migrate(2, &id_mapping);
    assert_eq!(new_registry.total(), 4);
    assert_eq!(new_registry.get(0), 1); // old[1] -> new[0]
    assert_eq!(new_registry.get(1), 2); // old[0] -> new[1]
}

/// Test migrate with empty registry
///
/// Given: an empty ConnectionRegistry
/// When: migrating to new registry
/// Then: new registry is empty
#[test]
fn test_migrate_empty() {
    let old_registry = ConnectionRegistry::new(2);
    let id_mapping = vec![Some(0), Some(1)];
    let new_registry = old_registry.migrate(2, &id_mapping);
    assert_eq!(new_registry.total(), 0);
    assert_eq!(new_registry.get(0), 0);
    assert_eq!(new_registry.get(1), 0);
}

/// Test migrate to larger capacity
///
/// Given: a ConnectionRegistry migrating to larger capacity
/// When: migrating to larger registry
/// Then: connections are migrated correctly
#[test]
fn test_migrate_larger_capacity() {
    let old_registry = ConnectionRegistry::new(2);
    old_registry.increment(0);
    old_registry.increment(1);
    let id_mapping = vec![Some(0), Some(1)];
    let new_registry = old_registry.migrate(4, &id_mapping);
    assert_eq!(new_registry.total(), 2);
    assert_eq!(new_registry.get(0), 1);
    assert_eq!(new_registry.get(1), 1);
    assert_eq!(new_registry.get(2), 0);
    assert_eq!(new_registry.get(3), 0);
}

/// Test migrate to smaller capacity
///
/// Given: a ConnectionRegistry migrating to smaller capacity
/// When: migrating to smaller registry
/// Then: only mapped connections are migrated
#[test]
fn test_migrate_smaller_capacity() {
    let old_registry = ConnectionRegistry::new(4);
    old_registry.increment(0);
    old_registry.increment(1);
    old_registry.increment(2);
    old_registry.increment(3);
    // Only map first 2 backends
    let id_mapping = vec![Some(0), Some(1), None, None];
    let new_registry = old_registry.migrate(2, &id_mapping);
    assert_eq!(new_registry.total(), 4); // Total preserved
    assert_eq!(new_registry.get(0), 1);
    assert_eq!(new_registry.get(1), 1);
}

/// Test iter_counts
///
/// Given: a ConnectionRegistry with connections
/// When: iterating over counts
/// Then: iterator returns correct counts
#[test]
fn test_iter_counts() {
    let registry = ConnectionRegistry::new(3);
    registry.increment(0);
    registry.increment(0);
    registry.increment(1);
    let counts: Vec<_> = registry.iter_counts().collect();
    assert_eq!(counts, vec![(0, 2), (1, 1), (2, 0)]);
}

/// Test iter_counts with empty registry
///
/// Given: an empty ConnectionRegistry
/// When: iterating over counts
/// Then: iterator returns zeros
#[test]
fn test_iter_counts_empty() {
    let registry = ConnectionRegistry::new(3);
    let counts: Vec<_> = registry.iter_counts().collect();
    assert_eq!(counts, vec![(0, 0), (1, 0), (2, 0)]);
}

/// Test increment/decrement balance
///
/// Given: a ConnectionRegistry
/// When: incrementing and decrementing same amount
/// Then: final state is correct
#[test]
fn test_increment_decrement_balance() {
    let registry = ConnectionRegistry::new(2);
    registry.increment(0);
    registry.increment(0);
    registry.increment(1);
    registry.decrement(0);
    registry.decrement(1);
    assert_eq!(registry.total(), 1);
    assert_eq!(registry.get(0), 1);
    assert_eq!(registry.get(1), 0);
}

/// Test Default trait implementation
///
/// Given: default ConnectionRegistry
/// When: checking properties
/// Then: registry is empty
#[test]
fn test_default() {
    let registry = ConnectionRegistry::default();
    assert_eq!(registry.total(), 0);
    assert_eq!(registry.snapshot_per_backend().len(), 0);
}

/// Test Debug trait implementation
///
/// Given: a ConnectionRegistry
/// When: formatting with Debug
/// Then: debug string is not empty
#[test]
fn test_debug() {
    let registry = ConnectionRegistry::new(2);
    let debug_str = format!("{:?}", registry);
    assert!(!debug_str.is_empty());
}
