//! Tests for the Health module
//!
use lemonade_service::worker::{HealthError, HealthResponse};
use proptest::prelude::*;
use rstest::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

mod common;
use common::*;

#[rstest]
#[case("test error message")]
#[case("another error")]
#[case("")]
fn health_error_new_with_message_should_succeed(#[case] message: &str) {
    let error = HealthError::new(message);
    let display_str = format!("{}", error);
    assert!(display_str.contains(message));
}

#[rstest]
#[case("test error message".to_string())]
#[case("another error".to_string())]
fn health_error_new_with_string_should_succeed(#[case] message: String) {
    let error = HealthError::new(message.clone());
    let display_str = format!("{}", error);
    assert!(display_str.contains(&message));
}

#[rstest]
#[case("ok", "test-service")]
#[case("healthy", "my-service")]
#[case("degraded", "another-service")]
fn health_response_new_with_strings_should_succeed(
    #[case] status: &str,
    #[case] service: &str,
) {
    let response = HealthResponse::new(status, service);
    assert_eq!(response.status(), status);
    assert_eq!(response.service(), service);
}

#[rstest]
#[case("ok", "test-service")]
fn health_response_status_getter_should_succeed(
    #[case] status: &str,
    #[case] service: &str,
) {
    let response = HealthResponse::new(status, service);
    assert_eq!(response.status(), status);
}

#[rstest]
#[case("ok", "test-service")]
fn health_response_service_getter_should_succeed(
    #[case] status: &str,
    #[case] service: &str,
) {
    let response = HealthResponse::new(status, service);
    assert_eq!(response.service(), service);
}

#[rstest]
#[case("ok", "test-service")]
fn health_response_debug_should_succeed(#[case] status: &str, #[case] service: &str) {
    let response = HealthResponse::new(status, service);
    let debug_str = format!("{:?}", response);
    assert!(debug_str.contains(status));
    assert!(debug_str.contains(service));
}

#[rstest]
#[case("ok", "test-service")]
fn health_response_clone_should_succeed(#[case] status: &str, #[case] service: &str) {
    let response = HealthResponse::new(status, service);
    let cloned = response.clone();
    assert_eq!(cloned.status(), response.status());
    assert_eq!(cloned.service(), response.service());
}

#[rstest]
#[case("ok", "test-service", "ok", "test-service", true)]
#[case("ok", "test-service", "error", "test-service", false)]
#[case("ok", "test-service", "ok", "other-service", false)]
fn health_response_partial_eq_should_succeed(
    #[case] status1: &str,
    #[case] service1: &str,
    #[case] status2: &str,
    #[case] service2: &str,
    #[case] should_be_equal: bool,
) {
    let response1 = HealthResponse::new(status1, service1);
    let response2 = HealthResponse::new(status2, service2);
    if should_be_equal {
        assert_eq!(response1, response2);
    } else {
        assert_ne!(response1, response2);
    }
}

#[rstest]
#[case("ok", "test-service")]
fn health_response_hash_should_succeed(#[case] status: &str, #[case] service: &str) {
    let response = HealthResponse::new(status, service);
    let mut hasher1 = DefaultHasher::new();
    response.hash(&mut hasher1);
    let hash1 = hasher1.finish();

    let mut hasher2 = DefaultHasher::new();
    response.hash(&mut hasher2);
    let hash2 = hasher2.finish();

    assert_eq!(hash1, hash2);
}

#[rstest]
#[case("ok", "test-service")]
fn health_response_serialize_should_succeed(#[case] status: &str, #[case] service: &str) {
    let response = HealthResponse::new(status, service);
    let result = serde_json::to_string(&response);
    assert!(result.is_ok());
    let json = result.expect("Failed to serialize HealthResponse");
    assert!(json.contains(status));
    assert!(json.contains(service));
}

#[rstest]
#[case(r#"{"status":"ok","service":"test-service"}"#, "ok", "test-service")]
#[case(
    r#"{"status":"healthy","service":"my-service"}"#,
    "healthy",
    "my-service"
)]
fn health_response_deserialize_should_succeed(
    #[case] json: &str,
    #[case] expected_status: &str,
    #[case] expected_service: &str,
) {
    let result: Result<HealthResponse, _> = serde_json::from_str(json);
    assert!(result.is_ok());
    let response = result.expect("Failed to deserialize HealthResponse");
    assert_eq!(response.status(), expected_status);
    assert_eq!(response.service(), expected_service);
}

// Property-based tests using proptest

proptest! {
    #[test]
    fn health_response_round_trip_serialization(
        status in status_strategy(),
        service in response_service_strategy()
    ) {
        let response = HealthResponse::new(status.clone(), service.clone());
        let json = serde_json::to_string(&response)
            .expect("Serialization should succeed");
        let deserialized: HealthResponse = serde_json::from_str(&json)
            .expect("Deserialization should succeed");
        prop_assert_eq!(response.status(), deserialized.status());
        prop_assert_eq!(response.service(), deserialized.service());
    }

    #[test]
    fn health_response_clone_property(
        status in status_strategy(),
        service in response_service_strategy()
    ) {
        let response = HealthResponse::new(status.clone(), service.clone());
        let cloned = response.clone();
        prop_assert_eq!(response.status(), cloned.status());
        prop_assert_eq!(response.service(), cloned.service());
    }

    #[test]
    fn health_response_hash_consistency(
        status in status_strategy(),
        service in response_service_strategy()
    ) {
        let response = HealthResponse::new(status, service);
        let mut hasher1 = DefaultHasher::new();
        response.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        response.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        prop_assert_eq!(hash1, hash2);
    }

    #[test]
    fn health_response_equality_implies_same_hash(
        status in status_strategy(),
        service in response_service_strategy()
    ) {
        let response1 = HealthResponse::new(status.clone(), service.clone());
        let response2 = HealthResponse::new(status, service);

        if response1 == response2 {
            let mut hasher1 = DefaultHasher::new();
            response1.hash(&mut hasher1);
            let hash1 = hasher1.finish();

            let mut hasher2 = DefaultHasher::new();
            response2.hash(&mut hasher2);
            let hash2 = hasher2.finish();

            prop_assert_eq!(hash1, hash2);
        }
    }
}
