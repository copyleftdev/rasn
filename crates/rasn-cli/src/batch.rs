//! Parallel batch processing using Rayon
//!
//! Provides high-performance batch lookups with:
//! - Parallel DNS resolution
//! - Parallel Arrow table lookups
//! - Configurable thread pool
//! - Individual error handling
//! - Progress reporting

use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use rasn_arrow::IpRangeTableV4;
use rasn_core::AsnInfo;
use rasn_resolver::DnsResolver;

/// Number of CPU cores available
fn num_cpus_get() -> usize {
    num_cpus::get()
}

/// Batch processing result
#[derive(Debug, Clone)]
pub struct BatchResult {
    pub input: String,
    pub result: Result<AsnInfo, String>,
}

/// Batch processor with parallel execution
pub struct BatchProcessor {
    arrow_table: Option<Arc<IpRangeTableV4>>,
    dns_resolver: Option<Arc<DnsResolver>>,
    thread_pool: rayon::ThreadPool,
}

impl BatchProcessor {
    /// Create a new batch processor
    ///
    /// # Arguments
    ///
    /// * `arrow_path` - Optional path to Arrow/Parquet data
    /// * `num_threads` - Number of threads (default: CPU cores * 2)
    pub fn new(arrow_path: Option<&Path>, num_threads: Option<usize>) -> Result<Self> {
        let num_threads = num_threads.unwrap_or_else(|| num_cpus_get() * 2);
        
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()?;

        let arrow_table = if let Some(path) = arrow_path {
            Some(Arc::new(IpRangeTableV4::from_parquet(path)?))
        } else {
            None
        };

        let dns_resolver = Some(Arc::new(DnsResolver::new()?));

        Ok(Self {
            arrow_table,
            dns_resolver,
            thread_pool,
        })
    }

    /// Process a batch of IP addresses in parallel
    ///
    /// # Arguments
    ///
    /// * `ips` - Vector of IP addresses (as u32)
    pub fn process_ips(&self, ips: Vec<u32>) -> Vec<BatchResult> {
        let total = ips.len();
        let processed = Arc::new(AtomicUsize::new(0));

        self.thread_pool.install(|| {
            ips.into_par_iter()
                .map(|ip| {
                    let result = self.lookup_ip(ip);
                    
                    let count = processed.fetch_add(1, Ordering::Relaxed) + 1;
                    if count % 1000 == 0 || count == total {
                        eprintln!("Processed {}/{} IPs", count, total);
                    }

                    BatchResult {
                        input: format!("{}.{}.{}.{}", 
                            (ip >> 24) & 0xFF,
                            (ip >> 16) & 0xFF,
                            (ip >> 8) & 0xFF,
                            ip & 0xFF
                        ),
                        result,
                    }
                })
                .collect()
        })
    }

    /// Process a batch of domains in parallel (with DNS resolution)
    ///
    /// # Arguments
    ///
    /// * `domains` - Vector of domain names
    pub fn process_domains(&self, domains: Vec<String>) -> Vec<BatchResult> {
        let total = domains.len();
        let processed = Arc::new(AtomicUsize::new(0));
        let resolver = self.dns_resolver.as_ref().map(|r| r.as_ref());

        self.thread_pool.install(|| {
            domains.into_par_iter()
                .map(|domain| {
                    let result = self.lookup_domain(&domain, resolver);
                    
                    let count = processed.fetch_add(1, Ordering::Relaxed) + 1;
                    if count % 100 == 0 || count == total {
                        eprintln!("Processed {}/{} domains", count, total);
                    }

                    BatchResult {
                        input: domain,
                        result,
                    }
                })
                .collect()
        })
    }

    /// Lookup single IP in Arrow table
    fn lookup_ip(&self, ip: u32) -> Result<AsnInfo, String> {
        if let Some(ref table) = self.arrow_table {
            table
                .find_ip(ip)
                .ok_or_else(|| format!("No ASN found for IP"))
        } else {
            Err("Arrow table not loaded".to_string())
        }
    }

    /// Lookup domain (resolve DNS then lookup IP)
    fn lookup_domain(&self, domain: &str, resolver: Option<&DnsResolver>) -> Result<AsnInfo, String> {
        let resolver = resolver.ok_or_else(|| "DNS resolver not available".to_string())?;
        
        // Synchronous DNS resolution (blocking in thread pool)
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;
        
        let ips = rt.block_on(resolver.resolve(domain))
            .map_err(|e| format!("DNS resolution failed: {}", e))?;

        let ip = ips.first()
            .ok_or_else(|| "No IPs returned".to_string())?;

        // Convert IpAddr to u32 (IPv4 only for now)
        let ip_u32 = match ip {
            std::net::IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();
                u32::from_be_bytes(octets)
            }
            std::net::IpAddr::V6(_) => {
                return Err("IPv6 not yet supported".to_string());
            }
        };

        self.lookup_ip(ip_u32)
    }

    /// Get thread pool info
    pub fn thread_count(&self) -> usize {
        self.thread_pool.current_num_threads()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_processor_creation() {
        let processor = BatchProcessor::new(None, Some(4));
        assert!(processor.is_ok());
        assert_eq!(processor.unwrap().thread_count(), 4);
    }

    #[test]
    fn test_batch_processor_default_threads() {
        let processor = BatchProcessor::new(None, None).unwrap();
        assert!(processor.thread_count() > 0);
    }
}
