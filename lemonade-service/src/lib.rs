//! Lemonade Service Library
//!

pub mod config;
pub mod error_response;
pub mod worker;

use crate::worker::WorkerServiceImpl;
use std::sync::Arc;

/// Application state containing the worker service and config
#[derive(Clone)]
pub struct AppState {
    /// Worker service
    pub worker_service: Arc<WorkerServiceImpl>,
    /// Config
    pub config: Arc<crate::config::Config>,
}

impl AppState {
    /// Create a new application state from config
    pub fn new(config: crate::config::Config) -> Self {
        let worker_service = WorkerServiceImpl::new(
            config.service_name().to_string(),
            config.work_delay(),
        );
        Self {
            worker_service: Arc::new(worker_service),
            config: Arc::new(config),
        }
    }
}
