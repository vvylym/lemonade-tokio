//! Metrics registry module
//!
use crate::prelude::*;

/// Performance struct
#[derive(Clone, Debug, Default)]
pub struct MetricsSnapshot {
    /// Per backend performance
    per_backend: DashMap<BackendId, BackendMetrics>,
}

impl MetricsSnapshot {
    /// Get metrics for specific backend
    pub fn get(&self, backend_id: BackendId) -> Option<BackendMetrics> {
        self.per_backend
            .get(&backend_id)
            .map(|entry| entry.value().clone())
    }

    /// Update metrics for backend
    pub fn update(&self, backend_id: BackendId, metrics: BackendMetrics) {
        self.per_backend.insert(backend_id, metrics);
    }

    /// Get all backend IDs with metrics
    pub fn backend_ids(&self) -> Vec<BackendId> {
        self.per_backend.iter().map(|entry| *entry.key()).collect()
    }

    /// Check if metrics exist for backend
    pub fn has_metrics(&self, backend_id: BackendId) -> bool {
        self.per_backend.contains_key(&backend_id)
    }

    /// Get average latency for backend (common metric for strategies)
    pub fn avg_latency(&self, backend_id: BackendId) -> Option<f64> {
        self.get(backend_id).map(|m| m.avg_latency_ms)
    }

    /// Get error rate for backend
    pub fn error_rate(&self, backend_id: BackendId) -> Option<f32> {
        self.get(backend_id).map(|m| m.error_rate)
    }

    /// Clear metrics for a backend (when backend removed)
    pub fn remove(&self, backend_id: BackendId) {
        self.per_backend.remove(&backend_id);
    }
}

/// Backend performance struct
#[derive(Debug, Clone, Default)]
pub struct BackendMetrics {
    /// Average latency
    pub avg_latency_ms: f64,
    /// 95th percentile latency
    pub p95_latency_ms: f64,
    /// Error rate
    pub error_rate: f32,
    /// Last updated timestamp
    pub last_updated_ms: u64,
}
