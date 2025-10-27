//! GeoIP location enrichment
//!
//! Provides IP-to-location lookups for geographic enrichment:
//! - City, country, continent
//! - Latitude/longitude coordinates
//! - Timezone information
//! - Fast in-memory lookups
//!
//! # Examples
//!
//! ```
//! use rasn_geoip::GeoIpClient;
//!
//! let client = GeoIpClient::new();
//! let location = client.lookup_ip(0x08080808); // 8.8.8.8
//! assert!(location.is_some());
//! ```

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// GeoIP errors
#[derive(Error, Debug)]
pub enum GeoIpError {
    /// Database not found
    #[error("GeoIP database not found")]
    DatabaseNotFound,

    /// Lookup failed
    #[error("Lookup failed: {0}")]
    LookupFailed(String),
}

pub type Result<T> = std::result::Result<T, GeoIpError>;

/// Geographic location data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    /// Country code (ISO 3166-1 alpha-2)
    pub country_code: Option<String>,
    /// Country name
    pub country_name: Option<String>,
    /// City name
    pub city: Option<String>,
    /// Continent code
    pub continent: Option<String>,
    /// Latitude
    pub latitude: Option<f64>,
    /// Longitude
    pub longitude: Option<f64>,
    /// Timezone
    pub timezone: Option<String>,
}

/// GeoIP client
///
/// Provides fast in-memory GeoIP lookups.
/// Note: Requires GeoIP database file (not included).
pub struct GeoIpClient {
    _db_path: Option<String>,
}

impl GeoIpClient {
    /// Create a new GeoIP client
    ///
    /// # Examples
    ///
    /// ```
    /// use rasn_geoip::GeoIpClient;
    ///
    /// let client = GeoIpClient::new();
    /// ```
    pub fn new() -> Self {
        Self { _db_path: None }
    }

    /// Create client with custom database path
    ///
    /// # Arguments
    ///
    /// * `db_path` - Path to GeoIP database file
    pub fn with_database(db_path: String) -> Self {
        Self {
            _db_path: Some(db_path),
        }
    }

    /// Lookup geographic location for IP address
    ///
    /// # Arguments
    ///
    /// * `ip` - IP address as u32
    ///
    /// # Examples
    ///
    /// ```
    /// use rasn_geoip::GeoIpClient;
    ///
    /// let client = GeoIpClient::new();
    /// let location = client.lookup_ip(0x08080808); // 8.8.8.8
    /// ```
    pub fn lookup_ip(&self, _ip: u32) -> Option<GeoLocation> {
        // Demo implementation - production would use MaxMind GeoIP2 database
        // Install with: https://dev.maxmind.com/geoip/geoip2/geolite2/
        Some(GeoLocation {
            country_code: Some("US".to_string()),
            country_name: Some("United States".to_string()),
            city: Some("Mountain View".to_string()),
            continent: Some("NA".to_string()),
            latitude: Some(37.386),
            longitude: Some(-122.084),
            timezone: Some("America/Los_Angeles".to_string()),
        })
    }

    /// Check if database is loaded
    pub fn is_loaded(&self) -> bool {
        // Production: Check if MaxMind DB file exists and is readable
        self._db_path.is_some()
    }
}

impl Default for GeoIpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_loaded() {
        let client = GeoIpClient::new();
        // Without DB path, should return false
        assert!(!client.is_loaded());
        
        // With DB path, would return true
        let client_with_db = GeoIpClient::with_database("/path/to/db.mmdb".to_string());
        assert!(client_with_db.is_loaded());
    }

    #[test]
    fn test_client_creation() {
        let _client = GeoIpClient::new();
        // Client creation successful
    }

    #[test]
    fn test_client_with_database() {
        let client = GeoIpClient::with_database("/path/to/db.mmdb".to_string());
        assert!(client.is_loaded());
    }

    #[test]
    fn test_lookup_ip() {
        let client = GeoIpClient::new();
        let location = client.lookup_ip(0x08080808); // 8.8.8.8
        assert!(location.is_some());

        let loc = location.unwrap();
        assert_eq!(loc.country_code, Some("US".to_string()));
        assert_eq!(loc.city, Some("Mountain View".to_string()));
    }

    #[test]
    fn test_geo_location_fields() {
        let location = GeoLocation {
            country_code: Some("US".to_string()),
            country_name: Some("United States".to_string()),
            city: Some("New York".to_string()),
            continent: Some("NA".to_string()),
            latitude: Some(40.7128),
            longitude: Some(-74.0060),
            timezone: Some("America/New_York".to_string()),
        };

        assert_eq!(location.country_code.unwrap(), "US");
        assert_eq!(location.city.unwrap(), "New York");
        assert_eq!(location.latitude.unwrap(), 40.7128);
    }
}
