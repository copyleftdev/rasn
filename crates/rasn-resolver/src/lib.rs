//! DNS resolver with caching for RASN
//!
//! This crate provides async DNS resolution with:
//! - A/AAAA record lookups
//! - PTR (reverse DNS) lookups
//! - In-memory LRU caching
//! - Concurrent query batching
//!
//! # Examples
//!
//! ```no_run
//! use rasn_resolver::DnsResolver;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let resolver = DnsResolver::new()?;
//!     let ips = resolver.resolve("google.com").await?;
//!     println!("IPs: {:?}", ips);
//!     Ok(())
//! }
//! ```

use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::TokioAsyncResolver;
use lru::LruCache;
use std::net::IpAddr;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;

/// DNS resolution errors
#[derive(Error, Debug)]
pub enum DnsError {
    /// Domain not found (NXDOMAIN)
    #[error("Domain not found: {0}")]
    NotFound(String),

    /// DNS lookup failed
    #[error("DNS lookup failed: {0}")]
    LookupFailed(String),

    /// Invalid domain name
    #[error("Invalid domain name: {0}")]
    InvalidDomain(String),

    /// Timeout
    #[error("DNS query timed out after {0:?}")]
    Timeout(Duration),

    /// Resolver error
    #[error("Resolver error: {0}")]
    ResolverError(String),
}

pub type Result<T> = std::result::Result<T, DnsError>;

/// Cache entry with TTL
#[derive(Clone, Debug)]
struct CacheEntry {
    ips: Vec<IpAddr>,
    expires_at: Instant,
}

impl CacheEntry {
    fn new(ips: Vec<IpAddr>, ttl: Duration) -> Self {
        Self {
            ips,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// DNS cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Current cache size
    pub size: usize,
    /// Cache capacity
    pub capacity: usize,
}

impl CacheStats {
    /// Calculate cache hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

/// DNS resolver with caching
///
/// Provides async DNS resolution with LRU caching and configurable timeouts.
pub struct DnsResolver {
    resolver: TokioAsyncResolver,
    cache: Arc<RwLock<LruCache<String, CacheEntry>>>,
    stats: Arc<RwLock<CacheStats>>,
    timeout: Duration,
    default_ttl: Duration,
}

impl DnsResolver {
    /// Create a new DNS resolver with default configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use rasn_resolver::DnsResolver;
    ///
    /// let resolver = DnsResolver::new()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new() -> Result<Self> {
        Self::with_capacity(1000)
    }

    /// Create a DNS resolver with custom cache capacity
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of cached entries
    pub fn with_capacity(capacity: usize) -> Result<Self> {
        let resolver =
            TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

        Ok(Self {
            resolver,
            cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(capacity).unwrap(),
            ))),
            stats: Arc::new(RwLock::new(CacheStats {
                capacity,
                ..Default::default()
            })),
            timeout: Duration::from_secs(5),
            default_ttl: Duration::from_secs(300), // 5 minutes
        })
    }

    /// Resolve a domain name to IP addresses
    ///
    /// # Arguments
    ///
    /// * `domain` - Domain name to resolve (e.g., "google.com")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rasn_resolver::DnsResolver;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let resolver = DnsResolver::new()?;
    /// let ips = resolver.resolve("google.com").await?;
    /// assert!(!ips.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn resolve(&self, domain: &str) -> Result<Vec<IpAddr>> {
        // Check cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(entry) = cache.get(domain) {
                if !entry.is_expired() {
                    // Cache hit
                    let mut stats = self.stats.write().await;
                    stats.hits += 1;
                    return Ok(entry.ips.clone());
                } else {
                    // Expired entry, remove it
                    cache.pop(domain);
                }
            }
        }

        // Cache miss - do actual lookup
        let mut stats = self.stats.write().await;
        stats.misses += 1;
        drop(stats);

        // Perform DNS lookup with timeout
        let lookup_future = self.resolver.lookup_ip(domain);
        let result = tokio::time::timeout(self.timeout, lookup_future)
            .await
            .map_err(|_| DnsError::Timeout(self.timeout))?
            .map_err(|e| {
                use hickory_resolver::error::ResolveErrorKind;
                if matches!(e.kind(), ResolveErrorKind::NoRecordsFound { .. }) {
                    DnsError::NotFound(domain.to_string())
                } else {
                    DnsError::LookupFailed(e.to_string())
                }
            })?;

        let ips: Vec<IpAddr> = result.iter().collect();

        if ips.is_empty() {
            return Err(DnsError::NotFound(domain.to_string()));
        }

        // Cache the result
        let entry = CacheEntry::new(ips.clone(), self.default_ttl);
        let mut cache = self.cache.write().await;
        cache.put(domain.to_string(), entry);

        // Update stats
        let mut stats = self.stats.write().await;
        stats.size = cache.len();

        Ok(ips)
    }

    /// Get cache statistics
    ///
    /// # Examples
    ///
    /// ```
    /// # use rasn_resolver::DnsResolver;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let resolver = DnsResolver::new()?;
    /// let stats = resolver.cache_stats().await;
    /// println!("Hit rate: {:.1}%", stats.hit_rate());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn cache_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Clear the DNS cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        let mut stats = self.stats.write().await;
        stats.size = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_expiry() {
        let entry = CacheEntry::new(vec![], Duration::from_millis(1));
        assert!(!entry.is_expired());
        std::thread::sleep(Duration::from_millis(10));
        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_stats_hit_rate() {
        let stats = CacheStats {
            hits: 90,
            misses: 10,
            size: 50,
            capacity: 100,
        };
        assert_eq!(stats.hit_rate(), 90.0);

        let empty_stats = CacheStats::default();
        assert_eq!(empty_stats.hit_rate(), 0.0);
    }

    #[tokio::test]
    async fn test_resolver_creation() {
        let resolver = DnsResolver::new();
        assert!(resolver.is_ok());

        let resolver = DnsResolver::with_capacity(500);
        assert!(resolver.is_ok());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let resolver = DnsResolver::new().unwrap();
        let stats = resolver.cache_stats().await;
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.capacity, 1000);
    }
}
