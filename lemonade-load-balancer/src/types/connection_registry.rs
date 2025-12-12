//! Connection registry module
//!
use crate::prelude::*;

/// Connection registry struct
#[derive(Debug, Default)]
pub struct ConnectionRegistry {
    /// Total connections
    total: AtomicUsize,
    /// Per backend connections
    per_backend: Box<[AtomicUsize]>,
}

impl ConnectionRegistry {
    /// Create a new connection registry
    pub fn new(cap: usize) -> Self {
        let mut v = Vec::with_capacity(cap);
        for _ in 0..cap {
            v.push(AtomicUsize::new(0));
        }
        Self {
            total: AtomicUsize::new(0),
            per_backend: v.into_boxed_slice(),
        }
    }

    /// Increment connection count for a backend index
    pub fn increment(&self, backend_idx: usize) {
        self.total.fetch_add(1, Ordering::Relaxed);
        if backend_idx < self.per_backend.len() {
            self.per_backend[backend_idx].fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Decrement connection count for a backend index
    pub fn decrement(&self, backend_idx: usize) {
        self.total.fetch_sub(1, Ordering::Relaxed);
        if backend_idx < self.per_backend.len() {
            self.per_backend[backend_idx].fetch_sub(1, Ordering::Relaxed);
        }
    }

    /// Get total connections
    pub fn total(&self) -> usize {
        self.total.load(Ordering::Relaxed)
    }

    /// Get snapshot of per backend connections
    pub fn snapshot_per_backend(&self) -> Vec<usize> {
        self.per_backend
            .iter()
            .map(|a| a.load(Ordering::Relaxed))
            .collect()
    }

    /// Get connection count for specific backend
    pub fn get(&self, backend_idx: usize) -> usize {
        if backend_idx < self.per_backend.len() {
            self.per_backend[backend_idx].load(Ordering::Relaxed)
        } else {
            0
        }
    }

    /// Check if backend has capacity (for health/metrics to avoid overloading)
    pub fn has_capacity(
        &self,
        backend_idx: usize,
        max_connections: Option<usize>,
    ) -> bool {
        if let Some(max) = max_connections {
            self.get(backend_idx) < max
        } else {
            true // No limit
        }
    }

    /// Migrate connection counts when backends change
    pub fn migrate(
        &self,
        new_cap: usize,
        id_mapping: &[Option<usize>], // maps old index -> new index
    ) -> ConnectionRegistry {
        let new_registry = ConnectionRegistry::new(new_cap);

        // Migrate total count
        let old_total = self.total.load(Ordering::Relaxed);
        new_registry.total.store(old_total, Ordering::Relaxed);

        // Migrate per-backend counts
        for (old_idx, new_idx_opt) in id_mapping.iter().enumerate() {
            if let Some(new_idx) = new_idx_opt
                && old_idx < self.per_backend.len()
                && *new_idx < new_cap
            {
                let count = self.per_backend[old_idx].load(Ordering::Relaxed);
                new_registry.per_backend[*new_idx].store(count, Ordering::Relaxed);
            }
        }

        new_registry
    }

    /// Get all backend indices with their connection counts
    pub fn iter_counts(&self) -> impl Iterator<Item = (usize, usize)> {
        self.per_backend
            .iter()
            .enumerate()
            .map(|(idx, count)| (idx, count.load(Ordering::Relaxed)))
    }
}
