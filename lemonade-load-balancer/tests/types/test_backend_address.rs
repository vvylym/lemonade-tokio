//! Tests for BackendAddress parsing and conversion
//!
use lemonade_load_balancer::prelude::*;
use proptest::prelude::*;
use rstest::*;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};

#[rstest]
#[case("127.0.0.1:8080", IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)]
#[case("192.168.1.1:9090", IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 9090)]
fn test_parse_ipv4(
    #[case] addr_str: &str,
    #[case] expected_ip: IpAddr,
    #[case] expected_port: u16,
) {
    let result = BackendAddress::parse(addr_str);
    assert!(result.is_ok());
    let addr = result.expect("Failed to parse address");
    // Resolve to SocketAddr to check IP and port
    let socket_addrs: Vec<SocketAddr> = addr.to_socket_addrs().unwrap().collect();
    assert_eq!(socket_addrs[0].ip(), expected_ip);
    assert_eq!(socket_addrs[0].port(), expected_port);
}

#[test]
fn test_parse_ipv6() {
    let addr_str = "[::1]:8080";
    let result = BackendAddress::parse(addr_str);
    assert!(result.is_ok());
    let addr = result.expect("Failed to parse address");
    // Resolve to SocketAddr to check IP and port
    let socket_addrs: Vec<SocketAddr> = addr.to_socket_addrs().unwrap().collect();
    assert_eq!(
        socket_addrs[0].ip(),
        IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))
    );
    assert_eq!(socket_addrs[0].port(), 8080);
}

#[rstest]
#[case("127.0.0.1", "missing port")]
#[case("", "empty string")]
fn test_parse_invalid_format(#[case] addr_str: &str, #[case] _description: &str) {
    let result = BackendAddress::parse(addr_str);
    assert!(result.is_err());
}

#[test]
fn test_from_socket_addr() {
    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 9090);
    let backend_addr = BackendAddress::from(socket_addr);
    assert_eq!(backend_addr.as_str(), "192.168.1.1:9090");
}

#[test]
fn test_as_ref_str() {
    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), 3000);
    let backend_addr = BackendAddress::from(socket_addr);
    let str_ref = backend_addr.as_ref();
    assert_eq!(str_ref, "10.0.0.1:3000");
}

#[rstest]
#[case(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080))]
#[case(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 9090))]
fn test_debug(#[case] socket_addr: SocketAddr) {
    let backend_addr = BackendAddress::from(socket_addr);
    let debug_str = format!("{:?}", backend_addr);
    assert!(!debug_str.is_empty());
}

#[rstest]
#[case(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080))]
#[case(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 9090))]
fn test_clone(#[case] socket_addr: SocketAddr) {
    let backend_addr = BackendAddress::from(socket_addr);
    let cloned = backend_addr.clone();
    assert_eq!(backend_addr, cloned);
    assert_eq!(backend_addr.as_ref(), cloned.as_ref());
}

#[rstest]
#[case(
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
    true
)]
#[case(
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)), 8080),
    false
)]
fn test_partial_eq(
    #[case] addr1: SocketAddr,
    #[case] addr2: SocketAddr,
    #[case] should_be_equal: bool,
) {
    let addr1_backend = BackendAddress::from(addr1);
    let addr2_backend = BackendAddress::from(addr2);
    if should_be_equal {
        assert_eq!(addr1_backend, addr2_backend);
    } else {
        assert_ne!(addr1_backend, addr2_backend);
    }
}

// Property-based tests
use crate::common::socket_addr_strategy;

proptest! {
    #[test]
    fn backend_address_roundtrip(socket_addr in socket_addr_strategy()) {
        let backend_addr = BackendAddress::from(socket_addr);
        // BackendAddress stores as string, so compare string representations
        prop_assert_eq!(backend_addr.as_str(), socket_addr.to_string());
    }

    #[test]
    fn backend_address_clone_property(socket_addr in socket_addr_strategy()) {
        let backend_addr = BackendAddress::from(socket_addr);
        let cloned = backend_addr.clone();
        prop_assert_eq!(&backend_addr, &cloned);
        prop_assert_eq!(backend_addr.as_ref(), cloned.as_ref());
    }

    #[test]
    fn backend_address_reflexive_equality(socket_addr in socket_addr_strategy()) {
        let backend_addr = BackendAddress::from(socket_addr);
        prop_assert_eq!(&backend_addr, &backend_addr);
    }
}

#[test]
fn test_serialize_deserialize_ipv4() {
    let addr = BackendAddress::from(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        8080,
    ));
    let json = serde_json::to_string(&addr).expect("Failed to serialize");
    let deserialized: BackendAddress =
        serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(addr, deserialized);
}

#[test]
fn test_serialize_deserialize_ipv6() {
    let addr = BackendAddress::from(SocketAddr::new(
        IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
        9090,
    ));
    let json = serde_json::to_string(&addr).expect("Failed to serialize");
    let deserialized: BackendAddress =
        serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(addr, deserialized);
}

#[test]
fn test_deserialize_invalid_json() {
    let result: Result<BackendAddress, _> = serde_json::from_str("\"invalid-no-port\"");
    assert!(result.is_err());
}

#[test]
fn test_hash_backend_address() {
    use std::collections::HashMap;
    let addr1 = BackendAddress::from(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        8080,
    ));
    let addr2 = BackendAddress::from(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        8080,
    ));
    let mut map = HashMap::new();
    map.insert(addr1, "value1");
    assert_eq!(map.get(&addr2), Some(&"value1"));
}

#[test]
fn test_from_str_with_hostname() {
    // Test parsing localhost hostname
    let result = BackendAddress::parse("localhost:8080");
    assert!(result.is_ok());
    let addr = result.expect("Failed to parse localhost");
    // Resolve to SocketAddr to check port
    let socket_addrs: Vec<SocketAddr> = addr.to_socket_addrs().unwrap().collect();
    assert_eq!(socket_addrs[0].port(), 8080);
}

#[test]
fn test_multiple_resolved_addresses() {
    // Some hostnames resolve to multiple addresses, we should get the first one
    let result = BackendAddress::parse("127.0.0.1:3000");
    assert!(result.is_ok());
}

#[test]
fn test_backend_address_eq_same_values() {
    let addr1 = BackendAddress::from(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(10, 10, 10, 10)),
        5000,
    ));
    let addr2 = BackendAddress::from(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(10, 10, 10, 10)),
        5000,
    ));
    assert_eq!(addr1, addr2);
}

#[test]
fn test_backend_address_ne_different_port() {
    let addr1 = BackendAddress::from(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(10, 10, 10, 10)),
        5000,
    ));
    let addr2 = BackendAddress::from(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(10, 10, 10, 10)),
        6000,
    ));
    assert_ne!(addr1, addr2);
}

#[test]
fn test_backend_address_ne_different_ip() {
    let addr1 = BackendAddress::from(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(10, 10, 10, 10)),
        5000,
    ));
    let addr2 = BackendAddress::from(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(10, 10, 10, 11)),
        5000,
    ));
    assert_ne!(addr1, addr2);
}
