//! Model Context Protocol (MCP) JSON-RPC 2.0 Server
//!
//! Implements the Model Context Protocol with JSON-RPC 2.0 for ASN lookups.
//! Supports STDIO transport for IDE integrations.
//!
//! # Features
//!
//! - JSON-RPC 2.0 compliant request/response handling
//! - Method routing (lookup_ip, lookup_asn, etc.)
//! - Error handling per JSON-RPC spec
//! - Concurrent request handling
//! - Integration with Arrow tables and cache
//!
//! # Examples
//!
//! ```
//! use rasn_mcp::McpServer;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let server = McpServer::new(Some(Path::new("data.parquet")))?;
//!
//! let request = r#"{
//!     "jsonrpc": "2.0",
//!     "method": "lookup_ip",
//!     "params": {"ip": "8.8.8.8"},
//!     "id": 1
//! }"#;
//!
//! let response = server.handle_request(request).await?;
//! println!("Response: {}", response);
//! # Ok(())
//! # }
//! ```

use rasn_arrow::IpRangeTableV4;
use rasn_cache::CacheLayer;
use rasn_cidr::Cidr;
use rasn_core::AsnInfo;
use rasn_resolver::DnsResolver;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

/// MCP server errors
#[derive(Error, Debug)]
pub enum McpError {
    /// JSON-RPC parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Method not found
    #[error("Method not found: {0}")]
    MethodNotFound(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Arrow table error
    #[error("Arrow table error: {0}")]
    ArrowError(String),
}

pub type Result<T> = std::result::Result<T, McpError>;

/// JSON-RPC 2.0 request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    fn parse_error(msg: &str) -> Self {
        Self {
            code: -32700,
            message: msg.to_string(),
            data: None,
        }
    }

    fn invalid_request(msg: &str) -> Self {
        Self {
            code: -32600,
            message: msg.to_string(),
            data: None,
        }
    }

    fn method_not_found(msg: &str) -> Self {
        Self {
            code: -32601,
            message: msg.to_string(),
            data: None,
        }
    }

    fn internal_error(msg: &str) -> Self {
        Self {
            code: -32603,
            message: msg.to_string(),
            data: None,
        }
    }
}

/// Lookup IP request parameters
#[derive(Debug, Deserialize)]
struct LookupIpParams {
    ip: String,
}

/// Lookup ASN request parameters
#[derive(Debug, Deserialize)]
struct LookupAsnParams {
    asn: u32,
}

/// Lookup domain request parameters
#[derive(Debug, Deserialize)]
struct LookupDomainParams {
    domain: String,
}

/// Bulk lookup request parameters
#[derive(Debug, Deserialize)]
struct BulkLookupParams {
    ips: Vec<String>,
}

/// CIDR analyze request parameters
#[derive(Debug, Deserialize)]
struct CidrAnalyzeParams {
    cidr: String,
}

/// Model Context Protocol Server
///
/// Handles JSON-RPC 2.0 requests for ASN lookups.
pub struct McpServer {
    arrow_table: Option<Arc<IpRangeTableV4>>,
    cache: Arc<CacheLayer>,
    resolver: Option<Arc<DnsResolver>>,
}

impl McpServer {
    /// Create a new MCP server
    ///
    /// # Arguments
    ///
    /// * `arrow_path` - Optional path to Arrow/Parquet data
    pub fn new(arrow_path: Option<&Path>) -> Result<Self> {
        let arrow_table = if let Some(path) = arrow_path {
            Some(Arc::new(
                IpRangeTableV4::from_parquet(path)
                    .map_err(|e| McpError::ArrowError(e.to_string()))?,
            ))
        } else {
            None
        };

        let cache =
            Arc::new(CacheLayer::new(10000).map_err(|e| McpError::InternalError(e.to_string()))?);

        Ok(Self { arrow_table, cache })
    }

    /// Handle a JSON-RPC 2.0 request
    ///
    /// # Arguments
    ///
    /// * `request_str` - JSON-RPC request as string
    pub async fn handle_request(&self, request_str: &str) -> Result<String> {
        // Parse JSON-RPC request
        let request: JsonRpcRequest =
            serde_json::from_str(request_str).map_err(|e| McpError::ParseError(e.to_string()))?;

        // Validate JSON-RPC version
        if request.jsonrpc != "2.0" {
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError::invalid_request("Invalid JSON-RPC version")),
                id: request.id,
            };
            return serde_json::to_string(&response)
                .map_err(|e| McpError::InternalError(e.to_string()));
        }

        // Route to handler
        let result = match request.method.as_str() {
            "lookup_ip" => self.handle_lookup_ip(&request.params).await,
            "lookup_asn" => self.handle_lookup_asn(&request.params).await,
            "lookup_domain" => self.handle_lookup_domain(&request.params).await,
            "bulk_lookup" => self.handle_bulk_lookup(&request.params).await,
            "cidr_analyze" => self.handle_cidr_analyze(&request.params).await,
            "reverse_lookup" => self.handle_reverse_lookup(&request.params).await,
            "enrich_data" => self.handle_enrich_data(&request.params).await,
            "ping" => Ok(serde_json::json!({"status": "ok"})),
            _ => Err(McpError::MethodNotFound(request.method.clone())),
        };

        // Build response
        let response = match result {
            Ok(data) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(data),
                error: None,
                id: request.id,
            },
            Err(e) => {
                let error = match e {
                    McpError::ParseError(msg) => JsonRpcError::parse_error(&msg),
                    McpError::InvalidRequest(msg) => JsonRpcError::invalid_request(&msg),
                    McpError::MethodNotFound(msg) => JsonRpcError::method_not_found(&msg),
                    McpError::InternalError(msg) => JsonRpcError::internal_error(&msg),
                    McpError::ArrowError(msg) => JsonRpcError::internal_error(&msg),
                };
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(error),
                    id: request.id,
                }
            }
        };

        serde_json::to_string(&response).map_err(|e| McpError::InternalError(e.to_string()))
    }

    /// Handle lookup_ip method
    async fn handle_lookup_ip(&self, params: &serde_json::Value) -> Result<serde_json::Value> {
        let params: LookupIpParams = serde_json::from_value(params.clone())
            .map_err(|e| McpError::InvalidRequest(format!("Invalid params: {}", e)))?;

        // Check cache first
        if let Some(cached) = self.cache.get(&params.ip).await {
            return serde_json::to_value(&cached)
                .map_err(|e| McpError::InternalError(e.to_string()));
        }

        // Parse IP to u32
        let ip_u32 = self
            .parse_ip(&params.ip)
            .map_err(McpError::InvalidRequest)?;

        // Lookup in Arrow table
        if let Some(ref table) = self.arrow_table {
            if let Some(info) = table.find_ip(ip_u32) {
                // Cache result
                self.cache
                    .set(&params.ip, info.clone(), Duration::from_secs(300))
                    .await;

                return serde_json::to_value(&info)
                    .map_err(|e| McpError::InternalError(e.to_string()));
            }
        }

        Err(McpError::InternalError("No ASN found".to_string()))
    }

    /// Handle lookup_asn method
    async fn handle_lookup_asn(&self, params: &serde_json::Value) -> Result<serde_json::Value> {
        let params: LookupAsnParams = serde_json::from_value(params.clone())
            .map_err(|e| McpError::InvalidRequest(format!("Invalid params: {}", e)))?;

        // Search Arrow table for ASN
        if let Some(ref table) = self.arrow_table {
            // Linear search for now - could optimize with index
            for i in 0..1000 {
                if let Some(info) = table.find_ip(i) {
                    if info.asn.0 == params.asn {
                        return serde_json::to_value(&info)
                            .map_err(|e| McpError::InternalError(e.to_string()));
                    }
                }
            }
        }

        Err(McpError::InternalError("ASN not found".to_string()))
    }

    /// Handle lookup_domain method
    async fn handle_lookup_domain(&self, params: &serde_json::Value) -> Result<serde_json::Value> {
        let params: LookupDomainParams = serde_json::from_value(params.clone())
            .map_err(|e| McpError::InvalidRequest(format!("Invalid params: {}", e)))?;

        // Resolve domain to IP
        if let Some(ref resolver) = self.resolver {
            let ips = resolver
                .resolve(&params.domain)
                .await
                .map_err(|e| McpError::InternalError(e.to_string()))?;

            if let Some(ip_addr) = ips.first() {
                let ip_u32 = match ip_addr {
                    std::net::IpAddr::V4(ipv4) => {
                        let octets = ipv4.octets();
                        u32::from_be_bytes(octets)
                    }
                    std::net::IpAddr::V6(_) => {
                        return Err(McpError::InternalError("IPv6 not supported".to_string()));
                    }
                };

                // Lookup IP in Arrow table
                if let Some(ref table) = self.arrow_table {
                    if let Some(info) = table.find_ip(ip_u32) {
                        return serde_json::to_value(&serde_json::json!({
                            "domain": params.domain,
                            "ip": format!("{}", ip_addr),
                            "asn_info": info
                        }))
                        .map_err(|e| McpError::InternalError(e.to_string()));
                    }
                }
            }
        }

        Err(McpError::InternalError("Domain lookup failed".to_string()))
    }

    /// Handle bulk_lookup method
    async fn handle_bulk_lookup(&self, params: &serde_json::Value) -> Result<serde_json::Value> {
        let params: BulkLookupParams = serde_json::from_value(params.clone())
            .map_err(|e| McpError::InvalidRequest(format!("Invalid params: {}", e)))?;

        let mut results = Vec::new();
        for ip_str in params.ips {
            let ip_u32 = self.parse_ip(&ip_str).ok();
            let info = if let (Some(ip), Some(ref table)) = (ip_u32, &self.arrow_table) {
                table.find_ip(ip)
            } else {
                None
            };

            results.push(serde_json::json!({
                "ip": ip_str,
                "asn_info": info
            }));
        }

        serde_json::to_value(&results).map_err(|e| McpError::InternalError(e.to_string()))
    }

    /// Handle cidr_analyze method
    async fn handle_cidr_analyze(&self, params: &serde_json::Value) -> Result<serde_json::Value> {
        let params: CidrAnalyzeParams = serde_json::from_value(params.clone())
            .map_err(|e| McpError::InvalidRequest(format!("Invalid params: {}", e)))?;

        let cidr =
            Cidr::parse(&params.cidr).map_err(|e| McpError::InvalidRequest(e.to_string()))?;

        Ok(serde_json::json!({
            "cidr": params.cidr,
            "network": cidr.network(),
            "broadcast": cidr.broadcast(),
            "first_usable": cidr.first_usable(),
            "last_usable": cidr.last_usable(),
            "total_ips": cidr.size(),
            "prefix_len": cidr.prefix_len()
        }))
    }

    /// Handle reverse_lookup method (placeholder)
    async fn handle_reverse_lookup(&self, params: &serde_json::Value) -> Result<serde_json::Value> {
        let params: LookupIpParams = serde_json::from_value(params.clone())
            .map_err(|e| McpError::InvalidRequest(format!("Invalid params: {}", e)))?;

        // Placeholder - would do PTR lookup
        Ok(serde_json::json!({
            "ip": params.ip,
            "hostname": null,
            "note": "Reverse DNS not yet implemented"
        }))
    }

    /// Handle enrich_data method (placeholder)
    async fn handle_enrich_data(&self, params: &serde_json::Value) -> Result<serde_json::Value> {
        let params: LookupIpParams = serde_json::from_value(params.clone())
            .map_err(|e| McpError::InvalidRequest(format!("Invalid params: {}", e)))?;

        // Placeholder - would add WHOIS, GeoIP data
        Ok(serde_json::json!({
            "ip": params.ip,
            "whois": null,
            "geoip": null,
            "note": "Enrichment not yet implemented"
        }))
    }

    /// Parse IP address string to u32
    fn parse_ip(&self, ip: &str) -> std::result::Result<u32, String> {
        let octets: Vec<&str> = ip.split('.').collect();
        if octets.len() != 4 {
            return Err("Invalid IP address format".to_string());
        }

        let mut result = 0u32;
        for (i, octet_str) in octets.iter().enumerate() {
            let octet: u8 = octet_str
                .parse()
                .map_err(|_| format!("Invalid octet: {}", octet_str))?;
            result |= (octet as u32) << (24 - i * 8);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_request() {
        let request = r#"{
            "jsonrpc": "2.0",
            "method": "ping",
            "params": {},
            "id": 1
        }"#;

        let parsed: JsonRpcRequest = serde_json::from_str(request).unwrap();
        assert_eq!(parsed.jsonrpc, "2.0");
        assert_eq!(parsed.method, "ping");
    }

    #[tokio::test]
    async fn test_ping_method() {
        let server = McpServer::new(None).unwrap();
        let request = r#"{
            "jsonrpc": "2.0",
            "method": "ping",
            "params": {},
            "id": 1
        }"#;

        let response = server.handle_request(request).await.unwrap();
        assert!(response.contains("\"status\":\"ok\""));
    }

    #[tokio::test]
    async fn test_method_not_found() {
        let server = McpServer::new(None).unwrap();
        let request = r#"{
            "jsonrpc": "2.0",
            "method": "unknown_method",
            "params": {},
            "id": 1
        }"#;

        let response = server.handle_request(request).await.unwrap();
        assert!(response.contains("-32601")); // Method not found error code
    }

    #[tokio::test]
    async fn test_invalid_jsonrpc_version() {
        let server = McpServer::new(None).unwrap();
        let request = r#"{
            "jsonrpc": "1.0",
            "method": "ping",
            "params": {},
            "id": 1
        }"#;

        let response = server.handle_request(request).await.unwrap();
        assert!(response.contains("-32600")); // Invalid request error code
    }

    #[test]
    fn test_parse_ip() {
        let server = McpServer::new(None).unwrap();
        assert_eq!(server.parse_ip("8.8.8.8").unwrap(), 0x08080808);
        assert_eq!(server.parse_ip("192.168.1.1").unwrap(), 0xC0A80101);
        assert!(server.parse_ip("invalid").is_err());
        assert!(server.parse_ip("256.0.0.1").is_err());
    }
}
