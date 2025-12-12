//! Tests for BackendAddress parsing and conversion
//!
use lemonade_load_balancer::prelude::*;
use proptest::prelude::*;
use rstest::*;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

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
    assert_eq!(addr.as_ref().ip(), expected_ip);
    assert_eq!(addr.as_ref().port(), expected_port);
}

#[test]
fn test_parse_ipv6() {
    let addr_str = "[::1]:8080";
    let result = BackendAddress::parse(addr_str);
    assert!(result.is_ok());
    let addr = result.expect("Failed to parse address");
    assert_eq!(
        addr.as_ref().ip(),
        IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))
    );
    assert_eq!(addr.as_ref().port(), 8080);
}

#[rstest]
#[case("127.0.0.1", "missing port")]
#[case("", "empty string")]
fn test_parse_invalid_format(#[case] addr_str: &str, #[case] _description: &str) {
    let result = BackendAddress::parse(addr_str);
    assert!(result.is_err());
}

#[test]
fn test_parse_unresolvable_hostname() {
    let addr_str = "invalid-hostname-that-does-not-exist.example:8080";
    let result = BackendAddress::parse(addr_str);
    assert!(result.is_err());
    match result.expect_err("Failed to parse address") {
        BackendAddressError::ResolutionFailed(_) => {}
        _ => panic!("Expected ResolutionFailed error"),
    }
}

#[test]
fn test_from_socket_addr() {
    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 9090);
    let backend_addr = BackendAddress::from(socket_addr);
    assert_eq!(backend_addr.as_ref(), &socket_addr);
}

#[test]
fn test_as_ref_socket_addr() {
    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), 3000);
    let backend_addr = BackendAddress::from(socket_addr);
    let socket_ref = backend_addr.as_ref();
    assert_eq!(socket_ref, &socket_addr);
    assert_eq!(socket_ref.ip(), IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
    assert_eq!(socket_ref.port(), 3000);
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
        prop_assert_eq!(backend_addr.as_ref(), &socket_addr);
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
