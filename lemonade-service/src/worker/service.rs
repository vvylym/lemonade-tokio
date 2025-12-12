//! Worker service module
//!
use std::time::Duration;

use super::{
    HealthError, HealthResponse, HealthService, WorkError, WorkResponse, WorkService,
    constants::{WORKER_SERVICE_ERROR_MESSAGE, WORKER_SERVICE_OK_MESSAGE},
};
use async_trait::async_trait;

/// Worker service struct
#[derive(Debug, Clone)]
pub struct WorkerServiceImpl {
    /// Service name
    service_name: String,
    /// Work delay
    work_delay: Duration,
}

impl WorkerServiceImpl {
    /// Create a new worker service
    pub fn new(service_name: impl Into<String>, work_delay: Duration) -> Self {
        Self {
            service_name: service_name.into(),
            work_delay,
        }
    }

    /// Get the service name
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// Get the work delay
    #[must_use]
    pub fn work_delay(&self) -> Duration {
        self.work_delay
    }

    /// Validate config
    #[inline]
    pub fn validate(&self) -> bool {
        !self.service_name().is_empty() && !self.work_delay().is_zero()
    }
}

#[async_trait]
impl HealthService for WorkerServiceImpl {
    /// Perform a health check
    async fn health_check(&self) -> Result<HealthResponse, HealthError> {
        // Validate config
        if self.validate() {
            // If config is valid, return health response
            Ok(HealthResponse::new(
                WORKER_SERVICE_OK_MESSAGE,
                self.service_name(),
            ))
        } else {
            // If config is invalid, return error
            Err(HealthError::new(WORKER_SERVICE_ERROR_MESSAGE.to_owned()))
        }
    }
}

#[async_trait]
impl WorkService for WorkerServiceImpl {
    /// Perform a work
    async fn work(&self) -> Result<WorkResponse, WorkError> {
        // Validate config
        if self.validate() {
            // config is valid, sleep for work delay
            std::thread::sleep(self.work_delay());
            // return work response
            Ok(WorkResponse::new(
                true,
                self.service_name(),
                self.work_delay().as_millis() as u64,
            ))
        } else {
            // If config is invalid, return error
            Err(WorkError::new(WORKER_SERVICE_ERROR_MESSAGE.to_owned()))
        }
    }
}
