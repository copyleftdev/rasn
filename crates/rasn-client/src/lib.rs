//! HTTP client for external ASN API integration
//!
//! Provides async HTTP client with:
//! - Connection pooling
//! - Automatic retries with exponential backoff
//! - Rate limiting
//! - Timeout configuration
//!
//! # Examples
//!
//! ```no_run
//! use rasn_client::ApiClient;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = ApiClient::new("your-api-key".to_string());
//! let info = client.lookup_ip("8.8.8.8").await?;
//! println!("ASN: {}", info.asn);
//! # Ok(())
//! # }
//! ```

use governor::{Quota, RateLimiter};
use rasn_core::{Asn, AsnInfo};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

/// API client errors
#[derive(Error, Debug)]
pub enum ApiError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),

    /// API returned an error
    #[error("API error: {0}")]
    ApiError(String),

    /// Invalid response format
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded, retry after {0}s")]
    RateLimited(u64),

    /// Timeout
    #[error("Request timed out after {0:?}")]
    Timeout(Duration),

    /// No data found
    #[error("No ASN data found for: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, ApiError>;

/// API response for ASN lookup
#[derive(Debug, Deserialize, Serialize)]
struct ApiResponse {
    #[serde(rename = "asn")]
    asn_number: Option<u32>,
    #[serde(default)]
    organization: String,
    #[serde(default)]
    country: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

/// HTTP client for ASN API lookups
///
/// Provides connection pooling, retries, and rate limiting for external API calls.
pub struct ApiClient {
    client: Client,
    api_key: String,
    base_url: String,
    _timeout: Duration,
    max_retries: u32,
    rate_limiter: Arc<
        RateLimiter<
            governor::state::direct::NotKeyed,
            governor::state::InMemoryState,
            governor::clock::DefaultClock,
            governor::middleware::NoOpMiddleware,
        >,
    >,
}

impl ApiClient {
    /// Create a new API client
    ///
    /// # Arguments
    ///
    /// * `api_key` - API key for authentication
    ///
    /// # Examples
    ///
    /// ```
    /// use rasn_client::ApiClient;
    ///
    /// let client = ApiClient::new("my-api-key".to_string());
    /// ```
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(10)
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();

        // Rate limiter: 100 requests per second (burst)
        let quota = Quota::per_second(NonZeroU32::new(100).unwrap());
        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        Self {
            client,
            api_key,
            base_url: "https://api.projectdiscovery.io".to_string(), // Example API endpoint
            _timeout: Duration::from_secs(10),
            max_retries: 3,
            rate_limiter,
        }
    }

    /// Create client with custom configuration
    pub fn with_config(api_key: String, base_url: String, timeout: Duration) -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(10)
            .timeout(timeout)
            .build()
            .unwrap();

        // Rate limiter: 100 requests per second (burst)
        let quota = Quota::per_second(NonZeroU32::new(100).unwrap());
        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        Self {
            client,
            api_key,
            base_url,
            _timeout: timeout,
            max_retries: 3,
            rate_limiter,
        }
    }

    /// Lookup ASN information for an IP address
    ///
    /// # Arguments
    ///
    /// * `ip` - IP address as string
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rasn_client::ApiClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ApiClient::new("api-key".to_string());
    /// let info = client.lookup_ip("8.8.8.8").await?;
    /// assert_eq!(info.asn.0, 15169);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn lookup_ip(&self, ip: &str) -> Result<AsnInfo> {
        // Wait for rate limiter
        self.rate_limiter.until_ready().await;

        let url = format!("{}/asn/{}", self.base_url, ip);

        let mut retries = 0;
        let mut backoff = Duration::from_millis(100);

        loop {
            match self.make_request(&url).await {
                Ok(response) => return self.parse_response(response, ip),
                Err(_e) if retries < self.max_retries => {
                    retries += 1;
                    tokio::time::sleep(backoff).await;
                    backoff *= 2; // Exponential backoff
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Make HTTP request with authentication
    async fn make_request(&self, url: &str) -> Result<ApiResponse> {
        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("User-Agent", "rasn/0.1.0")
            .send()
            .await
            .map_err(|e| ApiError::RequestFailed(e.to_string()))?;

        // Check for rate limiting
        if response.status() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(60);
            return Err(ApiError::RateLimited(retry_after));
        }

        // Check for other errors
        if !response.status().is_success() {
            return Err(ApiError::ApiError(format!("HTTP {}", response.status())));
        }

        response
            .json()
            .await
            .map_err(|e| ApiError::InvalidResponse(e.to_string()))
    }

    /// Parse API response into AsnInfo
    fn parse_response(&self, response: ApiResponse, query: &str) -> Result<AsnInfo> {
        let asn = response
            .asn_number
            .ok_or_else(|| ApiError::NotFound(query.to_string()))?;

        Ok(AsnInfo {
            asn: Asn(asn),
            organization: response.organization,
            country: response.country,
            description: response.description,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = ApiClient::new("test-key".to_string());
        assert_eq!(client.api_key, "test-key");
        assert_eq!(client.max_retries, 3);
    }

    #[test]
    fn test_client_with_config() {
        let client = ApiClient::with_config(
            "key".to_string(),
            "https://api.test.com".to_string(),
            Duration::from_secs(5),
        );
        assert_eq!(client.base_url, "https://api.test.com");
        assert_eq!(client._timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_parse_response() {
        let client = ApiClient::new("test".to_string());
        let response = ApiResponse {
            asn_number: Some(15169),
            organization: "Google".to_string(),
            country: Some("US".to_string()),
            description: Some("Google LLC".to_string()),
        };

        let info = client.parse_response(response, "8.8.8.8").unwrap();
        assert_eq!(info.asn.0, 15169);
        assert_eq!(info.organization, "Google");
    }

    #[test]
    fn test_parse_response_no_asn() {
        let client = ApiClient::new("test".to_string());
        let response = ApiResponse {
            asn_number: None,
            organization: "Unknown".to_string(),
            country: None,
            description: None,
        };

        let result = client.parse_response(response, "192.168.1.1");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::NotFound(_)));
    }
}
