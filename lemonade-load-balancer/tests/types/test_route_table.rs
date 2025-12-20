//! Route table tests
//!
//! Tests for the RouteTable type covering:
//! - Construction (new, default)
//! - Accessors (len, is_empty, get, backend_ids)
//! - Search operations (find_index, contains)
//! - Filtering (healthy_backends, active_backends, draining_backends)
//! - Insertion and removal

use super::super::common::fixtures::*;
use lemonade_load_balancer::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{SystemTime, UNIX_EPOCH};

fn backend_meta_to_config(meta: BackendMeta) -> BackendConfig {
    BackendConfig::from(meta)
}

#[test]
fn route_table_new_should_succeed() {
    // Given: a vector of backend configs
    let backends = vec![
        backend_meta_to_config(create_test_backend_with_details(1, "backend-1", 8080)),
        backend_meta_to_config(create_test_backend_with_details(2, "backend-2", 8081)),
    ];

    // When: creating a new RouteTable
    let table = RouteTable::new(backends.clone());

    // Then: RouteTable is created successfully
    assert_eq!(table.len(), 2);
}

#[test]
fn route_table_new_with_empty_vec_should_succeed() {
    // Given: an empty vector of backend configs
    let backends = Vec::<BackendConfig>::new();

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
        backend_meta_to_config(create_test_backend_with_details(1, "backend-1", 8080)),
        backend_meta_to_config(create_test_backend_with_details(2, "backend-2", 8081)),
        backend_meta_to_config(create_test_backend_with_details(3, "backend-3", 8082)),
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
    let backends = vec![backend_meta_to_config(create_test_backend_with_details(
        1,
        "backend-1",
        8080,
    ))];
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
fn route_table_get_existing_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![
        backend_meta_to_config(create_test_backend_with_details(1, "backend-1", 8080)),
        backend_meta_to_config(create_test_backend_with_details(2, "backend-2", 8081)),
    ];
    let table = RouteTable::new(backends);

    // When: getting backend by existing id
    let backend = table.get(2);

    // Then: backend is found
    assert!(backend.is_some());
    assert_eq!(backend.expect("Backend not found").id(), 2u8);
}

#[test]
fn route_table_get_non_existing_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![backend_meta_to_config(create_test_backend_with_details(
        1,
        "backend-1",
        8080,
    ))];
    let table = RouteTable::new(backends);

    // When: getting backend by non-existing id
    let backend = table.get(99);

    // Then: backend is not found
    assert!(backend.is_none());
}

#[test]
fn route_table_backend_ids_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![
        backend_meta_to_config(create_test_backend_with_details(1, "backend-1", 8080)),
        backend_meta_to_config(create_test_backend_with_details(2, "backend-2", 8081)),
        backend_meta_to_config(create_test_backend_with_details(3, "backend-3", 8082)),
    ];
    let table = RouteTable::new(backends);

    // When: getting all backend IDs
    let ids = table.backend_ids();

    // Then: all IDs are returned (order may vary with DashMap)
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&1u8));
    assert!(ids.contains(&2u8));
    assert!(ids.contains(&3u8));
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
fn route_table_find_index_existing_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![
        backend_meta_to_config(create_test_backend_with_details(1, "backend-1", 8080)),
        backend_meta_to_config(create_test_backend_with_details(2, "backend-2", 8081)),
        backend_meta_to_config(create_test_backend_with_details(3, "backend-3", 8082)),
    ];
    let table = RouteTable::new(backends);

    // When: finding index by existing id
    let index = table.find_index(2);

    // Then: index is found (may vary with DashMap iteration order)
    assert!(index.is_some());
}

#[test]
fn route_table_find_index_non_existing_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![backend_meta_to_config(create_test_backend_with_details(
        1,
        "backend-1",
        8080,
    ))];
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
        backend_meta_to_config(create_test_backend_with_details(1, "backend-1", 8080)),
        backend_meta_to_config(create_test_backend_with_details(2, "backend-2", 8081)),
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
    let backends = vec![backend_meta_to_config(create_test_backend_with_details(
        1,
        "backend-1",
        8080,
    ))];
    let table = RouteTable::new(backends);

    // When: checking if non-existing id is contained
    let contains = table.contains(99);

    // Then: id is not contained
    assert!(!contains);
}

#[test]
fn route_table_healthy_backends_should_succeed() {
    // Given: a RouteTable with backends (all start healthy by default)
    let backends = vec![
        backend_meta_to_config(create_test_backend_with_details(1, "backend-1", 8080)),
        backend_meta_to_config(create_test_backend_with_details(2, "backend-2", 8081)),
        backend_meta_to_config(create_test_backend_with_details(3, "backend-3", 8082)),
    ];
    let table = RouteTable::new(backends);

    // When: getting healthy backends
    let healthy = table.healthy_backends();

    // Then: all backends are returned (they start healthy)
    assert_eq!(healthy.len(), 3);
}

#[test]
fn route_table_healthy_backends_with_unhealthy_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![
        backend_meta_to_config(create_test_backend_with_details(1, "backend-1", 8080)),
        backend_meta_to_config(create_test_backend_with_details(2, "backend-2", 8081)),
    ];
    let table = RouteTable::new(backends);

    // Mark one backend as unhealthy
    let backend2 = table.get(2).expect("Backend 2 not found");
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    backend2.set_health(false, now_ms);

    // When: getting healthy backends
    let healthy = table.healthy_backends();

    // Then: only healthy backend is returned
    assert_eq!(healthy.len(), 1);
    assert_eq!(healthy[0].id(), 1);
}

#[test]
fn route_table_active_backends_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![
        backend_meta_to_config(create_test_backend_with_details(1, "backend-1", 8080)),
        backend_meta_to_config(create_test_backend_with_details(2, "backend-2", 8081)),
    ];
    let table = RouteTable::new(backends);

    // Mark one backend as draining
    let backend2 = table.get(2).expect("Backend 2 not found");
    backend2.mark_draining();

    // When: getting active backends
    let active = table.active_backends();

    // Then: only active backend is returned
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id(), 1);
}

#[test]
fn route_table_draining_backends_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![
        backend_meta_to_config(create_test_backend_with_details(1, "backend-1", 8080)),
        backend_meta_to_config(create_test_backend_with_details(2, "backend-2", 8081)),
    ];
    let table = RouteTable::new(backends);

    // Mark one backend as draining
    let backend2 = table.get(2).expect("Backend 2 not found");
    backend2.mark_draining();

    // When: getting draining backends
    let draining = table.draining_backends();

    // Then: only draining backend is returned
    assert_eq!(draining.len(), 1);
    assert_eq!(draining[0].id(), 2);
}

#[test]
fn route_table_insert_should_succeed() {
    // Given: an empty RouteTable
    let table = RouteTable::new(Vec::new());

    // When: inserting a backend
    let config = BackendConfig {
        id: 5,
        name: Some("new-backend".to_string()),
        address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9090).into(),
        weight: Some(20),
    };
    let backend = Arc::new(Backend::new(config));
    table.insert(backend.clone());

    // Then: backend is inserted
    assert_eq!(table.len(), 1);
    assert!(table.contains(5));
    let retrieved = table.get(5).expect("Backend not found");
    assert_eq!(retrieved.id(), 5);
}

#[test]
fn route_table_remove_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![
        backend_meta_to_config(create_test_backend_with_details(1, "backend-1", 8080)),
        backend_meta_to_config(create_test_backend_with_details(2, "backend-2", 8081)),
    ];
    let table = RouteTable::new(backends);

    // When: removing a backend
    let removed = table.remove(2);

    // Then: backend is removed
    assert!(removed.is_some());
    assert_eq!(removed.expect("Backend not removed").id(), 2);
    assert_eq!(table.len(), 1);
    assert!(!table.contains(2));
}

#[test]
fn route_table_all_backends_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![
        backend_meta_to_config(create_test_backend_with_details(1, "backend-1", 8080)),
        backend_meta_to_config(create_test_backend_with_details(2, "backend-2", 8081)),
    ];
    let table = RouteTable::new(backends);

    // When: getting all backends
    let all = table.all_backends();

    // Then: all backends are returned
    assert_eq!(all.len(), 2);
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
fn route_table_debug_should_succeed() {
    // Given: a RouteTable with backends
    let backends = vec![backend_meta_to_config(create_test_backend_with_details(
        1,
        "backend-1",
        8080,
    ))];
    let table = RouteTable::new(backends);

    // When: formatting with Debug
    let debug_str = format!("{:?}", table);

    // Then: debug string is not empty
    assert!(!debug_str.is_empty());
}

#[test]
fn route_table_concurrent_access_should_succeed() {
    use std::sync::Arc;
    use std::thread;

    // Given: a RouteTable with backends
    let backends = vec![
        backend_meta_to_config(create_test_backend_with_details(0, "backend-0", 8080)),
        backend_meta_to_config(create_test_backend_with_details(1, "backend-1", 8081)),
    ];
    let table = Arc::new(RouteTable::new(backends));

    // When: accessing from multiple threads
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let table_clone = table.clone();
            thread::spawn(move || {
                let backend = table_clone.get(0);
                assert!(backend.is_some());
            })
        })
        .collect();

    // Then: all accesses succeed
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

#[test]
fn route_table_active_backends_filters_draining_should_succeed() {
    // Given: a RouteTable with a backend
    let backends = vec![backend_meta_to_config(create_test_backend_with_details(
        0,
        "backend-0",
        8080,
    ))];
    let table = RouteTable::new(backends);

    // Mark backend as draining
    if let Some(backend) = table.get(0) {
        backend.mark_draining();
    }

    // When: getting active backends
    let active = table.active_backends();

    // Then: draining backend is not included
    assert_eq!(active.len(), 0);
}
