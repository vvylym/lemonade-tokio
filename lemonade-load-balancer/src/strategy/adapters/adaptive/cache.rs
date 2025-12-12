//! Adaptive strategy cache module
//!
use crate::prelude::*;

/// Cached score for a backend
#[derive(Debug, Clone)]
struct CachedScore {
    /// Computed score (lower is better)
    score: f64,
    /// Timestamp when score was computed (milliseconds)
    computed_at: u64,
    /// Metrics version at computation time
    metrics_version: u64,
    /// Connections version at computation time
    connections_version: u64,
}

/// Cache for adaptive strategy scores
pub struct AdaptiveCache {
    /// Cached scores per backend
    scores: DashMap<BackendId, CachedScore>,
    /// Current metrics version (incremented on metrics update)
    metrics_version: AtomicU64,
    /// Current connections version (incremented on connection change)
    connections_version: AtomicU64,
    /// Cache TTL in milliseconds
    ttl_ms: u64,
}

impl Default for AdaptiveCache {
    fn default() -> Self {
        use super::constants::DEFAULT_CACHE_TTL_MS;
        Self::new(DEFAULT_CACHE_TTL_MS)
    }
}

impl AdaptiveCache {
    /// Create a new adaptive cache
    pub fn new(ttl_ms: u64) -> Self {
        Self {
            scores: DashMap::new(),
            metrics_version: AtomicU64::new(0),
            connections_version: AtomicU64::new(0),
            ttl_ms,
        }
    }

    /// Get cached score for a backend if valid
    pub fn get(&self, backend_id: BackendId, now_ms: u64) -> Option<f64> {
        let cached = self.scores.get(&backend_id)?;
        let cached_value = cached.value();

        // Check if cache is still valid
        let age = now_ms.saturating_sub(cached_value.computed_at);
        if age > self.ttl_ms {
            return None; // Expired
        }

        // Check if versions match
        let current_metrics_ver = self.metrics_version.load(Ordering::Relaxed);
        let current_conn_ver = self.connections_version.load(Ordering::Relaxed);

        if cached_value.metrics_version != current_metrics_ver
            || cached_value.connections_version != current_conn_ver
        {
            return None; // Invalidated
        }

        Some(cached_value.score)
    }

    /// Store a score in cache
    pub fn put(&self, backend_id: BackendId, score: f64, now_ms: u64) {
        let metrics_ver = self.metrics_version.load(Ordering::Relaxed);
        let conn_ver = self.connections_version.load(Ordering::Relaxed);

        let cached = CachedScore {
            score,
            computed_at: now_ms,
            metrics_version: metrics_ver,
            connections_version: conn_ver,
        };

        self.scores.insert(backend_id, cached);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adaptive_cache_new_should_succeed() {
        // Given: a TTL value
        let ttl_ms = 1000u64;

        // When: creating a new AdaptiveCache
        let cache = AdaptiveCache::new(ttl_ms);

        // Then: cache is created successfully
        // Verify by using it
        assert!(cache.get(1, 0).is_none()); // No cached value yet
    }

    #[test]
    fn adaptive_cache_default_should_succeed() {
        // Given: default AdaptiveCache
        // When: creating with default
        let cache = AdaptiveCache::default();

        // Then: cache is created
        // Verify by using it
        assert!(cache.get(1, 0).is_none()); // No cached value yet
    }

    #[test]
    fn adaptive_cache_put_and_get_should_succeed() {
        // Given: an AdaptiveCache
        let cache = AdaptiveCache::new(1000);
        let backend_id = 1u8;
        let score = 10.5;
        let now_ms = 1000u64;

        // When: putting a score and getting it
        cache.put(backend_id, score, now_ms);
        let retrieved = cache.get(backend_id, now_ms);

        // Then: score is retrieved correctly
        assert_eq!(retrieved, Some(10.5));
    }

    #[test]
    fn adaptive_cache_get_expired_should_succeed() {
        // Given: an AdaptiveCache with cached score
        let cache = AdaptiveCache::new(100); // TTL of 100ms
        let backend_id = 1u8;
        cache.put(backend_id, 10.5, 1000);

        // When: getting score after TTL expires
        let retrieved = cache.get(backend_id, 1200); // 200ms later, exceeds TTL

        // Then: returns None (expired)
        assert_eq!(retrieved, None);
    }

    #[test]
    fn adaptive_cache_get_within_ttl_should_succeed() {
        // Given: an AdaptiveCache with cached score
        let cache = AdaptiveCache::new(1000); // TTL of 1000ms
        let backend_id = 1u8;
        cache.put(backend_id, 10.5, 1000);

        // When: getting score within TTL
        let retrieved = cache.get(backend_id, 1500); // 500ms later, within TTL

        // Then: score is returned
        assert_eq!(retrieved, Some(10.5));
    }

    #[test]
    fn adaptive_cache_get_non_existing_should_succeed() {
        // Given: an AdaptiveCache
        let cache = AdaptiveCache::new(1000);

        // When: getting score for non-existing backend
        let retrieved = cache.get(99, 1000);

        // Then: returns None
        assert_eq!(retrieved, None);
    }

    #[test]
    fn adaptive_cache_put_overwrite_should_succeed() {
        // Given: an AdaptiveCache with existing cached score
        let cache = AdaptiveCache::new(1000);
        let backend_id = 1u8;
        cache.put(backend_id, 10.5, 1000);

        // When: putting a new score
        cache.put(backend_id, 20.0, 2000);

        // Then: new score is stored
        let retrieved = cache.get(backend_id, 2000);
        assert_eq!(retrieved, Some(20.0));
    }

    #[test]
    fn adaptive_cache_multiple_backends_should_succeed() {
        // Given: an AdaptiveCache
        let cache = AdaptiveCache::new(1000);
        let now_ms = 1000u64;

        // When: putting scores for multiple backends
        cache.put(1, 10.5, now_ms);
        cache.put(2, 20.0, now_ms);
        cache.put(3, 30.0, now_ms);

        // Then: all scores can be retrieved
        assert_eq!(cache.get(1, now_ms), Some(10.5));
        assert_eq!(cache.get(2, now_ms), Some(20.0));
        assert_eq!(cache.get(3, now_ms), Some(30.0));
    }

    #[test]
    fn adaptive_cache_get_at_exact_ttl_boundary_should_succeed() {
        // Given: an AdaptiveCache with cached score
        let cache = AdaptiveCache::new(100); // TTL of 100ms
        let backend_id = 1u8;
        cache.put(backend_id, 10.5, 1000);

        // When: getting score exactly at TTL boundary
        let retrieved = cache.get(backend_id, 1100); // Exactly 100ms later

        // Then: returns None (age > ttl_ms, 100 > 100 is false, so it's still valid)
        // Actually, age = 100, and 100 > 100 is false, so it returns the value
        assert_eq!(retrieved, Some(10.5));
    }

    #[test]
    fn adaptive_cache_get_just_after_ttl_should_succeed() {
        // Given: an AdaptiveCache with cached score
        let cache = AdaptiveCache::new(100); // TTL of 100ms
        let backend_id = 1u8;
        cache.put(backend_id, 10.5, 1000);

        // When: getting score just after TTL expires
        let retrieved = cache.get(backend_id, 1101); // 101ms later, exceeds TTL

        // Then: returns None (expired)
        assert_eq!(retrieved, None);
    }

    #[test]
    fn adaptive_cache_get_just_before_ttl_should_succeed() {
        // Given: an AdaptiveCache with cached score
        let cache = AdaptiveCache::new(100); // TTL of 100ms
        let backend_id = 1u8;
        cache.put(backend_id, 10.5, 1000);

        // When: getting score just before TTL expires
        let retrieved = cache.get(backend_id, 1099); // 99ms later, just before TTL

        // Then: score is returned
        assert_eq!(retrieved, Some(10.5));
    }

    #[test]
    fn adaptive_cache_get_with_zero_ttl_should_succeed() {
        // Given: an AdaptiveCache with zero TTL
        let cache = AdaptiveCache::new(0);
        let backend_id = 1u8;
        cache.put(backend_id, 10.5, 1000);

        // When: getting score at same timestamp
        let retrieved = cache.get(backend_id, 1000);

        // Then: returns score (age = 0, and 0 > 0 is false, so still valid)
        assert_eq!(retrieved, Some(10.5));
    }

    #[test]
    fn adaptive_cache_get_with_zero_ttl_after_put_should_succeed() {
        // Given: an AdaptiveCache with zero TTL
        let cache = AdaptiveCache::new(0);
        let backend_id = 1u8;
        cache.put(backend_id, 10.5, 1000);

        // When: getting score after put timestamp
        let retrieved = cache.get(backend_id, 1001);

        // Then: returns None (age = 1, and 1 > 0 is true, so expired)
        assert_eq!(retrieved, None);
    }

    #[test]
    fn adaptive_cache_get_with_future_timestamp_should_succeed() {
        // Given: an AdaptiveCache with cached score
        let cache = AdaptiveCache::new(1000);
        let backend_id = 1u8;
        cache.put(backend_id, 10.5, 1000);

        // When: getting score with future timestamp (time went backwards)
        let retrieved = cache.get(backend_id, 500); // Before computed_at

        // Then: age calculation handles it (saturating_sub returns 0)
        // Age is 0, which is <= TTL, so it should return the score
        assert_eq!(retrieved, Some(10.5));
    }
}
