//! MCP Server Transports
//!
//! Implements STDIO and HTTP transports for the MCP server.

use crate::{McpError, McpServer, Result};
use std::io::{BufRead, Write};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// STDIO transport for MCP server
///
/// Reads JSON-RPC requests from stdin and writes responses to stdout.
/// Suitable for Claude Desktop and other IDE integrations.
pub struct StdioTransport {
    server: Arc<McpServer>,
}

impl StdioTransport {
    /// Create new STDIO transport
    pub fn new(server: Arc<McpServer>) -> Self {
        Self { server }
    }

    /// Run the STDIO transport (blocking)
    pub fn run_blocking(&self) -> Result<()> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        for line in stdin.lock().lines() {
            let request = line.map_err(|e| McpError::InternalError(e.to_string()))?;

            // Skip empty lines
            if request.trim().is_empty() {
                continue;
            }

            // Handle request
            let runtime = tokio::runtime::Runtime::new()
                .map_err(|e| McpError::InternalError(e.to_string()))?;

            let response = runtime.block_on(self.server.handle_request(&request))?;

            // Write response
            writeln!(stdout, "{}", response).map_err(|e| McpError::InternalError(e.to_string()))?;
            stdout
                .flush()
                .map_err(|e| McpError::InternalError(e.to_string()))?;
        }

        Ok(())
    }

    /// Run the STDIO transport (async)
    pub async fn run_async(&self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            let n = reader
                .read_line(&mut line)
                .await
                .map_err(|e| McpError::InternalError(e.to_string()))?;

            // EOF
            if n == 0 {
                break;
            }

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Handle request
            let response = self.server.handle_request(&line).await?;

            // Write response
            stdout
                .write_all(response.as_bytes())
                .await
                .map_err(|e| McpError::InternalError(e.to_string()))?;
            stdout
                .write_all(b"\n")
                .await
                .map_err(|e| McpError::InternalError(e.to_string()))?;
            stdout
                .flush()
                .await
                .map_err(|e| McpError::InternalError(e.to_string()))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::McpServer;

    #[test]
    fn test_stdio_transport_creation() {
        let server = Arc::new(McpServer::new(None).unwrap());
        let _transport = StdioTransport::new(server);
    }
}
