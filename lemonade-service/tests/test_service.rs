//! Tests for the Service module
//!
use lemonade_service::worker::{HealthService, WorkService, WorkerServiceImpl};
use proptest::prelude::*;
use rstest::*;
use std::time::Duration;

mod common;
use common::*;

#[rstest]
#[case("test-service", Duration::from_millis(2))]
#[case("my-service", Duration::from_millis(4))]
fn worker_service_impl_new_with_valid_config_should_succeed(
    #[case] service_name: &str,
    #[case] work_delay: Duration,
) {
    let service = WorkerServiceImpl::new(service_name, work_delay);
    assert_eq!(service.service_name(), service_name);
    assert_eq!(service.work_delay(), work_delay);
}

#[rstest]
#[case("test-service", Duration::from_millis(3))]
fn worker_service_impl_service_name_getter_should_succeed(
    #[case] service_name: &str,
    #[case] work_delay: Duration,
) {
    let service = WorkerServiceImpl::new(service_name, work_delay);
    assert_eq!(service.service_name(), service_name);
}

#[rstest]
#[case("test-service", Duration::from_millis(1))]
fn worker_service_impl_work_delay_getter_should_succeed(
    #[case] service_name: &str,
    #[case] work_delay: Duration,
) {
    let service = WorkerServiceImpl::new(service_name, work_delay);
    assert_eq!(service.work_delay(), work_delay);
}

#[rstest]
#[case("test-service", Duration::from_millis(2), true)]
#[case("", Duration::from_millis(3), false)]
#[case("test-service", Duration::ZERO, false)]
#[case("", Duration::ZERO, false)]
fn worker_service_impl_validate_should_succeed(
    #[case] service_name: &str,
    #[case] work_delay: Duration,
    #[case] expected_valid: bool,
) {
    let service = WorkerServiceImpl::new(service_name, work_delay);
    assert_eq!(service.validate(), expected_valid);
}

#[tokio::test]
#[rstest]
#[case("test-service", Duration::from_millis(1), true)]
#[case("", Duration::from_millis(2), false)]
#[case("test-service", Duration::ZERO, false)]
#[case("", Duration::ZERO, false)]
async fn health_service_health_check_should_succeed(
    #[case] service_name: &str,
    #[case] work_delay: Duration,
    #[case] should_succeed: bool,
) {
    let service = WorkerServiceImpl::new(service_name, work_delay);
    let result = service.health_check().await;

    if should_succeed {
        assert!(result.is_ok());
        let response = result.expect("Failed to health check");
        assert_eq!(response.status(), "ok");
        assert_eq!(response.service(), service_name);
    } else {
        assert!(result.is_err());
        let error = result.expect_err("Expected error");
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("service name or work delay is empty"));
    }
}

#[tokio::test]
#[rstest]
#[case("test-service", Duration::from_millis(1), true)]
#[case("", Duration::from_millis(3), false)]
#[case("test-service", Duration::ZERO, false)]
#[case("", Duration::ZERO, false)]
async fn work_service_work_should_succeed(
    #[case] service_name: &str,
    #[case] work_delay: Duration,
    #[case] should_succeed: bool,
) {
    let service = WorkerServiceImpl::new(service_name, work_delay);
    let result = service.work().await;

    if should_succeed {
        assert!(result.is_ok());
        let response = result.expect("Failed to work");
        assert!(response.status());
        assert_eq!(response.service(), service_name);
        assert_eq!(response.duration_ms(), work_delay.as_millis() as u64);
    } else {
        assert!(result.is_err());
        let error = result.expect_err("Expected error");
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("service name or work delay is empty"));
    }
}

// Property-based tests using proptest

proptest! {
    #[test]
    fn service_validate_property(
        service_name in service_name_strategy(),
        work_delay in duration_strategy()
    ) {
        let service = WorkerServiceImpl::new(service_name.clone(), work_delay);
        let is_valid = service.validate();
        let expected_valid = !service_name.is_empty() && !work_delay.is_zero();
        prop_assert_eq!(is_valid, expected_valid);
    }

    #[test]
    fn service_validate_with_empty_name_should_fail(
        work_delay in duration_strategy()
    ) {
        let service = WorkerServiceImpl::new("", work_delay);
        prop_assert!(!service.validate());
    }

    #[test]
    fn service_validate_with_zero_delay_should_fail(
        service_name in service_name_strategy()
    ) {
        let service = WorkerServiceImpl::new(service_name, Duration::ZERO);
        prop_assert!(!service.validate());
    }
}

// Property-based tests for async functions using proptest with tokio runtime
proptest! {
    #[test]
    fn health_check_succeeds_if_validate_returns_true(
        service_name in service_name_strategy(),
        work_delay in duration_strategy()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let service = WorkerServiceImpl::new(service_name.clone(), work_delay);
        let is_valid = service.validate();

        let result = rt.block_on(service.health_check());
        let health_check_succeeded = result.is_ok();

        prop_assert_eq!(is_valid, health_check_succeeded);
    }

    #[test]
    fn work_succeeds_if_validate_returns_true(
        service_name in service_name_strategy(),
        work_delay in duration_strategy()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let service = WorkerServiceImpl::new(service_name.clone(), work_delay);
        let is_valid = service.validate();

        let result = rt.block_on(service.work());
        let work_succeeded = result.is_ok();

        prop_assert_eq!(is_valid, work_succeeded);
    }
}
