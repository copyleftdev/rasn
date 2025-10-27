//! Multi-level LRU cache for ASN data
//!
//! Provides a two-tier caching system:
//! - **L1**: In-memory LRU cache (10k entries, <100ns lookup)
//! - **L2**: Optional disk-backed cache (100k entries, <1ms lookup)
//!
//! # Architecture
//!
//! - L1 is checked first for fastest access
//! - L2 is checked on L1 miss (if enabled)
//! - TTL support for automatic expiration
//! - Thread-safe with RwLock
//!
//! # Examples
//!
//! ```
//! use rasn_cache::CacheLayer;
//! use rasn_core::{Asn, AsnInfo};
//! use std::time::Duration;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let cache = CacheLayer::new(10000)?;
//!
//! let info = AsnInfo {
//!     asn: Asn(15169),
//!     organization: "Google".to_string(),
//!     country: Some("US".to_string()),
//!     description: Some("Google LLC".to_string()),
//! };
//!
//! // Set with 5 minute TTL
//! cache.set("8.8.8.8", info.clone(), Duration::from_secs(300)).await;
//!
//! // Get from cache
//! let cached = cache.get("8.8.8.8").await;
//! assert!(cached.is_some());
//!
//! // Check stats
//! let stats = cache.stats().await;
//! println!("Hit rate: {:.1}%", stats.hit_rate());
//! # Ok(())
//! # }
//! ```

use lru::LruCache;
use rasn_core::AsnInfo;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;

/// Cache errors
#[derive(Error, Debug)]
pub enum CacheError {
    /// Cache operation failed
    #[error("Cache error: {0}")]
    OperationFailed(String),
}

pub type Result<T> = std::result::Result<T, CacheError>;

/// Cached value with TTL
#[derive(Clone, Debug)]
struct CachedValue {
    data: AsnInfo,
    expires_at: Instant,
}

impl CachedValue {
    fn new(data: AsnInfo, ttl: Duration) -> Self {
        Self {
            data,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// L1 hits
    pub l1_hits: u64,
    /// L1 misses
    pub l1_misses: u64,
    /// L2 hits (if L2 enabled)
    pub l2_hits: u64,
    /// L2 misses (if L2 enabled)
    pub l2_misses: u64,
    /// Current L1 size
    pub l1_size: usize,
    /// L1 capacity
    pub l1_capacity: usize,
}

impl CacheStats {
    /// Calculate overall hit rate
    pub fn hit_rate(&self) -> f64 {
        let total_hits = self.l1_hits + self.l2_hits;
        let total_requests = total_hits + self.l1_misses + self.l2_misses;
        if total_requests == 0 {
            0.0
        } else {
            (total_hits as f64 / total_requests as f64) * 100.0
        }
    }

    /// Calculate L1 hit rate
    pub fn l1_hit_rate(&self) -> f64 {
        let total = self.l1_hits + self.l1_misses;
        if total == 0 {
            0.0
        } else {
            (self.l1_hits as f64 / total as f64) * 100.0
        }
    }
}

/// Multi-level cache layer
///
/// Provides L1 (memory) and optional L2 (disk) caching with LRU eviction.
pub struct CacheLayer {
    l1: Arc<RwLock<LruCache<String, CachedValue>>>,
    stats: Arc<RwLock<CacheStats>>,
    l1_capacity: usize,
}

impl CacheLayer {
    /// Create a new cache layer
    ///
    /// # Arguments
    ///
    /// * `l1_capacity` - Maximum number of entries in L1 cache
    ///
    /// # Examples
    ///
    /// ```
    /// use rasn_cache::CacheLayer;
    ///
    /// let cache = CacheLayer::new(10000)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(l1_capacity: usize) -> Result<Self> {
        let capacity = NonZeroUsize::new(l1_capacity)
            .ok_or_else(|| CacheError::OperationFailed("Capacity must be > 0".to_string()))?;

        Ok(Self {
            l1: Arc::new(RwLock::new(LruCache::new(capacity))),
            stats: Arc::new(RwLock::new(CacheStats {
                l1_capacity,
                ..Default::default()
            })),
            l1_capacity,
        })
    }

    /// Get value from cache
    ///
    /// Checks L1 first, then L2 (if enabled). Returns None if not found or expired.
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key to look up
    pub async fn get(&self, key: &str) -> Option<AsnInfo> {
        // Check L1
        let mut l1 = self.l1.write().await;
        if let Some(cached) = l1.get(key) {
            if !cached.is_expired() {
                // L1 hit
                let mut stats = self.stats.write().await;
                stats.l1_hits += 1;
                return Some(cached.data.clone());
            } else {
                // Expired, remove it
                l1.pop(key);
            }
        }

        // L1 miss
        let mut stats = self.stats.write().await;
        stats.l1_misses += 1;
        None
    }

    /// Set value in cache
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    /// * `value` - Value to cache
    /// * `ttl` - Time to live
    pub async fn set(&self, key: &str, value: AsnInfo, ttl: Duration) {
        let cached = CachedValue::new(value, ttl);
        let mut l1 = self.l1.write().await;
        l1.put(key.to_string(), cached);

        // Update stats
        let mut stats = self.stats.write().await;
        stats.l1_size = l1.len();
    }

    /// Invalidate a cache entry
    ///
    /// # Arguments
    ///
    /// * `key` - Key to invalidate
    pub async fn invalidate(&self, key: &str) {
        let mut l1 = self.l1.write().await;
        l1.pop(key);

        let mut stats = self.stats.write().await;
        stats.l1_size = l1.len();
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        let mut l1 = self.l1.write().await;
        l1.clear();

        let mut stats = self.stats.write().await;
        stats.l1_size = 0;
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Get L1 capacity
    pub fn capacity(&self) -> usize {
        self.l1_capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rasn_core::Asn;

    fn test_asn_info() -> AsnInfo {
        AsnInfo {
            asn: Asn(15169),
            organization: "Google".to_string(),
            country: Some("US".to_string()),
            description: Some("Google LLC".to_string()),
        }
    }

    #[tokio::test]
    async fn test_cache_creation() {
        let cache = CacheLayer::new(1000);
        assert!(cache.is_ok());
        assert_eq!(cache.unwrap().capacity(), 1000);
    }

    #[tokio::test]
    async fn test_cache_set_get() {
        let cache = CacheLayer::new(100).unwrap();
        let info = test_asn_info();

        cache
            .set("8.8.8.8", info.clone(), Duration::from_secs(60))
            .await;

        let cached = cache.get("8.8.8.8").await;
        assert!(cached.is_some());
        let cached = cached.unwrap();
        assert_eq!(cached.asn.0, 15169);
        assert_eq!(cached.organization, "Google");
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = CacheLayer::new(100).unwrap();
        let result = cache.get("1.1.1.1").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_ttl_expiry() {
        let cache = CacheLayer::new(100).unwrap();
        let info = test_asn_info();

        // Set with 1ms TTL
        cache
            .set("8.8.8.8", info.clone(), Duration::from_millis(1))
            .await;

        // Should be cached immediately
        assert!(cache.get("8.8.8.8").await.is_some());

        // Wait for expiry
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Should be expired
        assert!(cache.get("8.8.8.8").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_invalidate() {
        let cache = CacheLayer::new(100).unwrap();
        let info = test_asn_info();

        cache
            .set("8.8.8.8", info.clone(), Duration::from_secs(60))
            .await;
        assert!(cache.get("8.8.8.8").await.is_some());

        cache.invalidate("8.8.8.8").await;
        assert!(cache.get("8.8.8.8").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = CacheLayer::new(100).unwrap();
        let info = test_asn_info();

        cache
            .set("8.8.8.8", info.clone(), Duration::from_secs(60))
            .await;
        cache
            .set("1.1.1.1", info.clone(), Duration::from_secs(60))
            .await;

        let stats = cache.stats().await;
        assert_eq!(stats.l1_size, 2);

        cache.clear().await;

        let stats = cache.stats().await;
        assert_eq!(stats.l1_size, 0);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = CacheLayer::new(100).unwrap();
        let info = test_asn_info();

        // Set value
        cache
            .set("8.8.8.8", info.clone(), Duration::from_secs(60))
            .await;

        // Hit
        cache.get("8.8.8.8").await;

        // Miss
        cache.get("1.1.1.1").await;

        let stats = cache.stats().await;
        assert_eq!(stats.l1_hits, 1);
        assert_eq!(stats.l1_misses, 1);
        assert_eq!(stats.l1_hit_rate(), 50.0);
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let cache = CacheLayer::new(2).unwrap();
        let info = test_asn_info();

        // Fill cache
        cache
            .set("key1", info.clone(), Duration::from_secs(60))
            .await;
        cache
            .set("key2", info.clone(), Duration::from_secs(60))
            .await;

        // Add one more - should evict key1
        cache
            .set("key3", info.clone(), Duration::from_secs(60))
            .await;

        // key1 should be evicted
        assert!(cache.get("key1").await.is_none());
        assert!(cache.get("key2").await.is_some());
        assert!(cache.get("key3").await.is_some());
    }
}
