//! Tests for BackendMeta
//!
use lemonade_load_balancer::prelude::*;
use proptest::prelude::*;
use rstest::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::common::fixtures::create_test_backend;
use crate::common::*;

#[test]
fn test_new_with_valid_inputs() {
    let meta = BackendMeta::new(
        1u8,
        Some("backend-1"),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        Some(10u8),
    );

    assert_eq!(meta.id(), &1u8);
    assert_eq!(meta.name(), Some(&"backend-1".to_string()));
    assert_eq!(meta.weight(), Some(10u8));
}

#[rstest]
#[case(Some("backend-1"), Some(10u8), Some(&"backend-1".to_string()), Some(10u8))]
#[case(None, Some(5u8), None, Some(5u8))]
#[case(Some("backend-3"), None, Some(&"backend-3".to_string()), None)]
fn test_new_with_optional_fields(
    #[case] name: Option<&str>,
    #[case] weight: Option<u8>,
    #[case] expected_name: Option<&String>,
    #[case] expected_weight: Option<u8>,
) {
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let meta = BackendMeta::new(1u8, name, address, weight);
    assert_eq!(meta.name(), expected_name);
    assert_eq!(meta.weight(), expected_weight);
}

#[rstest]
fn test_getters(#[from(create_test_backend)] backend: BackendMeta) {
    assert!(*backend.id() < 255);
    // Address should contain port >= 8080
    assert!(
        backend.address().as_str().contains(":808")
            || backend.address().as_str().contains(":809")
    );
}

#[rstest]
#[case(Some("my-backend"), Some(&"my-backend".to_string()))]
#[case(None, None)]
fn test_name_getter(#[case] input_name: Option<&str>, #[case] expected: Option<&String>) {
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let meta =
        BackendMeta::new(1u8, input_name, BackendAddress::from(address), Some(10u8));
    assert_eq!(meta.name(), expected);
}

#[rstest]
#[case(Some(25u8), Some(25u8))]
#[case(None, None)]
fn test_weight_getter(#[case] input_weight: Option<u8>, #[case] expected: Option<u8>) {
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let meta = BackendMeta::new(1u8, Some("test"), address, input_weight);
    assert_eq!(meta.weight(), expected);
}

#[test]
fn test_address_getter() {
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 9090);
    let meta =
        BackendMeta::new(1u8, Some("test"), BackendAddress::from(address), Some(10u8));
    let addr = meta.address();
    assert_eq!(addr.as_str(), "192.168.1.1:9090");
}

#[test]
fn test_new_with_string_name() {
    let meta = BackendMeta::new(
        1u8,
        Some("backend-1".to_string()),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        Some(10u8),
    );
    assert_eq!(meta.name(), Some(&"backend-1".to_string()));
}

#[test]
fn test_new_with_backend_address() {
    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let backend_addr = BackendAddress::from(socket_addr);
    let meta = BackendMeta::new(1u8, Some("backend-1"), backend_addr, Some(10u8));
    assert_eq!(meta.address().as_str(), "127.0.0.1:8080");
}

#[rstest]
#[case(
    1u8,
    Some("backend-1"),
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
    Some(10u8)
)]
fn test_debug(
    #[case] id: u8,
    #[case] name: Option<&str>,
    #[case] addr: SocketAddr,
    #[case] weight: Option<u8>,
) {
    let meta = BackendMeta::new(id, name, addr, weight);
    let debug_str = format!("{:?}", meta);
    assert!(!debug_str.is_empty());
}

#[rstest]
#[case(
    1u8,
    Some("backend-1"),
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
    Some(10u8)
)]
fn test_clone(
    #[case] id: u8,
    #[case] name: Option<&str>,
    #[case] addr: SocketAddr,
    #[case] weight: Option<u8>,
) {
    let meta = BackendMeta::new(id, name, addr, weight);
    let cloned = meta.clone();
    assert_eq!(meta, cloned);
    assert_eq!(meta.id(), cloned.id());
    assert_eq!(meta.name(), cloned.name());
    assert_eq!(meta.weight(), cloned.weight());
}

// Property-based tests
proptest! {
    #[test]
    fn backend_meta_clone_property(meta in backend_meta_strategy()) {
        let cloned = meta.clone();
        prop_assert_eq!(meta.id(), cloned.id());
        prop_assert_eq!(meta.name(), cloned.name());
        prop_assert_eq!(meta.weight(), cloned.weight());
    }

    #[test]
    fn backend_meta_reflexive_equality(meta in backend_meta_strategy()) {
        prop_assert_eq!(&meta, &meta);
    }

    #[test]
    fn backend_meta_symmetric_equality(
        meta1 in backend_meta_strategy(),
        meta2 in backend_meta_strategy()
    ) {
        if meta1 == meta2 {
            prop_assert_eq!(&meta2, &meta1);
        }
    }
}

#[test]
fn test_serialize_deserialize_backend_meta() {
    let meta = BackendMeta::new(
        5u8,
        Some("test-backend"),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), 8080),
        Some(15u8),
    );
    let json = serde_json::to_string(&meta).expect("Failed to serialize");
    let deserialized: BackendMeta =
        serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(meta.id(), deserialized.id());
    assert_eq!(meta.name(), deserialized.name());
    assert_eq!(meta.weight(), deserialized.weight());
}

#[test]
fn test_backend_meta_with_backend_address() {
    let addr_str = "192.168.0.1:9000";
    let backend_addr = BackendAddress::parse(addr_str).expect("Failed to parse");
    let meta = BackendMeta::new(10u8, Some("addr-backend"), backend_addr, Some(20u8));
    assert_eq!(meta.id(), &10u8);
    assert_eq!(meta.name(), Some(&"addr-backend".to_string()));
    assert_eq!(meta.weight(), Some(20u8));
}

#[test]
fn test_backend_meta_equality_different_address() {
    let meta1 = BackendMeta::new(
        1u8,
        Some("backend"),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        Some(10u8),
    );
    let meta2 = BackendMeta::new(
        1u8,
        Some("backend"),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
        Some(10u8),
    );
    assert_ne!(meta1, meta2);
}

#[test]
fn test_backend_meta_equality_different_weight() {
    let meta1 = BackendMeta::new(
        1u8,
        Some("backend"),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        Some(10u8),
    );
    let meta2 = BackendMeta::new(
        1u8,
        Some("backend"),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        Some(20u8),
    );
    assert_ne!(meta1, meta2);
}

#[test]
fn test_backend_meta_equality_different_name() {
    let meta1 = BackendMeta::new(
        1u8,
        Some("backend1"),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        Some(10u8),
    );
    let meta2 = BackendMeta::new(
        1u8,
        Some("backend2"),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        Some(10u8),
    );
    assert_ne!(meta1, meta2);
}

#[test]
fn test_backend_meta_hash() {
    use std::collections::HashMap;
    let meta1 = BackendMeta::new(
        1u8,
        Some("backend"),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        Some(10u8),
    );
    let meta2 = BackendMeta::new(
        1u8,
        Some("backend"),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        Some(10u8),
    );
    let mut map = HashMap::new();
    map.insert(meta1, "value");
    assert_eq!(map.get(&meta2), Some(&"value"));
}

#[test]
fn test_backend_meta_with_string_owned() {
    let name_string = "my-string".to_string();
    let meta = BackendMeta::new(
        10u8,
        Some(name_string.as_str()),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        Some(5u8),
    );
    assert_eq!(meta.name(), Some(&"my-string".to_string()));
}

#[test]
fn test_backend_meta_display_with_all_fields() {
    let meta = BackendMeta::new(
        1u8,
        Some("display-backend"),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        Some(10u8),
    );
    let display = format!("{:?}", meta);
    assert!(display.contains("display-backend"));
}

#[test]
fn test_backend_meta_display_without_name() {
    let meta = BackendMeta::new(
        1u8,
        None::<String>,
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
        Some(10u8),
    );
    let display = format!("{:?}", meta);
    assert!(display.contains("127.0.0.1"));
}
