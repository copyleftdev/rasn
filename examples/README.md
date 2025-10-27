# RASN Examples

This directory contains example code demonstrating various RASN features.

## Running Examples

```bash
# Basic IP lookup
cargo run --example basic_lookup

# CIDR operations
cargo run --example cidr_operations

# MCP server
cargo run --example mcp_server
```

## Examples

### basic_lookup.rs

Demonstrates basic IP-to-ASN lookups using Arrow tables.

**Features:**
- Loading Parquet data
- IP address lookup
- Displaying ASN information

### cidr_operations.rs

Shows CIDR block operations and IP range queries.

**Features:**
- CIDR parsing
- Network calculations
- IP containment checks
- IP iteration

### mcp_server.rs

Illustrates MCP JSON-RPC server usage.

**Features:**
- Server initialization
- Request handling
- Available methods
- Example requests

## Requirements

Some examples require additional data files:

- `basic_lookup.rs` - Requires `data/asn.parquet`
- Other examples work without data

## More Information

See the [main documentation](../README.md) for complete usage guide.
