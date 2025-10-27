//! CIDR operations and IP range queries
//!
//! Provides utilities for working with CIDR notation and IP ranges:
//! - Parse CIDR notation (e.g., "192.168.0.0/24")
//! - Generate IP iterators for ranges
//! - Check if IP is in CIDR block
//! - Range calculations
//!
//! # Examples
//!
//! ```
//! use rasn_cidr::Cidr;
//!
//! let cidr = Cidr::parse("192.168.1.0/24").unwrap();
//! assert_eq!(cidr.prefix_len(), 24);
//! assert_eq!(cidr.network(), 0xC0A80100); // 192.168.1.0
//! assert!(cidr.contains(0xC0A80101)); // 192.168.1.1
//! assert!(!cidr.contains(0xC0A80001)); // 192.168.0.1
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// CIDR errors
#[derive(Error, Debug)]
pub enum CidrError {
    /// Invalid CIDR notation
    #[error("Invalid CIDR notation: {0}")]
    InvalidNotation(String),

    /// Invalid IP address
    #[error("Invalid IP address: {0}")]
    InvalidIpAddress(String),

    /// Invalid prefix length
    #[error("Invalid prefix length: {0} (must be 0-32)")]
    InvalidPrefixLength(u8),

    /// Range too large
    #[error("CIDR range too large: /{0} (use smaller prefix)")]
    RangeTooLarge(u8),
}

pub type Result<T> = std::result::Result<T, CidrError>;

/// CIDR block representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cidr {
    /// Network address (base IP)
    network: u32,
    /// Prefix length (0-32)
    prefix_len: u8,
    /// Network mask
    mask: u32,
}

impl Cidr {
    /// Parse CIDR notation string
    ///
    /// # Arguments
    ///
    /// * `cidr` - CIDR string (e.g., "192.168.1.0/24")
    ///
    /// # Examples
    ///
    /// ```
    /// use rasn_cidr::Cidr;
    ///
    /// let cidr = Cidr::parse("10.0.0.0/8").unwrap();
    /// assert_eq!(cidr.prefix_len(), 8);
    /// ```
    pub fn parse(cidr: &str) -> Result<Self> {
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            return Err(CidrError::InvalidNotation(
                "Expected format: x.x.x.x/prefix".to_string(),
            ));
        }

        let ip_str = parts[0];
        let prefix_str = parts[1];

        // Parse prefix length
        let prefix_len: u8 = prefix_str
            .parse()
            .map_err(|_| CidrError::InvalidNotation(format!("Invalid prefix: {}", prefix_str)))?;

        if prefix_len > 32 {
            return Err(CidrError::InvalidPrefixLength(prefix_len));
        }

        // Parse IP address
        let ip = Self::parse_ipv4(ip_str)?;

        // Calculate mask
        let mask = if prefix_len == 0 {
            0
        } else {
            !((1u64 << (32 - prefix_len)) - 1) as u32
        };

        // Apply mask to get network address
        let network = ip & mask;

        Ok(Self {
            network,
            prefix_len,
            mask,
        })
    }

    /// Parse IPv4 address string to u32
    fn parse_ipv4(ip: &str) -> Result<u32> {
        let octets: Vec<&str> = ip.split('.').collect();
        if octets.len() != 4 {
            return Err(CidrError::InvalidIpAddress("Expected 4 octets".to_string()));
        }

        let mut result = 0u32;
        for (i, octet_str) in octets.iter().enumerate() {
            let octet: u8 = octet_str.parse().map_err(|_| {
                CidrError::InvalidIpAddress(format!("Invalid octet: {}", octet_str))
            })?;
            result |= (octet as u32) << (24 - i * 8);
        }

        Ok(result)
    }

    /// Create new CIDR from network address and prefix length
    ///
    /// # Arguments
    ///
    /// * `network` - Network address as u32
    /// * `prefix_len` - Prefix length (0-32)
    pub fn new(network: u32, prefix_len: u8) -> Result<Self> {
        if prefix_len > 32 {
            return Err(CidrError::InvalidPrefixLength(prefix_len));
        }

        let mask = if prefix_len == 0 {
            0
        } else {
            !((1u64 << (32 - prefix_len)) - 1) as u32
        };

        Ok(Self {
            network: network & mask,
            prefix_len,
            mask,
        })
    }

    /// Get network address
    pub fn network(&self) -> u32 {
        self.network
    }

    /// Get prefix length
    pub fn prefix_len(&self) -> u8 {
        self.prefix_len
    }

    /// Get network mask
    pub fn mask(&self) -> u32 {
        self.mask
    }

    /// Get broadcast address
    pub fn broadcast(&self) -> u32 {
        self.network | !self.mask
    }

    /// Get first usable IP (network + 1)
    pub fn first_usable(&self) -> u32 {
        if self.prefix_len >= 31 {
            self.network
        } else {
            self.network + 1
        }
    }

    /// Get last usable IP (broadcast - 1)
    pub fn last_usable(&self) -> u32 {
        if self.prefix_len >= 31 {
            self.broadcast()
        } else {
            self.broadcast() - 1
        }
    }

    /// Get total number of IPs in this CIDR block
    pub fn size(&self) -> u64 {
        if self.prefix_len == 0 {
            1u64 << 32
        } else {
            1u64 << (32 - self.prefix_len)
        }
    }

    /// Check if IP address is in this CIDR block
    ///
    /// # Arguments
    ///
    /// * `ip` - IP address as u32
    pub fn contains(&self, ip: u32) -> bool {
        (ip & self.mask) == self.network
    }

    /// Get iterator over all IPs in this CIDR block
    ///
    /// Note: For large blocks (e.g., /8), this may be very slow.
    /// Consider using contains() or checking ranges instead.
    pub fn iter(&self) -> CidrIterator {
        CidrIterator::new(*self)
    }
}

impl fmt::Display for Cidr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}/{}",
            (self.network >> 24) & 0xFF,
            (self.network >> 16) & 0xFF,
            (self.network >> 8) & 0xFF,
            self.network & 0xFF,
            self.prefix_len
        )
    }
}

/// Iterator over IPs in a CIDR block
pub struct CidrIterator {
    current: u64,
    end: u64,
}

impl CidrIterator {
    fn new(cidr: Cidr) -> Self {
        let start = cidr.network() as u64;
        let end = cidr.broadcast() as u64;

        Self {
            current: start,
            end,
        }
    }
}

impl Iterator for CidrIterator {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current <= self.end {
            let ip = self.current as u32;
            self.current += 1;
            Some(ip)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cidr() {
        let cidr = Cidr::parse("192.168.1.0/24").unwrap();
        assert_eq!(cidr.network(), 0xC0A80100);
        assert_eq!(cidr.prefix_len(), 24);
    }

    #[test]
    fn test_parse_cidr_slash_8() {
        let cidr = Cidr::parse("10.0.0.0/8").unwrap();
        assert_eq!(cidr.network(), 0x0A000000);
        assert_eq!(cidr.prefix_len(), 8);
    }

    #[test]
    fn test_parse_invalid_cidr() {
        assert!(Cidr::parse("192.168.1.0").is_err());
        assert!(Cidr::parse("192.168.1.0/33").is_err());
        assert!(Cidr::parse("256.0.0.0/24").is_err());
    }

    #[test]
    fn test_cidr_contains() {
        let cidr = Cidr::parse("192.168.1.0/24").unwrap();
        assert!(cidr.contains(0xC0A80100)); // 192.168.1.0
        assert!(cidr.contains(0xC0A80101)); // 192.168.1.1
        assert!(cidr.contains(0xC0A801FF)); // 192.168.1.255
        assert!(!cidr.contains(0xC0A80001)); // 192.168.0.1
        assert!(!cidr.contains(0xC0A80200)); // 192.168.2.0
    }

    #[test]
    fn test_cidr_broadcast() {
        let cidr = Cidr::parse("192.168.1.0/24").unwrap();
        assert_eq!(cidr.broadcast(), 0xC0A801FF); // 192.168.1.255
    }

    #[test]
    fn test_cidr_usable_range() {
        let cidr = Cidr::parse("192.168.1.0/24").unwrap();
        assert_eq!(cidr.first_usable(), 0xC0A80101); // 192.168.1.1
        assert_eq!(cidr.last_usable(), 0xC0A801FE); // 192.168.1.254
    }

    #[test]
    fn test_cidr_size() {
        let cidr24 = Cidr::parse("192.168.1.0/24").unwrap();
        assert_eq!(cidr24.size(), 256);

        let cidr16 = Cidr::parse("192.168.0.0/16").unwrap();
        assert_eq!(cidr16.size(), 65536);

        let cidr8 = Cidr::parse("10.0.0.0/8").unwrap();
        assert_eq!(cidr8.size(), 16777216);
    }

    #[test]
    fn test_cidr_iterator_small() {
        let cidr = Cidr::parse("192.168.1.0/30").unwrap(); // 4 IPs
        let ips: Vec<u32> = cidr.iter().collect();
        assert_eq!(ips.len(), 4);
        assert_eq!(ips[0], 0xC0A80100); // 192.168.1.0
        assert_eq!(ips[3], 0xC0A80103); // 192.168.1.3
    }

    #[test]
    fn test_cidr_display() {
        let cidr = Cidr::parse("192.168.1.0/24").unwrap();
        assert_eq!(cidr.to_string(), "192.168.1.0/24");
    }

    #[test]
    fn test_cidr_new() {
        let cidr = Cidr::new(0xC0A80100, 24).unwrap();
        assert_eq!(cidr.network(), 0xC0A80100);
        assert_eq!(cidr.prefix_len(), 24);
    }

    #[test]
    fn test_cidr_slash_32() {
        let cidr = Cidr::parse("192.168.1.1/32").unwrap();
        assert_eq!(cidr.size(), 1);
        assert_eq!(cidr.first_usable(), 0xC0A80101);
        assert_eq!(cidr.last_usable(), 0xC0A80101);
    }
}
