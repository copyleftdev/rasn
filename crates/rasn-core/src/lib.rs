//! Core types and traits for RASN (Rust ASN Mapper)
//!
//! This crate provides the foundational types used throughout the RASN ecosystem:
//! - [`Asn`] - Autonomous System Number
//! - [`AsnInfo`] - Complete ASN information
//! - [`RasnError`] - Error types
//!
//!
//! ```
//! use rasn_core::{Asn, AsnInfo};
//!
//! let asn = Asn(15169);
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

pub mod security;

/// Autonomous System Number (ASN)
///
/// A unique identifier for an autonomous system on the internet.
/// Valid range: 0 to 4,294,967,295 (u32)
///
/// # Examples
///
/// ```
/// use rasn_core::Asn;
///
/// let google = Asn(15169);
/// let cloudflare = Asn(13335);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Asn(pub u32);

impl fmt::Display for Asn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AS{}", self.0)
    }
}

impl From<u32> for Asn {
    fn from(value: u32) -> Self {
        Asn(value)
    }
}

/// Complete information about an Autonomous System
///
/// Contains all metadata associated with an ASN including
/// organization name, country, and description.
///
/// # Examples
///
/// ```
/// use rasn_core::{Asn, AsnInfo};
///
/// let info = AsnInfo {
///     asn: Asn(15169),
///     organization: "Google LLC".to_string(),
///     country: Some("US".to_string()),
///     description: Some("Google".to_string()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AsnInfo {
    /// The ASN number
    pub asn: Asn,
    /// Organization name
    pub organization: String,
    /// ISO country code (e.g., "US", "GB")
    pub country: Option<String>,
    /// Human-readable description
    pub description: Option<String>,
}

/// Error types for RASN operations
#[derive(Error, Debug)]
pub enum RasnError {
    /// Invalid ASN number
    #[error("Invalid ASN: {0}")]
    InvalidAsn(String),

    /// Invalid IP address
    #[error("Invalid IP address: {0}")]
    InvalidIp(String),

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Parse error
    #[error("Parse error: {0}")]
    Parse(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

/// Result type alias for RASN operations
pub type Result<T> = std::result::Result<T, RasnError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asn_creation() {
        let asn = Asn(15169);
        assert_eq!(asn.0, 15169);
    }

    #[test]
    fn test_asn_display() {
        let asn = Asn(15169);
        assert_eq!(format!("{}", asn), "AS15169");
    }

    #[test]
    fn test_asn_from_u32() {
        let asn: Asn = 15169.into();
        assert_eq!(asn, Asn(15169));
    }

    #[test]
    fn test_asn_equality() {
        let asn1 = Asn(15169);
        let asn2 = Asn(15169);
        let asn3 = Asn(13335);

        assert_eq!(asn1, asn2);
        assert_ne!(asn1, asn3);
    }

    #[test]
    fn test_asn_ordering() {
        let asn1 = Asn(100);
        let asn2 = Asn(200);

        assert!(asn1 < asn2);
        assert!(asn2 > asn1);
    }

    #[test]
    fn test_asn_info_creation() {
        let info = AsnInfo {
            asn: Asn(15169),
            organization: "Google LLC".to_string(),
            country: Some("US".to_string()),
            description: Some("Google".to_string()),
        };

        assert_eq!(info.asn, Asn(15169));
        assert_eq!(info.organization, "Google LLC");
        assert_eq!(info.country, Some("US".to_string()));
    }

    #[test]
    fn test_asn_info_serialization() {
        let info = AsnInfo {
            asn: Asn(15169),
            organization: "Google LLC".to_string(),
            country: Some("US".to_string()),
            description: None,
        };

        let json = serde_json::to_string(&info).expect("serialization failed");
        assert!(json.contains("15169"));
        assert!(json.contains("Google LLC"));
    }

    #[test]
    fn test_asn_info_deserialization() {
        let json = r#"{"asn":15169,"organization":"Google LLC","country":"US","description":null}"#;
        let info: AsnInfo = serde_json::from_str(json).expect("deserialization failed");

        assert_eq!(info.asn, Asn(15169));
        assert_eq!(info.organization, "Google LLC");
        assert_eq!(info.country, Some("US".to_string()));
        assert_eq!(info.description, None);
    }

    #[test]
    fn test_error_display() {
        let err = RasnError::InvalidAsn("12345X".to_string());
        assert_eq!(format!("{}", err), "Invalid ASN: 12345X");

        let err = RasnError::NotFound("AS99999".to_string());
        assert_eq!(format!("{}", err), "Not found: AS99999");
    }

    #[test]
    fn test_result_type() {
        fn returns_result() -> Result<Asn> {
            Ok(Asn(15169))
        }

        let result = returns_result();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Asn(15169));
    }
}
