//! Strategy test helpers
//!
//! Provides helper functions specific to strategy testing

use lemonade_load_balancer::prelude::*;
use std::sync::Arc;

/// Set all backends in context as healthy
///
/// Given: a context
/// When: setting all backends as healthy
/// Then: all backends in routing table are marked as alive
/// Note: Backends start healthy by default, so this is mainly for ensuring state
pub fn set_all_backends_healthy(ctx: &Arc<Context>) {
    let routing = ctx.routing_table();
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    for backend in routing.all_backends() {
        backend.set_health(true, now_ms);
    }
}

/// Set specific backends as healthy by their IDs
///
/// Given: a context and backend IDs
/// When: setting backends as healthy
/// Then: specified backends are marked as alive
pub fn set_backends_healthy_by_id(ctx: &Arc<Context>, backend_ids: &[u8]) {
    let routing = ctx.routing_table();
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    for &backend_id in backend_ids {
        if let Some(backend) = routing.get(backend_id) {
            backend.set_health(true, now_ms);
        }
    }
}

/// Assert that a backend was selected
///
/// Given: a result from pick_backend and expected backend ID
/// When: checking the result
/// Then: asserts that the backend matches expected ID
pub fn assert_backend_selected(
    result: Result<BackendMeta, StrategyError>,
    expected_id: u8,
) {
    assert!(result.is_ok(), "Expected backend selection to succeed");
    let backend = result.expect("Failed to get backend");
    assert_eq!(
        backend.id(),
        &expected_id,
        "Expected backend ID {} but got {}",
        expected_id,
        backend.id()
    );
}

/// Assert that no backend was available
///
/// Given: a result from pick_backend
/// When: checking the result
/// Then: asserts that NoBackendAvailable error was returned
pub fn assert_no_backend_available(result: Result<BackendMeta, StrategyError>) {
    assert!(result.is_err(), "Expected backend selection to fail");
    assert!(
        matches!(
            result.expect_err("Expected error"),
            StrategyError::NoBackendAvailable
        ),
        "Expected NoBackendAvailable error"
    );
}
