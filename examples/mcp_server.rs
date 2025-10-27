//! MCP server example
//!
//! Run with: cargo run --example mcp_server

use rasn_mcp::McpServer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RASN - MCP Server Example\n");

    // Create MCP server (without data for this example)
    let server = McpServer::new(None)?;
    let server = Arc::new(server);

    println!("MCP Server ready!");
    println!("Available methods:");
    println!("  - lookup_ip");
    println!("  - lookup_asn");
    println!("  - lookup_domain");
    println!("  - bulk_lookup");
    println!("  - cidr_analyze");
    println!("  - reverse_lookup");
    println!("  - enrich_data");
    println!("  - ping");

    // Example request
    let request = r#"{
        "jsonrpc": "2.0",
        "method": "ping",
        "params": {},
        "id": 1
    }"#;

    println!("\nSending request: {}", request);
    let response = server.handle_request(request).await?;
    println!("Response: {}", response);

    // CIDR analysis example
    let cidr_request = r#"{
        "jsonrpc": "2.0",
        "method": "cidr_analyze",
        "params": {"cidr": "10.0.0.0/8"},
        "id": 2
    }"#;

    println!("\nSending CIDR request...");
    let cidr_response = server.handle_request(cidr_request).await?;
    println!("Response: {}", cidr_response);

    println!("\nMCP server example complete!");
    Ok(())
}
