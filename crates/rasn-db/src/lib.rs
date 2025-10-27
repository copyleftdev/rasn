//! RocksDB cold storage for overflow ASN data
//!
//! Provides persistent storage for the 0.1% of IP ranges not in Arrow tables.
//! Uses RocksDB with column families for efficient data organization.
//!
//! # Architecture
//!
//! - **ip_ranges**: Maps IP ranges to ASN numbers
//! - **asn_metadata**: Stores ASN metadata (org, country, description)
//! - **indexes**: Secondary indexes for efficient lookups
//!
//! # Compression
//!
//! - LZ4 for fast compression/decompression
//! - Snappy as fallback
//! - 256MB block cache for hot data
//!
//! # Examples
//!
//! ```no_run
//! use rasn_db::ColdStorage;
//! use rasn_core::{Asn, AsnInfo};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let storage = ColdStorage::open("./data/cold")?;
//!
//! // Store ASN info
//! let info = AsnInfo {
//!     asn: Asn(15169),
//!     organization: "Google".to_string(),
//!     country: Some("US".to_string()),
//!     description: Some("Google LLC".to_string()),
//! };
//! storage.put_asn_info(&info)?;
//!
//! // Retrieve ASN info
//! if let Some(retrieved) = storage.get_asn_info(15169)? {
//!     println!("Found: {}", retrieved.organization);
//! }
//! # Ok(())
//! # }
//! ```

use rasn_core::AsnInfo;
use rocksdb::{BlockBasedOptions, Options, DB};
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

/// Cold storage errors
#[derive(Error, Debug)]
pub enum StorageError {
    /// Database operation failed
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Serialization failed
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Data not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Invalid data format
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

impl From<rocksdb::Error> for StorageError {
    fn from(err: rocksdb::Error) -> Self {
        StorageError::DatabaseError(err.to_string())
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::SerializationError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, StorageError>;

/// Column family names
const CF_IP_RANGES: &str = "ip_ranges";
const CF_ASN_METADATA: &str = "asn_metadata";
const CF_INDEXES: &str = "indexes";

/// RocksDB cold storage for overflow ASN data
///
/// Stores the 0.1% of data not in Arrow tables with efficient compression.
pub struct ColdStorage {
    db: Arc<DB>,
}

impl ColdStorage {
    /// Open or create a cold storage database
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the database directory
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rasn_db::ColdStorage;
    ///
    /// let storage = ColdStorage::open("./data/cold")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        // Configure compression
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);

        // Configure block cache (256MB)
        let mut block_opts = BlockBasedOptions::default();
        block_opts.set_block_cache(&rocksdb::Cache::new_lru_cache(256 * 1024 * 1024));
        opts.set_block_based_table_factory(&block_opts);

        // Batch writes for better performance
        opts.set_write_buffer_size(64 * 1024 * 1024);

        // Open with column families
        let db = DB::open_cf(&opts, path, vec![CF_IP_RANGES, CF_ASN_METADATA, CF_INDEXES])?;

        Ok(Self { db: Arc::new(db) })
    }

    /// Store ASN information
    ///
    /// # Arguments
    ///
    /// * `info` - ASN information to store
    pub fn put_asn_info(&self, info: &AsnInfo) -> Result<()> {
        let cf = self.get_cf(CF_ASN_METADATA)?;
        let key = info.asn.0.to_be_bytes();
        let value = serde_json::to_vec(info)?;

        self.db.put_cf(&cf, key, value)?;
        Ok(())
    }

    /// Retrieve ASN information
    ///
    /// # Arguments
    ///
    /// * `asn` - ASN number to look up
    pub fn get_asn_info(&self, asn: u32) -> Result<Option<AsnInfo>> {
        let cf = self.get_cf(CF_ASN_METADATA)?;
        let key = asn.to_be_bytes();

        match self.db.get_cf(&cf, key)? {
            Some(bytes) => {
                let info = serde_json::from_slice(&bytes)?;
                Ok(Some(info))
            }
            None => Ok(None),
        }
    }

    /// Store IP range mapping
    ///
    /// # Arguments
    ///
    /// * `start_ip` - Start of IP range
    /// * `end_ip` - End of IP range
    /// * `asn` - ASN number for this range
    pub fn put_ip_range(&self, start_ip: u32, end_ip: u32, asn: u32) -> Result<()> {
        let cf = self.get_cf(CF_IP_RANGES)?;

        // Key: start_ip (big-endian for proper sorting)
        let key = start_ip.to_be_bytes();

        // Value: [end_ip, asn]
        let mut value = Vec::with_capacity(8);
        value.extend_from_slice(&end_ip.to_be_bytes());
        value.extend_from_slice(&asn.to_be_bytes());

        self.db.put_cf(&cf, key, value)?;
        Ok(())
    }

    /// Find IP in stored ranges
    ///
    /// # Arguments
    ///
    /// * `ip` - IP address to look up
    pub fn find_ip(&self, ip: u32) -> Result<Option<u32>> {
        let cf = self.get_cf(CF_IP_RANGES)?;
        let search_key = ip.to_be_bytes();

        // Use iterator to find range containing IP
        let mut iter = self.db.raw_iterator_cf(&cf);
        iter.seek_for_prev(&search_key);

        while iter.valid() {
            if let (Some(key), Some(value)) = (iter.key(), iter.value()) {
                if key.len() == 4 && value.len() == 8 {
                    let start_ip = u32::from_be_bytes(key.try_into().unwrap());
                    let end_ip = u32::from_be_bytes(value[0..4].try_into().unwrap());
                    let asn = u32::from_be_bytes(value[4..8].try_into().unwrap());

                    if ip >= start_ip && ip <= end_ip {
                        return Ok(Some(asn));
                    }

                    // If IP is before this range, no match
                    if ip < start_ip {
                        return Ok(None);
                    }
                }
            }
            iter.prev();
        }

        Ok(None)
    }

    /// Delete ASN information
    pub fn delete_asn_info(&self, asn: u32) -> Result<()> {
        let cf = self.get_cf(CF_ASN_METADATA)?;
        let key = asn.to_be_bytes();
        self.db.delete_cf(&cf, key)?;
        Ok(())
    }

    /// Get database statistics
    pub fn stats(&self) -> Result<String> {
        Ok(self.db.property_value("rocksdb.stats")?.unwrap_or_default())
    }

    /// Get column family handle
    fn get_cf(&self, name: &str) -> Result<&rocksdb::ColumnFamily> {
        self.db
            .cf_handle(name)
            .ok_or_else(|| StorageError::DatabaseError(format!("Column family {} not found", name)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_temp_storage() -> (ColdStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = ColdStorage::open(temp_dir.path()).unwrap();
        (storage, temp_dir)
    }

    #[test]
    fn test_storage_creation() {
        let (storage, _temp) = create_temp_storage();
        assert!(storage.stats().is_ok());
    }

    #[test]
    fn test_put_get_asn_info() {
        let (storage, _temp) = create_temp_storage();

        let info = AsnInfo {
            asn: Asn(15169),
            organization: "Google".to_string(),
            country: Some("US".to_string()),
            description: Some("Google LLC".to_string()),
        };

        storage.put_asn_info(&info).unwrap();

        let retrieved = storage.get_asn_info(15169).unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.asn.0, 15169);
        assert_eq!(retrieved.organization, "Google");
    }

    #[test]
    fn test_get_nonexistent() {
        let (storage, _temp) = create_temp_storage();
        let result = storage.get_asn_info(99999).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_put_find_ip_range() {
        let (storage, _temp) = create_temp_storage();

        // 8.8.8.0 - 8.8.8.255 -> ASN 15169
        storage.put_ip_range(0x08080800, 0x080808FF, 15169).unwrap();

        // Find IP in range
        let asn = storage.find_ip(0x08080808).unwrap(); // 8.8.8.8
        assert_eq!(asn, Some(15169));

        // Find IP outside range
        let asn = storage.find_ip(0x01010101).unwrap(); // 1.1.1.1
        assert_eq!(asn, None);
    }

    #[test]
    fn test_delete_asn_info() {
        let (storage, _temp) = create_temp_storage();

        let info = AsnInfo {
            asn: Asn(15169),
            organization: "Google".to_string(),
            country: Some("US".to_string()),
            description: None,
        };

        storage.put_asn_info(&info).unwrap();
        assert!(storage.get_asn_info(15169).unwrap().is_some());

        storage.delete_asn_info(15169).unwrap();
        assert!(storage.get_asn_info(15169).unwrap().is_none());
    }

    #[test]
    fn test_multiple_ranges() {
        let (storage, _temp) = create_temp_storage();

        // Multiple ranges
        storage.put_ip_range(0x08080800, 0x080808FF, 15169).unwrap(); // Google
        storage.put_ip_range(0x01010100, 0x010101FF, 13335).unwrap(); // Cloudflare

        assert_eq!(storage.find_ip(0x08080808).unwrap(), Some(15169));
        assert_eq!(storage.find_ip(0x01010101).unwrap(), Some(13335));
    }
}
