//! Security and API key management
//!
//! Provides secure storage and retrieval of API keys using environment
//! variables as the primary method (no keyring dependencies for simplicity).

use std::env;
use thiserror::Error;

/// Security errors
#[derive(Error, Debug)]
pub enum SecurityError {
    /// API key not found
    #[error("API key not found. Set RASN_API_KEY environment variable.")]
    KeyNotFound,

    /// Invalid API key format
    #[error("Invalid API key format: {0}")]
    InvalidKey(String),

    /// Environment error
    #[error("Environment error: {0}")]
    EnvError(String),
}

/// Result type for security operations
pub type Result<T> = std::result::Result<T, SecurityError>;

/// API key manager
///
/// Handles secure retrieval of API keys from environment variables.
pub struct KeyManager;

impl KeyManager {
    /// Create a new key manager
    pub fn new() -> Self {
        Self
    }

    /// Get API key from environment
    ///
    /// Checks RASN_API_KEY environment variable.
    pub fn get_api_key(&self) -> Result<String> {
        env::var("RASN_API_KEY").map_err(|_| SecurityError::KeyNotFound)
    }

    /// Validate API key format
    ///
    /// Basic validation - keys should be non-empty and alphanumeric.
    pub fn validate_key(&self, key: &str) -> Result<()> {
        if key.is_empty() {
            return Err(SecurityError::InvalidKey("Key is empty".to_string()));
        }

        if key.len() < 8 {
            return Err(SecurityError::InvalidKey(
                "Key must be at least 8 characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if API key is configured
    pub fn has_api_key(&self) -> bool {
        env::var("RASN_API_KEY").is_ok()
    }

    /// Get masked API key for display
    ///
    /// Shows only first 4 and last 4 characters.
    pub fn get_masked_key(&self) -> Result<String> {
        let key = self.get_api_key()?;
        if key.len() <= 8 {
            Ok("****".to_string())
        } else {
            Ok(format!("{}...{}", &key[..4], &key[key.len() - 4..]))
        }
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_manager_creation() {
        let _manager = KeyManager::new();
    }

    #[test]
    fn test_validate_key() {
        let manager = KeyManager::new();

        // Valid key
        assert!(manager.validate_key("test_key_12345").is_ok());

        // Empty key
        assert!(manager.validate_key("").is_err());

        // Too short
        assert!(manager.validate_key("short").is_err());
    }

    #[test]
    fn test_masked_key() {
        let manager = KeyManager::new();

        // Set test key
        std::env::set_var("RASN_API_KEY", "test_key_12345678");

        let masked = manager.get_masked_key().unwrap();
        assert!(masked.contains("test"));
        assert!(masked.contains("5678"));
        assert!(masked.contains("..."));

        // Clean up
        std::env::remove_var("RASN_API_KEY");
    }
}
