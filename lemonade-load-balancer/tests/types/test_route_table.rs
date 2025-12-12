//! Route table tests
//!
//! Tests for the RouteTable type covering:
//! - Construction (new, default)
//! - Accessors (len, is_empty, get_by_id, get_by_index, backend_ids)
//! - Iteration (iter)
//! - Search operations (find_index, contains)
//! - Filtering (filter_healthy)
//! - Trait implementations (Debug, Clone, Default)

use super::super::common::fixtures::*;
use lemonade_load_balancer::prelude::*;

#[test]
fn route_table_new_should_succeed() {
    // Given: a vector of backends
    let backends = vec![
        create_test_backend_with_details(1, "backend-1", 8080),
        create_test_backend_with_details(2, "backend-2", 8081),
    ];

    // When: creating a new RouteTable
    let table = RouteTable::new(backends.clone());

    // Then: RouteTable is created successfully
    assert_eq!(table.len(), 2);
}

#[test]
fn route_table_new_with_empty_vec_should_succeed() {
    // Given: an empty vector of backends
    let backends = Vec::<BackendMeta>::new();

    // When: creating a new RouteTable
    let table = RouteTable::new(backends);

    // Then: RouteTable is created and empty
    assert_eq!(table.len(), 0);
    assert!(table.is_empty());
}

#[test]
fn route_table_len_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![
        create_test_backend_with_details(1, "backend-1", 8080),
        create_test_backend_with_details(2, "backend-2", 8081),
        create_test_backend_with_details(3, "backend-3", 8082),
    ];
    let table = RouteTable::new(backends);

    // When: getting the length
    let len = table.len();

    // Then: length is correct
    assert_eq!(len, 3);
}

#[test]
fn route_table_is_empty_with_backends_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![create_test_backend_with_details(1, "backend-1", 8080)];
    let table = RouteTable::new(backends);

    // When: checking if empty
    let is_empty = table.is_empty();

    // Then: table is not empty
    assert!(!is_empty);
}

#[test]
fn route_table_is_empty_without_backends_should_succeed() {
    // Given: an empty RouteTable
    let table = RouteTable::new(Vec::new());

    // When: checking if empty
    let is_empty = table.is_empty();

    // Then: table is empty
    assert!(is_empty);
}

#[test]
fn route_table_get_by_id_existing_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![
        create_test_backend_with_details(1, "backend-1", 8080),
        create_test_backend_with_details(2, "backend-2", 8081),
    ];
    let table = RouteTable::new(backends);

    // When: getting backend by existing id
    let backend = table.get_by_id(2);

    // Then: backend is found
    assert!(backend.is_some());
    assert_eq!(backend.expect("Backend not found").id(), &2u8);
}

#[test]
fn route_table_get_by_id_non_existing_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![create_test_backend_with_details(1, "backend-1", 8080)];
    let table = RouteTable::new(backends);

    // When: getting backend by non-existing id
    let backend = table.get_by_id(99);

    // Then: backend is not found
    assert!(backend.is_none());
}

#[test]
fn route_table_get_by_index_valid_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![
        create_test_backend_with_details(1, "backend-1", 8080),
        create_test_backend_with_details(2, "backend-2", 8081),
    ];
    let table = RouteTable::new(backends);

    // When: getting backend by valid index
    let backend = table.get_by_index(0);

    // Then: backend is found
    assert!(backend.is_some());
    assert_eq!(backend.expect("Backend not found").id(), &1u8);
}

#[test]
fn route_table_get_by_index_invalid_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![create_test_backend_with_details(1, "backend-1", 8080)];
    let table = RouteTable::new(backends);

    // When: getting backend by invalid index
    let backend = table.get_by_index(10);

    // Then: backend is not found
    assert!(backend.is_none());
}

#[test]
fn route_table_backend_ids_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![
        create_test_backend_with_details(1, "backend-1", 8080),
        create_test_backend_with_details(2, "backend-2", 8081),
        create_test_backend_with_details(3, "backend-3", 8082),
    ];
    let table = RouteTable::new(backends);

    // When: getting all backend IDs
    let ids = table.backend_ids();

    // Then: all IDs are returned
    assert_eq!(ids, vec![1u8, 2u8, 3u8]);
}

#[test]
fn route_table_backend_ids_empty_should_succeed() {
    // Given: an empty RouteTable
    let table = RouteTable::new(Vec::new());

    // When: getting all backend IDs
    let ids = table.backend_ids();

    // Then: empty vector is returned
    assert!(ids.is_empty());
}

#[test]
fn route_table_iter_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![
        create_test_backend_with_details(1, "backend-1", 8080),
        create_test_backend_with_details(2, "backend-2", 8081),
    ];
    let table = RouteTable::new(backends);

    // When: iterating over backends
    let mut iter = table.iter();
    let first = iter.next();
    let second = iter.next();
    let third = iter.next();

    // Then: iteration works correctly
    assert!(first.is_some());
    assert_eq!(first.expect("First backend not found").id(), &1u8);
    assert!(second.is_some());
    assert_eq!(second.expect("Second backend not found").id(), &2u8);
    assert!(third.is_none());
}

#[test]
fn route_table_find_index_existing_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![
        create_test_backend_with_details(1, "backend-1", 8080),
        create_test_backend_with_details(2, "backend-2", 8081),
        create_test_backend_with_details(3, "backend-3", 8082),
    ];
    let table = RouteTable::new(backends);

    // When: finding index by existing id
    let index = table.find_index(2);

    // Then: index is found
    assert_eq!(index, Some(1));
}

#[test]
fn route_table_find_index_non_existing_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![create_test_backend_with_details(1, "backend-1", 8080)];
    let table = RouteTable::new(backends);

    // When: finding index by non-existing id
    let index = table.find_index(99);

    // Then: index is not found
    assert_eq!(index, None);
}

#[test]
fn route_table_contains_existing_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![
        create_test_backend_with_details(1, "backend-1", 8080),
        create_test_backend_with_details(2, "backend-2", 8081),
    ];
    let table = RouteTable::new(backends);

    // When: checking if existing id is contained
    let contains = table.contains(2);

    // Then: id is contained
    assert!(contains);
}

#[test]
fn route_table_contains_non_existing_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![create_test_backend_with_details(1, "backend-1", 8080)];
    let table = RouteTable::new(backends);

    // When: checking if non-existing id is contained
    let contains = table.contains(99);

    // Then: id is not contained
    assert!(!contains);
}

#[test]
fn route_table_filter_healthy_should_succeed() {
    // Given: a RouteTable with backends and a HealthRegistry
    let backends = vec![
        create_test_backend_with_details(1, "backend-1", 8080),
        create_test_backend_with_details(2, "backend-2", 8081),
        create_test_backend_with_details(3, "backend-3", 8082),
    ];
    let table = RouteTable::new(backends);
    let health = HealthRegistry::new(3);
    health.set_alive(0, true, 1000);
    health.set_alive(1, false, 1000);
    health.set_alive(2, true, 1000);

    // When: filtering healthy backends
    let healthy = table.filter_healthy(&health);

    // Then: only healthy backends are returned
    assert_eq!(healthy.len(), 2);
    assert_eq!(healthy[0].0, 0);
    assert_eq!(healthy[0].1.id(), &1u8);
    assert_eq!(healthy[1].0, 2);
    assert_eq!(healthy[1].1.id(), &3u8);
}

#[test]
fn route_table_filter_healthy_all_unhealthy_should_succeed() {
    // Given: a RouteTable with backends and a HealthRegistry with all unhealthy
    let backends = vec![
        create_test_backend_with_details(1, "backend-1", 8080),
        create_test_backend_with_details(2, "backend-2", 8081),
    ];
    let table = RouteTable::new(backends);
    let health = HealthRegistry::new(2);
    health.set_alive(0, false, 1000);
    health.set_alive(1, false, 1000);

    // When: filtering healthy backends
    let healthy = table.filter_healthy(&health);

    // Then: no healthy backends are returned
    assert!(healthy.is_empty());
}

#[test]
fn route_table_filter_healthy_all_healthy_should_succeed() {
    // Given: a RouteTable with backends and a HealthRegistry with all healthy
    let backends = vec![
        create_test_backend_with_details(1, "backend-1", 8080),
        create_test_backend_with_details(2, "backend-2", 8081),
    ];
    let table = RouteTable::new(backends);
    let health = HealthRegistry::new(2);
    health.set_alive(0, true, 1000);
    health.set_alive(1, true, 1000);

    // When: filtering healthy backends
    let healthy = table.filter_healthy(&health);

    // Then: all backends are returned
    assert_eq!(healthy.len(), 2);
}

#[test]
fn route_table_default_should_succeed() {
    // Given: default RouteTable
    let table = RouteTable::default();

    // When: checking properties
    // Then: table is empty
    assert!(table.is_empty());
    assert_eq!(table.len(), 0);
}

#[test]
fn route_table_clone_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![create_test_backend_with_details(1, "backend-1", 8080)];
    let table = RouteTable::new(backends);

    // When: cloning the table
    let cloned = table.clone();

    // Then: cloned table has same backends
    assert_eq!(table.len(), cloned.len());
    assert_eq!(
        table.get_by_id(1).expect("Backend not found").id(),
        cloned.get_by_id(1).expect("Backend not found").id()
    );
}

#[test]
fn route_table_debug_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![create_test_backend_with_details(1, "backend-1", 8080)];
    let table = RouteTable::new(backends);

    // When: formatting with Debug
    let debug_str = format!("{:?}", table);

    // Then: debug string is not empty
    assert!(!debug_str.is_empty());
}
