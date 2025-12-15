//! Route table module
//!
use crate::prelude::*;

/// Route table struct
#[derive(Debug, Default)]
pub struct RouteTable {
    /// Backends (private for encapsulation)
    backends: DashMap<BackendId, Arc<Backend>>,
}

impl RouteTable {
    /// Create a new route table from backend configs
    pub fn new(configs: Vec<BackendConfig>) -> Self {
        let map = DashMap::from_iter(
            configs
                .into_iter()
                .map(|config| (config.id, Arc::new(Backend::new(config)))),
        );
        Self { backends: map }
    }

    /// Get backend by id
    pub fn get(&self, id: BackendId) -> Option<Arc<Backend>> {
        self.backends.get(&id).map(|entry| entry.value().clone())
    }

    /// Get all backends
    pub fn all_backends(&self) -> Vec<Arc<Backend>> {
        self.backends
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get healthy backends (alive && not draining)
    pub fn healthy_backends(&self) -> Vec<Arc<Backend>> {
        self.backends
            .iter()
            .filter(|entry| {
                let backend = entry.value();
                backend.is_alive() && backend.is_active()
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get active backends (not draining)
    pub fn active_backends(&self) -> Vec<Arc<Backend>> {
        self.backends
            .iter()
            .filter(|entry| entry.value().is_active())
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get draining backends
    pub fn draining_backends(&self) -> Vec<Arc<Backend>> {
        self.backends
            .iter()
            .filter(|entry| entry.value().is_draining())
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get all backend IDs
    pub fn backend_ids(&self) -> Vec<BackendId> {
        self.backends.iter().map(|entry| *entry.key()).collect()
    }

    /// Get number of backends
    pub fn len(&self) -> usize {
        self.backends.len()
    }

    /// Check if the route table is empty
    pub fn is_empty(&self) -> bool {
        self.backends.is_empty()
    }

    /// Insert a backend
    pub fn insert(&self, backend: Arc<Backend>) {
        self.backends.insert(backend.id(), backend);
    }

    /// Remove a backend by id
    pub fn remove(&self, id: BackendId) -> Option<Arc<Backend>> {
        self.backends.remove(&id).map(|(_, backend)| backend)
    }

    /// Check if backend exists
    pub fn contains(&self, id: BackendId) -> bool {
        self.backends.contains_key(&id)
    }

    /// Find backend index by ID (for compatibility with old code during migration)
    /// Note: This is less efficient with DashMap, consider removing after migration
    pub fn find_index(&self, id: BackendId) -> Option<usize> {
        self.backends.iter().position(|entry| *entry.key() == id)
    }
}
