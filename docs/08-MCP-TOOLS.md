# MCP Tool Specifications

**Project:** RASN - Rust ASN Mapper  
**Version:** 1.0  
**Date:** October 26, 2025

---

## Tool 1: asn_lookup

### Description
Lookup comprehensive information for an ASN number including IP ranges, organization, country, and optional enrichment data.

### Input Schema

```json
{
  "type": "object",
  "properties": {
    "asn": {
      "type": "string",
      "description": "ASN number with or without 'AS' prefix",
      "pattern": "^(AS)?\\d+$",
      "examples": ["AS14421", "14421"]
    },
    "include_ipv6": {
      "type": "boolean",
      "description": "Include IPv6 ranges in results",
      "default": false
    },
    "enrich": {
      "type": "boolean",
      "description": "Include GeoIP and WHOIS enrichment",
      "default": false
    }
  },
  "required": ["asn"]
}
```

### Response Format

```json
{
  "asn": 14421,
  "name": "THERAVANCE",
  "organization": "Theravance Biopharma US, Inc.",
  "country": "US",
  "ip_ranges": [
    {
      "start": "216.101.17.0",
      "end": "216.101.17.255",
      "cidr": ["216.101.17.0/24"]
    }
  ],
  "enrichment": {
    "geo_location": {
      "country": "United States",
      "city": "South San Francisco",
      "latitude": 37.6547,
      "longitude": -122.4077
    },
    "abuse_contacts": ["abuse@theravance.com"],
    "registration_date": "2001-03-15"
  }
}
```

### Implementation

```rust
pub struct AsnLookupTool {
    service: Arc<AsnLookupService>,
}

#[async_trait]
impl McpTool for AsnLookupTool {
    fn name(&self) -> &str { "asn_lookup" }
    
    fn description(&self) -> &str {
        "Lookup comprehensive ASN information including ranges and enrichment"
    }
    
    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "asn": {
                    "type": "string",
                    "pattern": "^(AS)?\\d+$"
                },
                "include_ipv6": { "type": "boolean", "default": false },
                "enrich": { "type": "boolean", "default": false }
            },
            "required": ["asn"]
        })
    }
    
    async fn execute(&self, params: serde_json::Value) 
        -> Result<ToolResult> 
    {
        let asn: String = params["asn"].as_str().unwrap().into();
        let include_ipv6 = params["include_ipv6"].as_bool().unwrap_or(false);
        let enrich = params["enrich"].as_bool().unwrap_or(false);
        
        let info = self.service.lookup_asn(&asn).await?;
        
        let mut response = json!(info);
        
        if !include_ipv6 {
            // Filter out IPv6 ranges
            if let Some(ranges) = response["ip_ranges"].as_array_mut() {
                ranges.retain(|r| is_ipv4_range(r));
            }
        }
        
        Ok(ToolResult::text(response.to_string()))
    }
}
```

---

## Tool 2: ip_to_asn

### Description
Find which ASN owns a given IP address with optional reverse DNS and geolocation.

### Input Schema

```json
{
  "type": "object",
  "properties": {
    "ip": {
      "type": "string",
      "description": "IPv4 or IPv6 address",
      "pattern": "^(?:[0-9]{1,3}\\.){3}[0-9]{1,3}$|^[0-9a-fA-F:]+$",
      "examples": ["8.8.8.8", "2001:4860:4860::8888"]
    },
    "enrich": {
      "type": "boolean",
      "description": "Include geolocation and reverse DNS",
      "default": true
    },
    "include_neighbors": {
      "type": "boolean",
      "description": "Include adjacent ASN ranges",
      "default": false
    }
  },
  "required": ["ip"]
}
```

### Response Format

```json
{
  "ip": "8.8.8.8",
  "asn": 15169,
  "organization": "GOOGLE",
  "country": "US",
  "cidr": "8.8.8.0/24",
  "enrichment": {
    "reverse_dns": "dns.google",
    "geo_location": {
      "country": "United States",
      "city": "Mountain View",
      "latitude": 37.4056,
      "longitude": -122.0775
    }
  },
  "neighbors": [
    {
      "asn": 15169,
      "cidr": "8.8.4.0/24"
    }
  ]
}
```

---

## Tool 3: domain_to_asn

### Description
Resolve domain to IPs and find all associated ASNs.

### Input Schema

```json
{
  "type": "object",
  "properties": {
    "domain": {
      "type": "string",
      "description": "Domain name to resolve",
      "pattern": "^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?(\\.[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?)*$",
      "examples": ["google.com", "cloudflare.com"]
    },
    "resolvers": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Custom DNS resolvers (IP:port)",
      "default": ["8.8.8.8:53", "1.1.1.1:53"]
    },
    "include_ipv6": {
      "type": "boolean",
      "default": true
    },
    "follow_cname": {
      "type": "boolean",
      "description": "Follow CNAME records",
      "default": true
    }
  },
  "required": ["domain"]
}
```

### Response Format

```json
{
  "domain": "google.com",
  "resolved_ips": [
    "142.250.185.46",
    "2607:f8b0:4004:c07::71"
  ],
  "asns": [
    {
      "asn": 15169,
      "organization": "GOOGLE",
      "country": "US",
      "ip_count": 2
    }
  ],
  "cname_chain": []
}
```

---

## Tool 4: org_to_asn

### Description
Find all ASNs owned by an organization.

### Input Schema

```json
{
  "type": "object",
  "properties": {
    "organization": {
      "type": "string",
      "description": "Organization name (case-insensitive)",
      "minLength": 2,
      "examples": ["GOOGLE", "CLOUDFLARE", "AMAZON"]
    },
    "country": {
      "type": "string",
      "description": "Filter by ISO 3166-1 alpha-2 country code",
      "pattern": "^[A-Z]{2}$",
      "examples": ["US", "GB", "DE"]
    },
    "exact_match": {
      "type": "boolean",
      "description": "Exact organization name match",
      "default": false
    }
  },
  "required": ["organization"]
}
```

### Response Format

```json
{
  "organization": "GOOGLE",
  "matches": [
    {
      "asn": 15169,
      "name": "GOOGLE",
      "country": "US",
      "ip_range_count": 8765
    },
    {
      "asn": 396982,
      "name": "GOOGLE-CLOUD-PLATFORM",
      "country": "US",
      "ip_range_count": 1234
    }
  ],
  "total_asns": 2,
  "total_ip_ranges": 9999
}
```

---

## Tool 5: cidr_operations

### Description
Perform CIDR calculations and set operations.

### Input Schema

```json
{
  "type": "object",
  "properties": {
    "operation": {
      "type": "string",
      "enum": ["aggregate", "supernet", "split", "contains", "overlaps", "intersect", "subtract"],
      "description": "Operation to perform"
    },
    "cidrs": {
      "type": "array",
      "items": { "type": "string", "pattern": "^\\d+\\.\\d+\\.\\d+\\.\\d+/\\d+$" },
      "description": "List of CIDR ranges",
      "minItems": 1
    },
    "target": {
      "type": "string",
      "description": "Target CIDR for binary operations (contains, overlaps)",
      "pattern": "^\\d+\\.\\d+\\.\\d+\\.\\d+/\\d+$"
    },
    "split_prefix": {
      "type": "integer",
      "description": "New prefix length for split operation",
      "minimum": 0,
      "maximum": 32
    }
  },
  "required": ["operation", "cidrs"]
}
```

### Response Format

```json
{
  "operation": "aggregate",
  "input_count": 256,
  "output": [
    "192.168.0.0/23",
    "192.168.2.0/24"
  ],
  "output_count": 2,
  "compression_ratio": 128.0
}
```

### Operations

**aggregate**: Merge adjacent/overlapping CIDRs
```json
{
  "operation": "aggregate",
  "cidrs": ["192.168.0.0/24", "192.168.1.0/24"]
}
// Result: ["192.168.0.0/23"]
```

**supernet**: Find smallest supernet containing all CIDRs
```json
{
  "operation": "supernet",
  "cidrs": ["192.168.0.0/24", "192.168.2.0/24"]
}
// Result: ["192.168.0.0/22"]
```

**split**: Split CIDR into smaller subnets
```json
{
  "operation": "split",
  "cidrs": ["192.168.0.0/24"],
  "split_prefix": 26
}
// Result: ["192.168.0.0/26", "192.168.0.64/26", "192.168.0.128/26", "192.168.0.192/26"]
```

**contains**: Check if target is in any CIDR
```json
{
  "operation": "contains",
  "cidrs": ["192.168.0.0/24"],
  "target": "192.168.0.100/32"
}
// Result: true
```

**overlaps**: Find overlapping CIDRs
```json
{
  "operation": "overlaps",
  "cidrs": ["192.168.0.0/23", "192.168.1.0/24"]
}
// Result: [{"cidr1": "192.168.0.0/23", "cidr2": "192.168.1.0/24"}]
```

---

## Tool 6: asn_relationship

### Description
Analyze BGP relationships between ASNs (peering, transit).

### Input Schema

```json
{
  "type": "object",
  "properties": {
    "asn": {
      "type": "string",
      "pattern": "^(AS)?\\d+$",
      "description": "ASN to analyze"
    },
    "relationship_type": {
      "type": "string",
      "enum": ["peers", "upstreams", "downstreams", "all"],
      "default": "all",
      "description": "Type of relationships to return"
    },
    "max_depth": {
      "type": "integer",
      "minimum": 1,
      "maximum": 3,
      "default": 1,
      "description": "Relationship depth (1=direct, 2=2-hop, etc.)"
    }
  },
  "required": ["asn"]
}
```

### Response Format

```json
{
  "asn": 15169,
  "organization": "GOOGLE",
  "relationships": {
    "peers": [
      {
        "asn": 174,
        "organization": "COGENT",
        "relationship": "peer"
      }
    ],
    "upstreams": [],
    "downstreams": [
      {
        "asn": 396982,
        "organization": "GOOGLE-CLOUD-PLATFORM",
        "relationship": "customer"
      }
    ]
  },
  "summary": {
    "peer_count": 100,
    "upstream_count": 0,
    "downstream_count": 50
  }
}
```

---

## Tool 7: batch_lookup

### Description
Perform bulk ASN lookups efficiently with automatic batching.

### Input Schema

```json
{
  "type": "object",
  "properties": {
    "inputs": {
      "type": "array",
      "items": { "type": "string" },
      "description": "List of ASNs, IPs, domains, or organizations",
      "minItems": 1,
      "maxItems": 10000
    },
    "auto_detect": {
      "type": "boolean",
      "description": "Automatically detect input type",
      "default": true
    },
    "deduplicate": {
      "type": "boolean",
      "description": "Remove duplicate results",
      "default": true
    },
    "parallel": {
      "type": "boolean",
      "description": "Process in parallel",
      "default": true
    },
    "fail_fast": {
      "type": "boolean",
      "description": "Stop on first error",
      "default": false
    }
  },
  "required": ["inputs"]
}
```

### Response Format

```json
{
  "total_inputs": 1000,
  "successful": 998,
  "failed": 2,
  "duration_ms": 1234,
  "results": [
    {
      "input": "8.8.8.8",
      "type": "ip",
      "asn": 15169,
      "organization": "GOOGLE"
    }
  ],
  "errors": [
    {
      "input": "invalid.domain",
      "error": "DNS resolution failed"
    }
  ]
}
```

### Streaming Support

```json
// Progress updates during batch processing
{
  "type": "progress",
  "current": 500,
  "total": 1000,
  "message": "Processing batch 5/10"
}

// Individual results as they complete
{
  "type": "result",
  "input": "8.8.8.8",
  "asn": 15169
}

// Final summary
{
  "type": "complete",
  "successful": 998,
  "failed": 2
}
```

---

## Error Handling

### Common Error Codes

```rust
pub enum ToolError {
    // Input validation (400-level)
    InvalidInput { field: String, reason: String },
    MissingRequired { field: String },
    
    // Data not found (404-level)
    AsnNotFound { asn: u32 },
    DomainNotResolved { domain: String },
    
    // Rate limiting (429)
    RateLimitExceeded { retry_after: Duration },
    
    // Service errors (500-level)
    ServiceUnavailable { service: String },
    Timeout { operation: String },
}
```

### Error Response Format

```json
{
  "error": {
    "code": "ASN_NOT_FOUND",
    "message": "ASN 99999 not found in database",
    "details": {
      "asn": 99999,
      "suggestion": "Verify ASN number is correct"
    }
  }
}
```

---

## Performance Targets

| Tool | P50 Latency | P95 Latency | Max Throughput |
|------|-------------|-------------|----------------|
| asn_lookup | <50ms | <100ms | 1000/s |
| ip_to_asn | <20ms | <50ms | 5000/s |
| domain_to_asn | <100ms | <500ms | 500/s |
| org_to_asn | <200ms | <1s | 100/s |
| cidr_operations | <10ms | <50ms | 10000/s |
| asn_relationship | <500ms | <2s | 50/s |
| batch_lookup | 1ms/item | 5ms/item | 10000/s |
