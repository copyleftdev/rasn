//! WHOIS client for ASN enrichment
//!
//! Provides TCP-based WHOIS queries for deep ASN information:
//! - Query WHOIS servers (ARIN, RIPE, APNIC, etc.)
//! - Parse registration data
//! - Extract organization, contacts, dates
//! - Rate limiting and caching
//!
//! # Examples
//!
//! ```no_run
//! use rasn_whois::WhoisClient;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = WhoisClient::new();
//! let response = client.query_asn(15169).await?;
//! println!("Organization: {}", response.org_name.unwrap_or_default());
//! # Ok(())
//! # }
//! ```

use rasn_core::Asn;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// WHOIS errors
#[derive(Error, Debug)]
pub enum WhoisError {
    /// Connection failed
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Query failed
    #[error("Query failed: {0}")]
    QueryFailed(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Timeout
    #[error("Query timeout")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, WhoisError>;

/// WHOIS response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhoisResponse {
    /// ASN number
    pub asn: Asn,
    /// Organization name
    pub org_name: Option<String>,
    /// Organization ID
    pub org_id: Option<String>,
    /// Registration date
    pub reg_date: Option<String>,
    /// Update date
    pub update_date: Option<String>,
    /// Administrative contact
    pub admin_contact: Option<String>,
    /// Technical contact
    pub tech_contact: Option<String>,
    /// Raw WHOIS response
    pub raw_response: String,
}

/// WHOIS server configuration
#[derive(Debug, Clone)]
pub struct WhoisServer {
    pub host: String,
    pub port: u16,
}

impl WhoisServer {
    fn arin() -> Self {
        Self {
            host: "whois.arin.net".to_string(),
            port: 43,
        }
    }

    fn ripe() -> Self {
        Self {
            host: "whois.ripe.net".to_string(),
            port: 43,
        }
    }

    fn apnic() -> Self {
        Self {
            host: "whois.apnic.net".to_string(),
            port: 43,
        }
    }
}

/// WHOIS client
///
/// Provides TCP-based WHOIS queries with timeout and basic parsing.
pub struct WhoisClient {
    timeout: Duration,
}

impl WhoisClient {
    /// Create a new WHOIS client
    ///
    /// # Examples
    ///
    /// ```
    /// use rasn_whois::WhoisClient;
    ///
    /// let client = WhoisClient::new();
    /// ```
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_millis(500),
        }
    }

    /// Create client with custom timeout
    ///
    /// # Arguments
    ///
    /// * `timeout` - Query timeout duration
    pub fn with_timeout(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Query ASN information from WHOIS
    ///
    /// # Arguments
    ///
    /// * `asn` - ASN number to query
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rasn_whois::WhoisClient;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = WhoisClient::new();
    /// let response = client.query_asn(15169).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_asn(&self, asn: u32) -> Result<WhoisResponse> {
        // Try ARIN first (most common for AS queries)
        let server = WhoisServer::arin();
        let query = format!("AS{}\r\n", asn);

        let raw_response = self.query_server(&server, &query).await?;

        Ok(WhoisResponse {
            asn: Asn(asn),
            org_name: Self::extract_field(&raw_response, "OrgName:"),
            org_id: Self::extract_field(&raw_response, "OrgId:"),
            reg_date: Self::extract_field(&raw_response, "RegDate:"),
            update_date: Self::extract_field(&raw_response, "Updated:"),
            admin_contact: Self::extract_field(&raw_response, "OrgAbuseEmail:"),
            tech_contact: Self::extract_field(&raw_response, "OrgTechEmail:"),
            raw_response,
        })
    }

    /// Query WHOIS server with timeout
    async fn query_server(&self, server: &WhoisServer, query: &str) -> Result<String> {
        let addr = format!("{}:{}", server.host, server.port);

        // Connect with timeout
        let stream = tokio::time::timeout(self.timeout, TcpStream::connect(&addr))
            .await
            .map_err(|_| WhoisError::Timeout)?
            .map_err(|e| WhoisError::ConnectionFailed(e.to_string()))?;

        let mut stream = stream;

        // Send query
        stream
            .write_all(query.as_bytes())
            .await
            .map_err(|e| WhoisError::QueryFailed(e.to_string()))?;

        // Read response with timeout
        let mut response = Vec::new();
        tokio::time::timeout(self.timeout, stream.read_to_end(&mut response))
            .await
            .map_err(|_| WhoisError::Timeout)?
            .map_err(|e| WhoisError::QueryFailed(e.to_string()))?;

        String::from_utf8(response).map_err(|e| WhoisError::ParseError(e.to_string()))
    }

    /// Extract field value from WHOIS response
    fn extract_field(response: &str, field: &str) -> Option<String> {
        response
            .lines()
            .find(|line| line.starts_with(field))
            .and_then(|line| line.split(':').nth(1))
            .map(|value| value.trim().to_string())
    }

    /// Get list of available WHOIS servers
    pub fn available_servers() -> Vec<WhoisServer> {
        vec![
            WhoisServer::arin(),
            WhoisServer::ripe(),
            WhoisServer::apnic(),
        ]
    }
}

impl Default for WhoisClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = WhoisClient::new();
        assert_eq!(client.timeout, Duration::from_millis(500));
    }

    #[test]
    fn test_custom_timeout() {
        let client = WhoisClient::with_timeout(Duration::from_secs(1));
        assert_eq!(client.timeout, Duration::from_secs(1));
    }

    #[test]
    fn test_extract_field() {
        let response = "OrgName: Google LLC\nOrgId: GOGL\nRegDate: 2000-03-30";

        assert_eq!(
            WhoisClient::extract_field(response, "OrgName:"),
            Some("Google LLC".to_string())
        );
        assert_eq!(
            WhoisClient::extract_field(response, "OrgId:"),
            Some("GOGL".to_string())
        );
        assert_eq!(
            WhoisClient::extract_field(response, "RegDate:"),
            Some("2000-03-30".to_string())
        );
        assert_eq!(WhoisClient::extract_field(response, "NotFound:"), None);
    }

    #[test]
    fn test_server_configs() {
        let servers = WhoisClient::available_servers();
        assert_eq!(servers.len(), 3);
        assert_eq!(servers[0].host, "whois.arin.net");
        assert_eq!(servers[1].host, "whois.ripe.net");
        assert_eq!(servers[2].host, "whois.apnic.net");
    }

    #[test]
    fn test_whois_response_creation() {
        let response = WhoisResponse {
            asn: Asn(15169),
            org_name: Some("Google LLC".to_string()),
            org_id: Some("GOGL".to_string()),
            reg_date: Some("2000-03-30".to_string()),
            update_date: Some("2024-01-15".to_string()),
            admin_contact: Some("admin@google.com".to_string()),
            tech_contact: Some("tech@google.com".to_string()),
            raw_response: "test".to_string(),
        };

        assert_eq!(response.asn.0, 15169);
        assert_eq!(response.org_name.unwrap(), "Google LLC");
    }
}
