//! Common test utilities module
//! Provides shared fixtures, mocks, strategies, and helper functions for all tests

pub mod fixtures;
pub mod strategies;

use lemonade_load_balancer::prelude::BackendMeta;
use proptest::prelude::Strategy as ProptestStrategy;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

/// Generate a backend ID strategy (u8)
pub fn backend_id_strategy() -> impl ProptestStrategy<Value = u8> {
    0u8..=254u8
}

/// Generate a backend weight strategy (u8, 1-255)
pub fn backend_weight_strategy() -> impl ProptestStrategy<Value = u8> {
    1u8..=255u8
}

/// Generate a backend name strategy
pub fn backend_name_strategy() -> impl ProptestStrategy<Value = String> {
    "[a-zA-Z0-9_-]{1,20}"
}

/// Generate a SocketAddr strategy
pub fn socket_addr_strategy() -> impl ProptestStrategy<Value = SocketAddr> {
    use proptest::prelude::any;
    (any::<u32>(), 1u16..=65535u16)
        .prop_map(|(ip, port)| SocketAddr::new(IpAddr::V4(Ipv4Addr::from(ip)), port))
}

/// Generate a BackendMeta strategy
pub fn backend_meta_strategy() -> impl ProptestStrategy<Value = BackendMeta> {
    use proptest::option;
    (
        backend_id_strategy(),
        backend_name_strategy(),
        socket_addr_strategy(),
        option::of(backend_weight_strategy()),
    )
        .prop_map(|(id, name, addr, weight)| {
            BackendMeta::new(id, Some(name), addr, weight)
        })
}

/// Generate a duration strategy (1-100ms for fast tests)
pub fn duration_strategy() -> impl ProptestStrategy<Value = Duration> {
    (1u64..=100u64).prop_map(Duration::from_millis)
}

// Re-export commonly used items when needed
// Tests can import directly: use common::fixtures::* or common::strategies::*
