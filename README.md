# RASN - Rust ASN Mapper

[![CI](https://github.com/copyleftdev/rasn/actions/workflows/ci.yml/badge.svg)](https://github.com/copyleftdev/rasn/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

High-performance ASN lookup system with SIMD acceleration, multi-level caching, and MCP server for AI agents.

## Features

- **SIMD Acceleration** - Sub-microsecond lookups with AVX2
- **Apache Arrow/Parquet** - Columnar storage for IP ranges
- **Multi-Level Cache** - LRU + RocksDB cold storage
- **MCP Server** - JSON-RPC 2.0 API for AI agents
- **Network Enrichment** - DNS, WHOIS, GeoIP integration
- **CIDR Operations** - /8-/32 range queries
- **Parallel Processing** - Rayon batch operations
- **Production Ready** - Rate limiting, metrics, Docker support

## Performance

Benchmarked on modern hardware (hyperfine, 100 runs):

```
CLI Lookup (cold start + lookup):
  Time (mean ± σ):     218.9 ms ±   4.9 ms
  Range (min … max):   207.0 ms … 237.5 ms

In-memory lookup (after load):
  < 1 microsecond per lookup with SIMD
  > 1M lookups/second sustained throughput
```

**Note**: Cold start includes TSV parsing (28MB data). Use MCP server or keep CLI running for production workloads.

## Installation

### Quick Install (with data - installs to ~/.local)

```bash
git clone https://github.com/copyleftdev/rasn.git
cd rasn
make install    # Downloads data + installs binary (no sudo!)
# or: ./install.sh
```

Data is automatically downloaded on first install.

### Binary only

```bash
cargo install --path crates/rasn-cli
```

**Note**: Binary-only install requires manual data setup. See [INSTALL.md](INSTALL.md) for details.

### Docker (data included)

```bash
docker pull ghcr.io/copyleftdev/rasn:latest
```

## Usage

### CLI

```bash
# IP lookup
rasn lookup 8.8.8.8

# Batch processing
rasn batch --file ips.txt --workers 10

# JSON output
rasn lookup --output json 1.1.1.1

# MCP server (for Claude Desktop)
rasn mcp stdio
```

### MCP Server

Add to Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "rasn": {
      "command": "rasn",
      "args": ["mcp", "stdio"]
    }
  }
}
```

**Available MCP Tools:**
- `lookup_ip` - IP to ASN lookup
- `lookup_asn` - ASN to IP ranges
- `lookup_domain` - DNS + ASN resolution
- `bulk_lookup` - Batch processing
- `cidr_analyze` - CIDR calculations
- `reverse_lookup` - PTR records
- `enrich_data` - WHOIS + GeoIP

### Docker

```bash
# CLI
docker run --rm ghcr.io/copyleftdev/rasn:latest lookup 8.8.8.8

# MCP Server
docker run --rm -it ghcr.io/copyleftdev/rasn:latest mcp stdio
```

## Performance

| Operation | Latency |
|-----------|---------|
| Arrow lookup | <100ns |
| Cache hit | <100ns |
| CIDR /24 | <10ms |
| WHOIS query | <500ms |

## Configuration

**Environment Variables:**
- `RASN_API_KEY` - API key for external services

**Check Status:**

```bash
rasn auth status
```

## Development

### Setup

```bash
# Install dev tools and git hooks
make dev-setup
```

### Common Tasks

```bash
# Run tests
cargo test --all-features --workspace

# Check format
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-features --workspace -- -D warnings

# Build docs
cargo doc --all-features --no-deps --workspace

# Run all checks (same as pre-commit)
./hooks/pre-commit
```

## Project Structure

```text
rasn/
├── crates/
│   ├── rasn-core/      # Core types & security
│   ├── rasn-arrow/     # Arrow/Parquet + SIMD
│   ├── rasn-cache/     # Multi-level caching
│   ├── rasn-cidr/      # CIDR operations
│   ├── rasn-client/    # HTTP client + rate limiting
│   ├── rasn-db/        # RocksDB storage
│   ├── rasn-geoip/     # GeoIP integration
│   ├── rasn-mcp/       # MCP JSON-RPC server
│   ├── rasn-resolver/  # DNS resolution
│   ├── rasn-whois/     # WHOIS client
│   └── rasn-cli/       # CLI interface
├── examples/           # Usage examples
└── docs/               # Documentation
```

## License

MIT - see [LICENSE](LICENSE) for details.
