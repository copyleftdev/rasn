# RASN - Rust ASN Mapper

[![CI](https://github.com/copyleftdev/rasn/actions/workflows/ci.yml/badge.svg)](https://github.com/copyleftdev/rasn/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

High-performance ASN lookup system with SIMD acceleration, multi-level caching, and MCP server for AI agents.

## Features

- **SIMD Acceleration** - <100ns lookups with AVX2
- **Apache Arrow/Parquet** - Columnar storage for IP ranges
- **Multi-Level Cache** - LRU + RocksDB cold storage
- **MCP Server** - JSON-RPC 2.0 API for AI agents
- **Network Enrichment** - DNS, WHOIS, GeoIP integration
- **CIDR Operations** - /8-/32 range queries
- **Parallel Processing** - Rayon batch operations
- **Production Ready** - Rate limiting, metrics, Docker support

## Installation

```bash
git clone https://github.com/copyleftdev/rasn.git
cd rasn
cargo install --path crates/rasn-cli
```

Or using Docker:

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

```bash
# Run tests
cargo test --all-features --workspace

# Check format
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-features --workspace -- -D warnings

# Build docs
cargo doc --all-features --no-deps --workspace
```

## Project Structure

```
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
┌─────────────────────────────────────────┐
│         CLI / MCP Interface             │
└──────────────┬──────────────────────────┘
               │
┌──────────────┴──────────────────────────┐
│       Business Logic Layer              │
│  • Resolver  • ASN Lookup  • CIDR Ops   │
└──────────────┬──────────────────────────┘
               │
┌──────────────┴──────────────────────────┐
│     Infrastructure Layer                │
│  • Cache  • Rate Limiter  • Metrics     │
└──────────────┬──────────────────────────┘
               │
┌──────────────┴──────────────────────────┐
│        Data Source Layer                │
│  • PD API  • Local DB  • WHOIS  • BGP   │
└─────────────────────────────────────────┘
```

---

## 🛠️ Technology Stack

- **Runtime:** Tokio (async multi-threaded)
- **HTTP:** reqwest with connection pooling
- **DNS:** hickory-dns (high-performance)
- **Storage:** RocksDB (embedded database)
- **Cache:** dashmap (lock-free), LRU, Redis
- **Serialization:** simd-json (SIMD-accelerated)
- **CLI:** clap (derive macros)
- **MCP:** JSON-RPC 2.0 over stdio/HTTP/WebSocket

See [TRD](docs/02-TRD.md) for complete technology details.

---

## 🎨 Key Optimizations

### SIMD Acceleration (3-4x)
- Vectorized IP parsing
- Parallel CIDR operations
- Batch subnet masks

### Network Optimization (10-50x)
- HTTP/2 multiplexing
- Connection pooling
- Request batching
- Smart retry with backoff

### Memory Efficiency (2-10x)
- Zero-copy deserialization
- Arena allocators
- String interning
- Compact data structures

See optimization docs for detailed techniques:
- [SIMD Optimizations](docs/04-SIMD-OPTIMIZATIONS.md)
- [Network Optimizations](docs/05-NETWORK-OPTIMIZATIONS.md)
- [Memory Optimizations](docs/06-MEMORY-OPTIMIZATIONS.md)

---

## 🗺️ Roadmap

### Phase 1-2: Foundation & Core (Weeks 1-6)
- ✅ Project setup, CLI, basic API client
- ✅ DNS resolution, input parsing, output formats

### Phase 3-4: Performance & Features (Weeks 7-12)
- ⏳ Concurrent processing, caching, local DB
- ⏳ CIDR operations, data enrichment

### Phase 5: MCP Server (Weeks 13-15)
- ⏳ JSON-RPC server, 7 tools, streaming support

### Phase 6: Production (Weeks 16-18)
- ⏳ Security hardening, observability, documentation

See [Roadmap](docs/11-ROADMAP.md) for detailed timeline.

---

## 🤝 Contributing

Contributions are welcome! Please read our contributing guidelines.

### Development Setup

```bash
# Clone repository
git clone https://github.com/yourusername/rasn.git
cd rasn

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build project
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Project Structure

```
rasn/
├── crates/
│   ├── rasn-cli/       # CLI binary
│   ├── rasn-core/      # Core types
│   ├── rasn-client/    # API clients
│   ├── rasn-resolver/  # DNS resolution
│   ├── rasn-cidr/      # CIDR operations
│   ├── rasn-db/        # Local database
│   ├── rasn-cache/     # Caching layer
│   └── rasn-mcp/       # MCP server
└── docs/               # Documentation
```

## 🙏 Acknowledgments

- Apache Arrow for columnar storage
- Tokio for async runtime
- Rayon for parallel processing
- All other amazing Rust crates used in this project
- [Anthropic](https://www.anthropic.com) for the MCP specification
- [IPtoASN](https://iptoasn.com/) for ASN database
- The Rust community for excellent crates

---

## 📞 Contact & Support

- **Issues:** [GitHub Issues](https://github.com/yourusername/rasn/issues)
- **Discussions:** [GitHub Discussions](https://github.com/yourusername/rasn/discussions)
- **Documentation:** [docs/](docs/)

---

**Built with ❤️ in Rust**

## License

MIT License - see [LICENSE](LICENSE) for details.
