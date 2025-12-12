//! Route table module
//!
use crate::prelude::*;

/// Route table struct
#[derive(Debug, Default, Clone)]
pub struct RouteTable {
    /// Backends
    backends: Vec<BackendMeta>,
}

impl RouteTable {
    /// Create a new route table
    pub fn new(backends: Vec<BackendMeta>) -> Self {
        Self { backends }
    }

    /// Get number of backends
    pub fn len(&self) -> usize {
        self.backends.len()
    }

    /// Check if the route table is empty
    pub fn is_empty(&self) -> bool {
        self.backends.is_empty()
    }

    /// Get backend by id
    pub fn get_by_id(&self, id: BackendId) -> Option<&BackendMeta> {
        self.backends.iter().find(|backend| *backend.id() == id)
    }

    /// Get backend by index (for iteration)
    pub fn get_by_index(&self, idx: usize) -> Option<&BackendMeta> {
        self.backends.get(idx)
    }

    /// Get all backend IDs
    pub fn backend_ids(&self) -> Vec<BackendId> {
        self.backends.iter().map(|b| *b.id()).collect()
    }

    /// Iterate over backends
    pub fn iter(&self) -> impl Iterator<Item = &BackendMeta> {
        self.backends.iter()
    }

    /// Filter backends by health status (takes health registry reference)
    pub fn filter_healthy(&self, health: &HealthRegistry) -> Vec<(usize, &BackendMeta)> {
        self.backends
            .iter()
            .enumerate()
            .filter(|(idx, _)| health.is_alive(*idx))
            .collect()
    }

    /// Find backend index by ID (for ID to index conversion)
    pub fn find_index(&self, id: BackendId) -> Option<usize> {
        self.backends.iter().position(|b| *b.id() == id)
    }

    /// Check if backend exists
    pub fn contains(&self, id: BackendId) -> bool {
        self.backends.iter().any(|b| *b.id() == id)
    }
}
