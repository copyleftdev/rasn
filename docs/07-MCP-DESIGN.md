# MCP Server Design

**Project:** RASN - Rust ASN Mapper  
**Version:** 1.0  
**Date:** October 26, 2025

---

## MCP Protocol Overview

Model Context Protocol (MCP) is an open standard for connecting AI assistants to external tools and data sources. RASN implements a fully compliant MCP server.

**Key Concepts:**
- **Tools** - Functions AI can call
- **Resources** - Data AI can read
- **Prompts** - Pre-built templates
- **Transport** - Communication layer (stdio/HTTP/WS)

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    AI Assistant                          │
│              (Claude, GPT, etc.)                         │
└──────────────────┬──────────────────────────────────────┘
                   │ JSON-RPC 2.0
┌──────────────────┴──────────────────────────────────────┐
│                 MCP Transport Layer                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │   STDIO     │  │    HTTP     │  │  WebSocket  │     │
│  │  (primary)  │  │ (secondary) │  │  (streams)  │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
└──────────────────┬──────────────────────────────────────┘
                   │
┌──────────────────┴──────────────────────────────────────┐
│              MCP Server Core                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │Tool Registry│  │  Resource   │  │   Prompt    │     │
│  │  (7 tools)  │  │  Handlers   │  │  Templates  │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
└──────────────────┬──────────────────────────────────────┘
                   │
┌──────────────────┴──────────────────────────────────────┐
│            RASN Business Logic                           │
│     (ASN Lookup, DNS Resolution, CIDR Ops)               │
└──────────────────────────────────────────────────────────┘
```

---

## Core Traits

### MCP Server Trait

```rust
#[async_trait]
pub trait McpServer: Send + Sync {
    // Initialization handshake
    async fn initialize(&self, params: InitializeParams) 
        -> Result<InitializeResult>;
    
    // Tool management
    async fn list_tools(&self) -> Result<Vec<Tool>>;
    async fn call_tool(&self, request: ToolCallRequest) 
        -> Result<ToolCallResponse>;
    
    // Resource management
    async fn list_resources(&self) -> Result<Vec<Resource>>;
    async fn read_resource(&self, uri: &str) 
        -> Result<ResourceContent>;
    
    // Prompt management
    async fn list_prompts(&self) -> Result<Vec<Prompt>>;
    async fn get_prompt(&self, name: &str) 
        -> Result<PromptResult>;
    
    // Server info
    fn capabilities(&self) -> ServerCapabilities;
}
```

### Tool Trait

```rust
#[async_trait]
pub trait McpTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> serde_json::Value;
    
    async fn execute(
        &self, 
        params: serde_json::Value
    ) -> Result<ToolResult>;
    
    // Optional: streaming support
    async fn execute_streaming(
        &self,
        params: serde_json::Value
    ) -> Result<impl Stream<Item = ToolChunk>> {
        // Default: non-streaming
        let result = self.execute(params).await?;
        stream::once(async move { ToolChunk::Complete(result) })
    }
}
```

---

## Implementation

### Server Implementation

```rust
pub struct RasnMcpServer {
    asn_service: Arc<AsnLookupService>,
    dns_resolver: Arc<DnsResolver>,
    cidr_engine: Arc<CidrEngine>,
    tools: HashMap<String, Box<dyn McpTool>>,
    resources: HashMap<String, Box<dyn McpResource>>,
}

impl RasnMcpServer {
    pub fn new(
        asn_service: Arc<AsnLookupService>,
        dns_resolver: Arc<DnsResolver>,
        cidr_engine: Arc<CidrEngine>,
    ) -> Self {
        let mut tools: HashMap<String, Box<dyn McpTool>> = HashMap::new();
        
        // Register all 7 tools
        tools.insert("asn_lookup".into(), 
            Box::new(AsnLookupTool::new(asn_service.clone())));
        tools.insert("ip_to_asn".into(), 
            Box::new(IpToAsnTool::new(asn_service.clone())));
        tools.insert("domain_to_asn".into(), 
            Box::new(DomainToAsnTool::new(
                dns_resolver.clone(), 
                asn_service.clone()
            )));
        tools.insert("org_to_asn".into(), 
            Box::new(OrgToAsnTool::new(asn_service.clone())));
        tools.insert("cidr_operations".into(), 
            Box::new(CidrOperationsTool::new(cidr_engine.clone())));
        tools.insert("asn_relationship".into(), 
            Box::new(AsnRelationshipTool::new(asn_service.clone())));
        tools.insert("batch_lookup".into(), 
            Box::new(BatchLookupTool::new(
                asn_service.clone(),
                dns_resolver.clone()
            )));
        
        Self {
            asn_service,
            dns_resolver,
            cidr_engine,
            tools,
            resources: Self::register_resources(),
        }
    }
}

#[async_trait]
impl McpServer for RasnMcpServer {
    async fn initialize(&self, params: InitializeParams) 
        -> Result<InitializeResult> 
    {
        Ok(InitializeResult {
            protocol_version: "2024-11-05".into(),
            capabilities: self.capabilities(),
            server_info: ServerInfo {
                name: "rasn".into(),
                version: env!("CARGO_PKG_VERSION").into(),
            },
        })
    }
    
    async fn list_tools(&self) -> Result<Vec<Tool>> {
        let tools: Vec<Tool> = self.tools
            .values()
            .map(|tool| Tool {
                name: tool.name().into(),
                description: tool.description().into(),
                input_schema: tool.input_schema(),
            })
            .collect();
        
        Ok(tools)
    }
    
    async fn call_tool(&self, request: ToolCallRequest) 
        -> Result<ToolCallResponse> 
    {
        let tool = self.tools
            .get(&request.name)
            .ok_or_else(|| Error::ToolNotFound(request.name.clone()))?;
        
        // Validate input against schema
        validate_json(&request.arguments, &tool.input_schema())?;
        
        // Execute tool
        let result = tool.execute(request.arguments).await?;
        
        Ok(ToolCallResponse {
            content: vec![result],
            is_error: false,
        })
    }
    
    fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            tools: Some(ToolCapability { 
                list_changed: false 
            }),
            resources: Some(ResourceCapability { 
                subscribe: false,
                list_changed: false,
            }),
            prompts: Some(PromptCapability { 
                list_changed: false 
            }),
            experimental: None,
        }
    }
}
```

---

## Transport Layers

### 1. STDIO Transport (Primary)

```rust
pub struct StdioTransport {
    stdin: BufReader<Stdin>,
    stdout: Stdout,
}

impl StdioTransport {
    pub fn new() -> Self {
        Self {
            stdin: BufReader::new(io::stdin()),
            stdout: io::stdout(),
        }
    }
    
    pub async fn run(&mut self, server: Arc<dyn McpServer>) -> Result<()> {
        let mut line = String::new();
        
        loop {
            line.clear();
            self.stdin.read_line(&mut line).await?;
            
            if line.is_empty() {
                break;  // EOF
            }
            
            // Parse JSON-RPC request
            let request: JsonRpcRequest = serde_json::from_str(&line)?;
            
            // Handle request
            let response = self.handle_request(request, &server).await;
            
            // Write JSON-RPC response
            let response_json = serde_json::to_string(&response)?;
            writeln!(self.stdout, "{}", response_json)?;
            self.stdout.flush()?;
        }
        
        Ok(())
    }
    
    async fn handle_request(
        &self,
        request: JsonRpcRequest,
        server: &Arc<dyn McpServer>,
    ) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => {
                let params: InitializeParams = 
                    serde_json::from_value(request.params).unwrap();
                let result = server.initialize(params).await.unwrap();
                JsonRpcResponse::success(request.id, result)
            }
            "tools/list" => {
                let result = server.list_tools().await.unwrap();
                JsonRpcResponse::success(request.id, result)
            }
            "tools/call" => {
                let params: ToolCallRequest = 
                    serde_json::from_value(request.params).unwrap();
                let result = server.call_tool(params).await.unwrap();
                JsonRpcResponse::success(request.id, result)
            }
            // ... other methods
            _ => JsonRpcResponse::error(
                request.id,
                -32601,
                "Method not found",
            ),
        }
    }
}
```

### 2. HTTP Transport (Secondary)

```rust
use axum::{Router, routing::post, Json};

pub struct HttpTransport {
    server: Arc<dyn McpServer>,
    port: u16,
}

impl HttpTransport {
    pub async fn run(self) -> Result<()> {
        let app = Router::new()
            .route("/mcp", post(handle_mcp_request))
            .with_state(self.server);
        
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await?;
        
        Ok(())
    }
}

async fn handle_mcp_request(
    State(server): State<Arc<dyn McpServer>>,
    Json(request): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    // Same logic as STDIO transport
    let response = match request.method.as_str() {
        "tools/call" => {
            let params: ToolCallRequest = 
                serde_json::from_value(request.params).unwrap();
            let result = server.call_tool(params).await.unwrap();
            JsonRpcResponse::success(request.id, result)
        }
        // ... other methods
        _ => JsonRpcResponse::error(request.id, -32601, "Method not found"),
    };
    
    Json(response)
}
```

### 3. WebSocket Transport (Streaming)

```rust
use tokio_tungstenite::WebSocketStream;

pub struct WebSocketTransport {
    ws: WebSocketStream<TcpStream>,
    server: Arc<dyn McpServer>,
}

impl WebSocketTransport {
    pub async fn run(mut self) -> Result<()> {
        while let Some(msg) = self.ws.next().await {
            let msg = msg?;
            
            if msg.is_text() {
                let request: JsonRpcRequest = 
                    serde_json::from_str(msg.to_text()?)?;
                
                // Check if streaming is requested
                if self.is_streaming_request(&request) {
                    self.handle_streaming_request(request).await?;
                } else {
                    let response = self.handle_request(request).await;
                    let response_json = serde_json::to_string(&response)?;
                    self.ws.send(Message::Text(response_json)).await?;
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_streaming_request(
        &mut self,
        request: JsonRpcRequest,
    ) -> Result<()> {
        let tool_name = request.params["name"].as_str().unwrap();
        let tool = self.server.tools.get(tool_name).unwrap();
        
        let mut stream = tool.execute_streaming(request.params).await?;
        
        while let Some(chunk) = stream.next().await {
            let chunk_json = serde_json::to_string(&chunk)?;
            self.ws.send(Message::Text(chunk_json)).await?;
        }
        
        Ok(())
    }
}
```

---

## JSON-RPC Protocol

### Request Format

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "asn_lookup",
    "arguments": {
      "asn": "AS14421",
      "include_ipv6": false
    }
  }
}
```

### Response Format

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [{
      "type": "text",
      "text": "{\"asn\":14421,\"name\":\"THERAVANCE\",\"country\":\"US\",...}"
    }],
    "isError": false
  }
}
```

### Error Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "details": "Missing required field: asn"
    }
  }
}
```

---

## Schema Validation

```rust
use jsonschema::JSONSchema;

pub struct SchemaValidator {
    schemas: HashMap<String, JSONSchema>,
}

impl SchemaValidator {
    pub fn validate(
        &self,
        tool_name: &str,
        params: &serde_json::Value,
    ) -> Result<()> {
        let schema = self.schemas
            .get(tool_name)
            .ok_or_else(|| Error::SchemaNotFound)?;
        
        let result = schema.validate(params);
        
        if let Err(errors) = result {
            let error_messages: Vec<String> = errors
                .map(|e| e.to_string())
                .collect();
            
            return Err(Error::ValidationFailed(error_messages));
        }
        
        Ok(())
    }
}
```

---

## Streaming Support

### Streaming Response Format

```rust
pub enum ToolChunk {
    Progress {
        current: usize,
        total: usize,
        message: Option<String>,
    },
    Data(serde_json::Value),
    Complete(ToolResult),
    Error(String),
}

// Example: Stream large batch results
pub async fn execute_streaming_batch(
    inputs: Vec<String>,
) -> impl Stream<Item = ToolChunk> {
    let total = inputs.len();
    
    stream! {
        for (i, input) in inputs.into_iter().enumerate() {
            // Progress update
            yield ToolChunk::Progress {
                current: i + 1,
                total,
                message: Some(format!("Processing {}", input)),
            };
            
            // Actual result
            match process_input(&input).await {
                Ok(result) => yield ToolChunk::Data(result),
                Err(e) => yield ToolChunk::Error(e.to_string()),
            }
        }
        
        // Final chunk
        yield ToolChunk::Complete(ToolResult {
            message: format!("Processed {} inputs", total),
        });
    }
}
```

---

## Error Handling

### Error Codes (JSON-RPC Standard)

```rust
pub enum McpErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    
    // Custom errors (>= -32000)
    ToolNotFound = -32000,
    ResourceNotFound = -32001,
    ValidationFailed = -32002,
    RateLimitExceeded = -32003,
}
```

### Error Response Helper

```rust
impl JsonRpcResponse {
    pub fn error(
        id: Option<serde_json::Value>,
        code: i32,
        message: impl Into<String>,
    ) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}
```

---

## Security Considerations

### Rate Limiting

```rust
use governor::{Quota, RateLimiter};

pub struct SecureMcpServer {
    server: Arc<dyn McpServer>,
    rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl SecureMcpServer {
    pub async fn call_tool(&self, request: ToolCallRequest) 
        -> Result<ToolCallResponse> 
    {
        // Check rate limit
        self.rate_limiter.until_ready().await;
        
        // Delegate to underlying server
        self.server.call_tool(request).await
    }
}
```

### Input Sanitization

```rust
fn sanitize_input(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .take(256)  // Max length
        .collect()
}
```

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tool_registration() {
        let server = create_test_server();
        let tools = server.list_tools().await.unwrap();
        assert_eq!(tools.len(), 7);
    }
    
    #[tokio::test]
    async fn test_asn_lookup_tool() {
        let server = create_test_server();
        
        let request = ToolCallRequest {
            name: "asn_lookup".into(),
            arguments: json!({
                "asn": "AS14421"
            }),
        };
        
        let response = server.call_tool(request).await.unwrap();
        assert!(!response.is_error);
    }
}
```

### Integration Tests (Claude Desktop)

```bash
# Add to Claude Desktop config
{
  "mcpServers": {
    "rasn": {
      "command": "/path/to/rasn",
      "args": ["mcp"],
      "env": {
        "PDCP_API_KEY": "your-key-here"
      }
    }
  }
}
```

---

## Performance Considerations

- **Connection pooling:** Reuse resources across tool calls
- **Caching:** Cache ASN lookups within session
- **Batching:** Group multiple tool calls when possible
- **Streaming:** Use for large result sets (>100 items)
- **Timeout:** 30s default, configurable per tool
