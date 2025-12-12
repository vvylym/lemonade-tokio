//! Tests for HealthRegistry
//!
//! This module tests:
//! - HealthRegistry creation
//! - Setting and checking health status
//! - Filtering healthy/unhealthy backends
//! - Migration functionality
//! - Trait implementations (Debug, Default)

use lemonade_load_balancer::prelude::*;

/// Test HealthRegistry creation
///
/// Given: a capacity
/// When: creating a new HealthRegistry
/// Then: registry is created successfully
#[test]
fn test_new() {
    let cap = 5;
    let registry = HealthRegistry::new(cap);
    assert_eq!(registry.healthy_indices().len(), 0);
    assert_eq!(registry.unhealthy_indices().len(), cap);
    assert!(registry.is_valid_index(0));
    assert!(registry.is_valid_index(4));
    assert!(!registry.is_valid_index(5));
}

/// Test HealthRegistry creation with zero capacity
///
/// Given: zero capacity
/// When: creating a new HealthRegistry
/// Then: registry is created successfully
#[test]
fn test_new_with_zero_cap() {
    let registry = HealthRegistry::new(0);
    assert_eq!(registry.healthy_indices().len(), 0);
    assert_eq!(registry.unhealthy_indices().len(), 0);
    assert!(!registry.is_valid_index(0));
}

/// Test set_alive with true
///
/// Given: a HealthRegistry
/// When: setting backend as alive
/// Then: backends are marked as alive
#[test]
fn test_set_alive_true() {
    let registry = HealthRegistry::new(3);
    registry.set_alive(0, true, 1000);
    registry.set_alive(1, true, 2000);
    assert!(registry.is_alive(0));
    assert!(registry.is_alive(1));
    assert!(!registry.is_alive(2));
    assert_eq!(registry.last_check(0), Some(1000));
    assert_eq!(registry.last_check(1), Some(2000));
}

/// Test set_alive with false
///
/// Given: a HealthRegistry with alive backends
/// When: setting backend as not alive
/// Then: backend is marked as not alive
#[test]
fn test_set_alive_false() {
    let registry = HealthRegistry::new(3);
    registry.set_alive(0, true, 1000);
    registry.set_alive(1, true, 2000);
    registry.set_alive(0, false, 3000);
    assert!(!registry.is_alive(0));
    assert!(registry.is_alive(1));
    assert_eq!(registry.last_check(0), Some(3000));
}

/// Test set_alive with out-of-bounds index
///
/// Given: a HealthRegistry with capacity 2
/// When: setting out-of-bounds index as alive
/// Then: no change occurs (silently ignored)
#[test]
fn test_set_alive_out_of_bounds() {
    let registry = HealthRegistry::new(2);
    registry.set_alive(10, true, 1000);
    assert!(!registry.is_alive(10));
    assert_eq!(registry.last_check(10), None);
}

/// Test is_alive
///
/// Given: a HealthRegistry with mixed health statuses
/// When: checking if backends are alive
/// Then: status is correct
#[test]
fn test_is_alive() {
    let registry = HealthRegistry::new(4);
    registry.set_alive(0, true, 1000);
    registry.set_alive(1, false, 1000);
    registry.set_alive(2, true, 1000);
    assert!(registry.is_alive(0));
    assert!(!registry.is_alive(1));
    assert!(registry.is_alive(2));
    assert!(!registry.is_alive(3));
}

/// Test is_alive with out-of-bounds index
///
/// Given: a HealthRegistry
/// When: checking out-of-bounds index
/// Then: returns false
#[test]
fn test_is_alive_out_of_bounds() {
    let registry = HealthRegistry::new(2);
    assert!(!registry.is_alive(10));
}

/// Test last_check
///
/// Given: a HealthRegistry with checks
/// When: getting last check timestamps
/// Then: timestamps are correct
#[test]
fn test_last_check() {
    let registry = HealthRegistry::new(3);
    registry.set_alive(0, true, 1000);
    registry.set_alive(1, true, 2000);
    registry.set_alive(2, false, 3000);
    assert_eq!(registry.last_check(0), Some(1000));
    assert_eq!(registry.last_check(1), Some(2000));
    assert_eq!(registry.last_check(2), Some(3000));
}

/// Test last_check with out-of-bounds index
///
/// Given: a HealthRegistry
/// When: getting last check for out-of-bounds index
/// Then: returns None
#[test]
fn test_last_check_out_of_bounds() {
    let registry = HealthRegistry::new(2);
    assert_eq!(registry.last_check(10), None);
}

/// Test healthy_indices
///
/// Given: a HealthRegistry with mixed health statuses
/// When: getting healthy indices
/// Then: only healthy backends are returned
#[test]
fn test_healthy_indices() {
    let registry = HealthRegistry::new(5);
    registry.set_alive(0, true, 1000);
    registry.set_alive(1, false, 1000);
    registry.set_alive(2, true, 1000);
    registry.set_alive(3, false, 1000);
    registry.set_alive(4, true, 1000);
    let healthy = registry.healthy_indices();
    assert_eq!(healthy, vec![0, 2, 4]);
}

/// Test healthy_indices with all unhealthy
///
/// Given: a HealthRegistry with all unhealthy backends
/// When: getting healthy indices
/// Then: empty vector is returned
#[test]
fn test_healthy_indices_all_unhealthy() {
    let registry = HealthRegistry::new(3);
    let healthy = registry.healthy_indices();
    assert!(healthy.is_empty());
}

/// Test healthy_indices with all healthy
///
/// Given: a HealthRegistry with all healthy backends
/// When: getting healthy indices
/// Then: all indices are returned
#[test]
fn test_healthy_indices_all_healthy() {
    let registry = HealthRegistry::new(3);
    registry.set_alive(0, true, 1000);
    registry.set_alive(1, true, 1000);
    registry.set_alive(2, true, 1000);
    let healthy = registry.healthy_indices();
    assert_eq!(healthy, vec![0, 1, 2]);
}

/// Test unhealthy_indices
///
/// Given: a HealthRegistry with mixed health statuses
/// When: getting unhealthy indices
/// Then: only unhealthy backends are returned
#[test]
fn test_unhealthy_indices() {
    let registry = HealthRegistry::new(5);
    registry.set_alive(0, true, 1000);
    registry.set_alive(1, false, 1000);
    registry.set_alive(2, true, 1000);
    registry.set_alive(3, false, 1000);
    registry.set_alive(4, true, 1000);
    let unhealthy = registry.unhealthy_indices();
    assert_eq!(unhealthy, vec![1, 3]);
}

/// Test unhealthy_indices with all healthy
///
/// Given: a HealthRegistry with all healthy backends
/// When: getting unhealthy indices
/// Then: empty vector is returned
#[test]
fn test_unhealthy_indices_all_healthy() {
    let registry = HealthRegistry::new(3);
    registry.set_alive(0, true, 1000);
    registry.set_alive(1, true, 1000);
    registry.set_alive(2, true, 1000);
    let unhealthy = registry.unhealthy_indices();
    assert!(unhealthy.is_empty());
}

/// Test unhealthy_indices with all unhealthy
///
/// Given: a HealthRegistry with all unhealthy backends
/// When: getting unhealthy indices
/// Then: all indices are returned
#[test]
fn test_unhealthy_indices_all_unhealthy() {
    let registry = HealthRegistry::new(3);
    let unhealthy = registry.unhealthy_indices();
    assert_eq!(unhealthy, vec![0, 1, 2]);
}

/// Test migrate functionality
///
/// Given: a HealthRegistry with health statuses and a mapping
/// When: migrating to new registry
/// Then: health statuses are migrated correctly
#[test]
fn test_migrate() {
    let old_registry = HealthRegistry::new(3);
    old_registry.set_alive(0, true, 1000);
    old_registry.set_alive(1, false, 1000);
    old_registry.set_alive(2, true, 2000);
    // Mapping: old[0] -> new[1], old[1] -> new[0], old[2] -> None (removed)
    let id_mapping = vec![Some(1), Some(0), None];
    let new_registry = old_registry.migrate(2, &id_mapping);
    // old[1] (false) -> new[0] (should be false)
    assert!(!new_registry.is_alive(0));
    // old[0] (true) -> new[1] (should be true)
    assert!(new_registry.is_alive(1));
}

/// Test migrate with empty registry
///
/// Given: an empty HealthRegistry
/// When: migrating to new registry
/// Then: new registry is empty
#[test]
fn test_migrate_empty() {
    let old_registry = HealthRegistry::new(2);
    let id_mapping = vec![Some(0), Some(1)];
    let new_registry = old_registry.migrate(2, &id_mapping);
    assert_eq!(new_registry.healthy_indices().len(), 0);
    assert_eq!(new_registry.unhealthy_indices().len(), 2);
}

/// Test migrate to larger capacity
///
/// Given: a HealthRegistry migrating to larger capacity
/// When: migrating to larger registry
/// Then: health statuses are migrated correctly
#[test]
fn test_migrate_larger_capacity() {
    let old_registry = HealthRegistry::new(2);
    old_registry.set_alive(0, true, 1000);
    old_registry.set_alive(1, false, 1000);
    let id_mapping = vec![Some(0), Some(1)];
    let new_registry = old_registry.migrate(4, &id_mapping);
    assert!(new_registry.is_alive(0));
    assert!(!new_registry.is_alive(1));
    assert!(!new_registry.is_alive(2));
    assert!(!new_registry.is_alive(3));
}

/// Test migrate to smaller capacity
///
/// Given: a HealthRegistry migrating to smaller capacity
/// When: migrating to smaller registry
/// Then: only mapped health statuses are migrated
#[test]
fn test_migrate_smaller_capacity() {
    let old_registry = HealthRegistry::new(4);
    old_registry.set_alive(0, true, 1000);
    old_registry.set_alive(1, false, 1000);
    old_registry.set_alive(2, true, 1000);
    old_registry.set_alive(3, false, 1000);
    // Only map first 2 backends
    let id_mapping = vec![Some(0), Some(1), None, None];
    let new_registry = old_registry.migrate(2, &id_mapping);
    assert!(new_registry.is_alive(0));
    assert!(!new_registry.is_alive(1));
}

/// Test is_valid_index
///
/// Given: a HealthRegistry
/// When: checking if index is valid
/// Then: returns correct validity
#[test]
fn test_is_valid_index() {
    let registry = HealthRegistry::new(3);
    assert!(registry.is_valid_index(0));
    assert!(registry.is_valid_index(2));
    assert!(!registry.is_valid_index(3));
    assert!(!registry.is_valid_index(10));
}

/// Test Default trait implementation
///
/// Given: default HealthRegistry
/// When: checking properties
/// Then: registry is empty
#[test]
fn test_default() {
    let registry = HealthRegistry::default();
    assert_eq!(registry.healthy_indices().len(), 0);
    assert_eq!(registry.unhealthy_indices().len(), 0);
}

/// Test Debug trait implementation
///
/// Given: a HealthRegistry
/// When: formatting with Debug
/// Then: debug string is not empty
#[test]
fn test_debug() {
    let registry = HealthRegistry::new(2);
    let debug_str = format!("{:?}", registry);
    assert!(!debug_str.is_empty());
}
