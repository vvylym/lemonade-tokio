//! Health registry module
//!
use crate::prelude::*;

/// Health registry struct
#[derive(Debug, Default)]
pub struct HealthRegistry {
    /// Alive status
    alive: Box<[AtomicU8]>,
    /// Last check timestamp
    last_check_ms: Box<[AtomicU64]>,
}

impl HealthRegistry {
    /// Create a new health registry
    pub fn new(cap: usize) -> Self {
        let mut alive = Vec::with_capacity(cap);
        let mut last = Vec::with_capacity(cap);
        for _ in 0..cap {
            alive.push(AtomicU8::new(0));
            last.push(AtomicU64::new(0));
        }
        Self {
            alive: alive.into_boxed_slice(),
            last_check_ms: last.into_boxed_slice(),
        }
    }

    /// Set alive status
    pub fn set_alive(&self, idx: usize, alive: bool, now_ms: u64) {
        if idx < self.alive.len() {
            self.alive[idx].store(if alive { 1 } else { 0 }, Ordering::Relaxed);
            self.last_check_ms[idx].store(now_ms, Ordering::Relaxed);
        }
    }

    /// Check if backend is alive
    pub fn is_alive(&self, idx: usize) -> bool {
        idx < self.alive.len() && self.alive[idx].load(Ordering::Relaxed) == 1
    }

    /// Get last check timestamp
    pub fn last_check(&self, idx: usize) -> Option<u64> {
        if idx < self.last_check_ms.len() {
            Some(self.last_check_ms[idx].load(Ordering::Relaxed))
        } else {
            None
        }
    }

    /// Get all healthy backend indices
    pub fn healthy_indices(&self) -> Vec<usize> {
        self.alive
            .iter()
            .enumerate()
            .filter(|(_, status)| status.load(Ordering::Relaxed) == 1)
            .map(|(idx, _)| idx)
            .collect()
    }

    /// Get all unhealthy backend indices
    pub fn unhealthy_indices(&self) -> Vec<usize> {
        self.alive
            .iter()
            .enumerate()
            .filter(|(_, status)| status.load(Ordering::Relaxed) == 0)
            .map(|(idx, _)| idx)
            .collect()
    }

    /// Migrate health status when backends change
    pub fn migrate(
        &self,
        new_cap: usize,
        id_mapping: &[Option<usize>],
    ) -> HealthRegistry {
        let new_registry = HealthRegistry::new(new_cap);

        for (old_idx, new_idx_opt) in id_mapping.iter().enumerate() {
            if let Some(new_idx) = new_idx_opt
                && old_idx < self.alive.len()
                && *new_idx < new_cap
            {
                let alive = self.alive[old_idx].load(Ordering::Relaxed);
                let last_check = self.last_check_ms[old_idx].load(Ordering::Relaxed);
                new_registry.alive[*new_idx].store(alive, Ordering::Relaxed);
                new_registry.last_check_ms[*new_idx].store(last_check, Ordering::Relaxed);
            }
        }

        new_registry
    }

    /// Check if index is in bounds
    pub fn is_valid_index(&self, idx: usize) -> bool {
        idx < self.alive.len()
    }
}
