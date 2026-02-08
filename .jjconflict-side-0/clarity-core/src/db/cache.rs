#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Query result caching layer using Moka
//!
//! This module provides a high-performance, thread-safe caching layer for database query results.
//! It uses Moka's async cache with configurable TTL, capacity limits, and metrics tracking.

use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Cache configuration with builder pattern
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache
    pub max_capacity: u64,
    /// Time-to-live for cache entries
    pub time_to_live: Duration,
    /// Time-to-idle for cache entries
    pub time_to_idle: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: 1000,
            time_to_live: Duration::from_secs(300),
            time_to_idle: Duration::from_secs(60),
        }
    }
}

impl CacheConfig {
    /// Create a new CacheConfig with default values
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_capacity: 1000,
            time_to_live: Duration::from_secs(300),
            time_to_idle: Duration::from_secs(60),
        }
    }

    /// Set maximum capacity
    #[must_use]
    pub const fn with_max_capacity(mut self, capacity: u64) -> Self {
        self.max_capacity = capacity;
        self
    }

    /// Set time-to-live
    #[must_use]
    pub const fn with_time_to_live(mut self, duration: Duration) -> Self {
        self.time_to_live = duration;
        self
    }

    /// Set time-to-idle
    #[must_use]
    pub const fn with_time_to_idle(mut self, duration: Duration) -> Self {
        self.time_to_idle = duration;
        self
    }
}

/// Cache metrics tracking hits, misses, and hit rate
#[derive(Debug, Clone)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

/// Generic query cache wrapper around Moka
///
/// Provides thread-safe caching with metrics tracking and invalidation strategies.
pub struct QueryCache<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    inner: Arc<moka::future::Cache<K, V>>,
    hit_count: Arc<AtomicU64>,
    miss_count: Arc<AtomicU64>,
}

impl<K, V> QueryCache<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new QueryCache with the given configuration
    #[must_use]
    pub fn new(config: CacheConfig) -> Self {
        let inner = moka::future::Cache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(config.time_to_live)
            .time_to_idle(config.time_to_idle)
            .build();

        Self {
            inner: Arc::new(inner),
            hit_count: Arc::new(AtomicU64::new(0)),
            miss_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get a value from the cache
    ///
    /// Returns `None` if the key is not found or has expired.
    pub async fn get(&self, key: &K) -> Option<V> {
        let result = self.inner.get(key).await;
        match &result {
            Some(_) => {
                self.hit_count.fetch_add(1, Ordering::Relaxed);
            }
            None => {
                self.miss_count.fetch_add(1, Ordering::Relaxed);
            }
        }
        result
    }

    /// Insert a value into the cache
    pub async fn insert(&self, key: K, value: V) {
        self.inner.insert(key, value).await;
    }

    /// Invalidate a specific cache entry
    pub async fn invalidate(&self, key: &K) {
        self.inner.invalidate(key).await;
    }

    /// Invalidate all cache entries
    pub async fn invalidate_all(&self) {
        self.inner.invalidate_all();
    }

    /// Get cache metrics
    #[must_use]
    pub fn get_metrics(&self) -> CacheMetrics {
        let hits = self.hit_count.load(Ordering::Relaxed);
        let misses = self.miss_count.load(Ordering::Relaxed);
        let total = hits.saturating_add(misses);
        let hit_rate = if total > 0 {
            (hits as f64) / (total as f64)
        } else {
            0.0
        };

        CacheMetrics {
            hits,
            misses,
            hit_rate,
        }
    }

    /// Reset metrics counters
    pub fn reset_metrics(&self) {
        self.hit_count.store(0, Ordering::Relaxed);
        self.miss_count.store(0, Ordering::Relaxed);
    }

    /// Get the number of entries in the cache
    #[must_use]
    pub fn entry_count(&self) -> u64 {
        self.inner.entry_count()
    }

    /// Get the weighted size of the cache (same as entry_count for basic usage)
    #[must_use]
    pub fn weighted_size(&self) -> u64 {
        self.inner.weighted_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.max_capacity, 1000);
        assert_eq!(config.time_to_live, Duration::from_secs(300));
        assert_eq!(config.time_to_idle, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_cache_insert_and_get() {
        let cache: QueryCache<String, String> = QueryCache::new(CacheConfig::default());
        cache.insert("key1".to_string(), "value1".to_string()).await;
        let value = cache.get(&"key1".to_string()).await;
        assert_eq!(value, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_cache_metrics() {
        let cache: QueryCache<String, String> = QueryCache::new(CacheConfig::default());
        cache.insert("key1".to_string(), "value1".to_string()).await;
        cache.get(&"key1".to_string()).await;

        let metrics = cache.get_metrics();
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.misses, 0);
    }
}
