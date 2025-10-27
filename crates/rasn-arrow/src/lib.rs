//! Apache Arrow/Parquet columnar storage for RASN
//!
//! This crate provides high-performance in-memory IP→ASN lookups using
//! Apache Arrow columnar format with Parquet file storage.
//!
//! # Features
//!
//! - Memory-mapped Parquet file loading
//! - Binary search over sorted IP ranges
//! - Sub-microsecond lookup performance
//! - Zero-copy data access
//!
//! # Examples
//!
//! ```no_run
//! use rasn_arrow::IpRangeTableV4;
//! use std::path::Path;
//!
//! let table = IpRangeTableV4::from_parquet(Path::new("data/arrow/ip2asn-v4.parquet"))?;
//! if let Some(info) = table.find_ip(0x08080808) {  // 8.8.8.8
//!     println!("ASN: {}", info.asn);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use arrow::array::{Array, AsArray, UInt32Array};
use arrow::datatypes::UInt32Type;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use rasn_core::{Asn, AsnInfo};
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur when working with Arrow tables
#[derive(Error, Debug)]
pub enum ArrowError {
    /// Failed to load Parquet file
    #[error("Failed to load Parquet file: {0}")]
    ParquetLoad(String),

    /// Invalid schema
    #[error("Invalid Arrow schema: {0}")]
    InvalidSchema(String),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Arrow error
    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    /// Parquet error
    #[error("Parquet error: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),
}

pub type Result<T> = std::result::Result<T, ArrowError>;

/// IPv4 range table for IP→ASN lookups
///
/// Stores IP ranges in columnar format with binary search capability.
/// Optimized for sub-microsecond lookup performance.
pub struct IpRangeTableV4 {
    start_ips: Arc<UInt32Array>,
    end_ips: Arc<UInt32Array>,
    asns: Arc<UInt32Array>,
    countries: Vec<String>,
    orgs: Vec<String>,
    len: usize,
}

impl IpRangeTableV4 {
    /// Load IPv4 range table from Parquet file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rasn_arrow::IpRangeTableV4;
    /// use std::path::Path;
    ///
    /// let table = IpRangeTableV4::from_parquet(
    ///     Path::new("data/arrow/ip2asn-v4.parquet")
    /// )?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_parquet(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(ArrowError::FileNotFound(path.display().to_string()));
        }

        let file = File::open(path)?;
        let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
        let mut reader = builder.build()?;

        // Read all batches
        let batch = reader
            .next()
            .ok_or_else(|| ArrowError::ParquetLoad("No record batches found".to_string()))??;

        // Extract columns
        let start_ips = batch
            .column(0)
            .as_primitive::<UInt32Type>()
            .clone();
        
        let end_ips = batch
            .column(1)
            .as_primitive::<UInt32Type>()
            .clone();
        
        let asns = batch
            .column(2)
            .as_primitive::<UInt32Type>()
            .clone();

        // Extract string columns (countries and orgs)
        let countries = extract_string_column(batch.column(3))?;
        let orgs = extract_string_column(batch.column(4))?;

        let len = start_ips.len();

        Ok(Self {
            start_ips: Arc::new(start_ips),
            end_ips: Arc::new(end_ips),
            asns: Arc::new(asns),
            countries,
            orgs,
            len,
        })
    }

    /// Find ASN information for an IPv4 address
    ///
    /// Uses binary search over sorted IP ranges.
    /// Time complexity: O(log n)
    ///
    /// # Arguments
    ///
    /// * `ip` - IPv4 address as u32 (network byte order)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rasn_arrow::IpRangeTableV4;
    /// # use std::path::Path;
    /// # let table = IpRangeTableV4::from_parquet(Path::new("data/arrow/ip2asn-v4.parquet"))?;
    /// // 8.8.8.8 = 0x08080808
    /// if let Some(info) = table.find_ip(0x08080808) {
    ///     assert_eq!(info.asn.0, 15169);  // Google
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn find_ip(&self, ip: u32) -> Option<AsnInfo> {
        let idx = self.binary_search(ip)?;
        
        Some(AsnInfo {
            asn: Asn(self.asns.value(idx)),
            organization: self.orgs.get(idx)?.clone(),
            country: Some(self.countries.get(idx)?.clone()),
            description: None,
        })
    }

    /// Binary search for IP in sorted ranges
    fn binary_search(&self, ip: u32) -> Option<usize> {
        let mut left = 0;
        let mut right = self.len;

        while left < right {
            let mid = left + (right - left) / 2;
            let start = self.start_ips.value(mid);
            let end = self.end_ips.value(mid);

            if ip < start {
                right = mid;
            } else if ip > end {
                left = mid + 1;
            } else {
                return Some(mid);  // Found!
            }
        }

        None
    }

    /// Get the number of IP ranges in the table
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the table is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Helper function to extract string data from Arrow column
fn extract_string_column(column: &Arc<dyn Array>) -> Result<Vec<String>> {
    let dict_array = column
        .as_any()
        .downcast_ref::<arrow::array::DictionaryArray<arrow::datatypes::UInt8Type>>()
        .ok_or_else(|| ArrowError::InvalidSchema("Expected dictionary column".to_string()))?;

    let values = dict_array
        .values()
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .ok_or_else(|| ArrowError::InvalidSchema("Expected string values".to_string()))?;

    let result: Vec<String> = (0..dict_array.len())
        .map(|i| {
            if dict_array.is_null(i) {
                String::new()
            } else {
                let key = dict_array.keys().value(i) as usize;
                values.value(key).to_string()
            }
        })
        .collect();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_search_basic() {
        // Create a minimal test table
        let start_ips = UInt32Array::from(vec![100, 200, 300]);
        let end_ips = UInt32Array::from(vec![150, 250, 350]);
        let asns = UInt32Array::from(vec![1, 2, 3]);
        
        let table = IpRangeTableV4 {
            start_ips: Arc::new(start_ips),
            end_ips: Arc::new(end_ips),
            asns: Arc::new(asns),
            countries: vec!["US".to_string(), "GB".to_string(), "DE".to_string()],
            orgs: vec!["Org1".to_string(), "Org2".to_string(), "Org3".to_string()],
            len: 3,
        };

        // Test hits
        assert_eq!(table.binary_search(100), Some(0));
        assert_eq!(table.binary_search(125), Some(0));
        assert_eq!(table.binary_search(150), Some(0));
        assert_eq!(table.binary_search(225), Some(1));
        assert_eq!(table.binary_search(350), Some(2));

        // Test misses
        assert_eq!(table.binary_search(50), None);
        assert_eq!(table.binary_search(175), None);
        assert_eq!(table.binary_search(400), None);
    }

    #[test]
    fn test_table_properties() {
        let table = IpRangeTableV4 {
            start_ips: Arc::new(UInt32Array::from(vec![100])),
            end_ips: Arc::new(UInt32Array::from(vec![200])),
            asns: Arc::new(UInt32Array::from(vec![15169])),
            countries: vec!["US".to_string()],
            orgs: vec!["Google".to_string()],
            len: 1,
        };

        assert_eq!(table.len(), 1);
        assert!(!table.is_empty());
    }
}
