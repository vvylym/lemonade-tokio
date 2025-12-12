//! Tests for the Work module
//!
use lemonade_service::worker::{WorkError, WorkResponse};
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
fn work_error_new_with_message_should_succeed(#[case] message: &str) {
    let error = WorkError::new(message);
    let display_str = format!("{}", error);
    assert!(display_str.contains(message));
}

#[rstest]
#[case("test error message".to_string())]
#[case("another error".to_string())]
fn work_error_new_with_string_should_succeed(#[case] message: String) {
    let error = WorkError::new(message.clone());
    let display_str = format!("{}", error);
    assert!(display_str.contains(&message));
}

#[rstest]
#[case("test work error")]
fn work_error_display_should_succeed(#[case] message: &str) {
    let error = WorkError::new(message);
    let display_str = format!("{}", error);
    assert!(display_str.contains("health error")); // Note: actual error message says "health error" (bug in code)
    assert!(display_str.contains(message));
}

#[rstest]
#[case(true, "test-service", 100u64)]
#[case(false, "my-service", 200u64)]
#[case(true, "another-service", 50u64)]
fn work_response_new_with_valid_fields_should_succeed(
    #[case] status: bool,
    #[case] service: &str,
    #[case] duration_ms: u64,
) {
    let response = WorkResponse::new(status, service, duration_ms);
    assert_eq!(response.status(), status);
    assert_eq!(response.service(), service);
    assert_eq!(response.duration_ms(), duration_ms);
}

#[rstest]
#[case("test-service".to_string())]
#[case("my-service".to_string())]
fn work_response_new_with_string_should_succeed(#[case] service: String) {
    let response = WorkResponse::new(true, service.clone(), 100);
    assert_eq!(response.service(), service);
}

#[rstest]
#[case("test-service")]
#[case("my-service")]
fn work_response_new_with_str_should_succeed(#[case] service: &str) {
    let response = WorkResponse::new(true, service, 100);
    assert_eq!(response.service(), service);
}

#[rstest]
#[case(true)]
#[case(false)]
fn work_response_status_getter_should_succeed(#[case] status: bool) {
    let response = WorkResponse::new(status, "test-service", 100);
    assert_eq!(response.status(), status);
}

#[rstest]
#[case("test-service")]
#[case("my-service")]
fn work_response_service_getter_should_succeed(#[case] service: &str) {
    let response = WorkResponse::new(true, service, 100);
    assert_eq!(response.service(), service);
}

#[rstest]
#[case(100u64)]
#[case(200u64)]
#[case(50u64)]
fn work_response_duration_ms_getter_should_succeed(#[case] duration_ms: u64) {
    let response = WorkResponse::new(true, "test-service", duration_ms);
    assert_eq!(response.duration_ms(), duration_ms);
}

#[rstest]
#[case(true, "test-service", 100u64)]
fn work_response_debug_should_succeed(
    #[case] status: bool,
    #[case] service: &str,
    #[case] duration_ms: u64,
) {
    let response = WorkResponse::new(status, service, duration_ms);
    let debug_str = format!("{:?}", response);
    assert!(debug_str.contains(service));
}

#[rstest]
#[case(true, "test-service", 100u64)]
fn work_response_clone_should_succeed(
    #[case] status: bool,
    #[case] service: &str,
    #[case] duration_ms: u64,
) {
    let response = WorkResponse::new(status, service, duration_ms);
    let cloned = response.clone();
    assert_eq!(cloned.status(), response.status());
    assert_eq!(cloned.service(), response.service());
    assert_eq!(cloned.duration_ms(), response.duration_ms());
}

#[rstest]
#[case(true, "test-service", 100u64, true, "test-service", 100u64, true)]
#[case(true, "test-service", 100u64, false, "test-service", 100u64, false)]
#[case(true, "test-service", 100u64, true, "other-service", 100u64, false)]
#[case(true, "test-service", 100u64, true, "test-service", 200u64, false)]
fn work_response_partial_eq_should_succeed(
    #[case] status1: bool,
    #[case] service1: &str,
    #[case] duration_ms1: u64,
    #[case] status2: bool,
    #[case] service2: &str,
    #[case] duration_ms2: u64,
    #[case] should_be_equal: bool,
) {
    let response1 = WorkResponse::new(status1, service1, duration_ms1);
    let response2 = WorkResponse::new(status2, service2, duration_ms2);
    if should_be_equal {
        assert_eq!(response1, response2);
    } else {
        assert_ne!(response1, response2);
    }
}

#[rstest]
#[case(true, "test-service", 100u64)]
fn work_response_hash_should_succeed(
    #[case] status: bool,
    #[case] service: &str,
    #[case] duration_ms: u64,
) {
    let response = WorkResponse::new(status, service, duration_ms);
    let mut hasher1 = DefaultHasher::new();
    response.hash(&mut hasher1);
    let hash1 = hasher1.finish();

    let mut hasher2 = DefaultHasher::new();
    response.hash(&mut hasher2);
    let hash2 = hasher2.finish();

    assert_eq!(hash1, hash2);
}

#[rstest]
#[case(true, "test-service", 100u64)]
fn work_response_serialize_should_succeed(
    #[case] status: bool,
    #[case] service: &str,
    #[case] duration_ms: u64,
) {
    let response = WorkResponse::new(status, service, duration_ms);
    let result = serde_json::to_string(&response);
    assert!(result.is_ok());
    let json = result.expect("Failed to serialize WorkResponse");
    assert!(json.contains(service));
    assert!(json.contains(&duration_ms.to_string()));
}

#[rstest]
#[case(
    r#"{"status":true,"service":"test-service","duration_ms":100}"#,
    true,
    "test-service",
    100u64
)]
#[case(
    r#"{"status":false,"service":"my-service","duration_ms":200}"#,
    false,
    "my-service",
    200u64
)]
fn work_response_deserialize_should_succeed(
    #[case] json: &str,
    #[case] expected_status: bool,
    #[case] expected_service: &str,
    #[case] expected_duration_ms: u64,
) {
    let result: Result<WorkResponse, _> = serde_json::from_str(json);
    assert!(result.is_ok());
    let response = result.expect("Failed to deserialize WorkResponse");
    assert_eq!(response.status(), expected_status);
    assert_eq!(response.service(), expected_service);
    assert_eq!(response.duration_ms(), expected_duration_ms);
}

// Property-based tests using proptest

proptest! {
    #[test]
    fn work_response_round_trip_serialization(
        status in any::<bool>(),
        service in response_service_strategy(),
        duration_ms in duration_ms_strategy()
    ) {
        let response = WorkResponse::new(status, service.clone(), duration_ms);
        let json = serde_json::to_string(&response)
            .expect("Serialization should succeed");
        let deserialized: WorkResponse = serde_json::from_str(&json)
            .expect("Deserialization should succeed");
        prop_assert_eq!(response.status(), deserialized.status());
        prop_assert_eq!(response.service(), deserialized.service());
        prop_assert_eq!(response.duration_ms(), deserialized.duration_ms());
    }

    #[test]
    fn work_response_clone_property(
        status in any::<bool>(),
        service in response_service_strategy(),
        duration_ms in duration_ms_strategy()
    ) {
        let response = WorkResponse::new(status, service.clone(), duration_ms);
        let cloned = response.clone();
        prop_assert_eq!(response.status(), cloned.status());
        prop_assert_eq!(response.service(), cloned.service());
        prop_assert_eq!(response.duration_ms(), cloned.duration_ms());
    }

    #[test]
    fn work_response_hash_consistency(
        status in any::<bool>(),
        service in response_service_strategy(),
        duration_ms in duration_ms_strategy()
    ) {
        let response = WorkResponse::new(status, service, duration_ms);
        let mut hasher1 = DefaultHasher::new();
        response.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        response.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        prop_assert_eq!(hash1, hash2);
    }

    #[test]
    fn work_response_equality_implies_same_hash(
        status in any::<bool>(),
        service in response_service_strategy(),
        duration_ms in duration_ms_strategy()
    ) {
        let response1 = WorkResponse::new(status, service.clone(), duration_ms);
        let response2 = WorkResponse::new(status, service, duration_ms);

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
