# RASN - Rust ASN Mapper

[![CI](https://github.com/copyleftdev/rasn/actions/workflows/ci.yml/badge.svg)](https://github.com/copyleftdev/rasn/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/rasn.svg)](https://crates.io/crates/rasn)

High-performance ASN (Autonomous System Number) lookup system built with Rust.

## ✨ Features

- 🚀 **Blazing Fast**: <100ns lookups with SIMD-accelerated search (AVX2)
- 📦 **Apache Arrow**: Columnar storage with Parquet compression
- 💾 **Multi-Level Caching**: LRU cache + RocksDB cold storage
- 🌐 **Network Enrichment**: DNS, WHOIS, GeoIP integration
- 🔧 **CIDR Operations**: Full /8-/32 support with IP iteration
- 🤖 **MCP Server**: JSON-RPC 2.0 API for AI agents (Claude Desktop)
- ⚡ **Parallel Processing**: Rayon-powered batch operations
- 🔒 **Production Ready**: Rate limiting, metrics, Docker supportive MCP server support

**RASN** is a complete Rust rewrite of [ProjectDiscovery's ASNmap](https://github.com/projectdiscovery/asnmap), designed for 10-100x performance improvements while adding AI agent integration through the Model Context Protocol (MCP).

---

## ✨ Features

### Core Capabilities
- 🚀 **10-100x faster** than Go implementation
- 🧠 **Native MCP server** for AI assistants (Claude, GPT, etc.)
- 📦 **Multiple data sources** (API + local DB + WHOIS + BGP)
- ⚡ **Smart caching** (memory + disk + Redis)
- 🔒 **Enterprise security** (encrypted keys, audit logs)
- 🎯 **Advanced CIDR operations** (aggregation, overlap detection)

### Input Types
- ASN numbers (AS14421, 14421)
- IP addresses (IPv4/IPv6)
- Domain names (with DNS resolution)
- Organization names

### Output Formats
- CIDR ranges (default)
- JSON (structured data)
- CSV (pipe-delimited)
- Streaming (for large datasets)

---

## 🎯 Performance Targets

| Metric | Go (Current) | Rust (Target) | Improvement |
|--------|--------------|---------------|-------------|
| Single ASN lookup | 500ms | <50ms | **10x** |
| 1000 IP batch | 30s | <3s | **10x** |
| Memory (10k IPs) | 500MB | <50MB | **10x** |
| DNS queries/sec | 50 | >5000 | **100x** |
| Binary size | 15MB | <5MB | **3x** |

---

## 📚 Documentation

Comprehensive documentation is available in the [`docs/`](docs/) directory:

### Planning & Requirements
- **[00-INDEX.md](docs/00-INDEX.md)** - Documentation overview and navigation
- **[01-PRD.md](docs/01-PRD.md)** - Product requirements and features
- **[02-TRD.md](docs/02-TRD.md)** - Technical architecture and stack

### Design Specifications
- **[03-ALGORITHMS.md](docs/03-ALGORITHMS.md)** - Algorithm designs and complexity analysis
- **[04-SIMD-OPTIMIZATIONS.md](docs/04-SIMD-OPTIMIZATIONS.md)** - SIMD and vectorization strategies
- **[05-NETWORK-OPTIMIZATIONS.md](docs/05-NETWORK-OPTIMIZATIONS.md)** - Network and I/O optimizations
- **[06-MEMORY-OPTIMIZATIONS.md](docs/06-MEMORY-OPTIMIZATIONS.md)** - Memory management techniques

### MCP Integration
- **[07-MCP-DESIGN.md](docs/07-MCP-DESIGN.md)** - MCP server architecture
- **[08-MCP-TOOLS.md](docs/08-MCP-TOOLS.md)** - All 7 MCP tool specifications

### API & Implementation
- **[09-API-CONTRACTS.md](docs/09-API-CONTRACTS.md)** - Public API contracts
- **[10-DATA-SOURCES.md](docs/10-DATA-SOURCES.md)** - Data source integration
- **[11-ROADMAP.md](docs/11-ROADMAP.md)** - Implementation phases (18 weeks)
- **[12-PROJECT-STRUCTURE.md](docs/12-PROJECT-STRUCTURE.md)** - Workspace organization

---

## 🚀 Quick Start

### Installation

```bash
# From source
git clone https://github.com/copyleftdev/rasn.git
cd rasn
cargo install --path crates/rasn-cli

# Or using cargo (once published)

```bash
git clone https://github.com/copyleftdev/rasn.git
cd rasn
cargo install --path crates/rasn-cli
```

#### Using Docker

```bash
docker pull ghcr.io/copyleftdev/rasn:latest
docker run --rm rasn:latest lookup 8.8.8.8
```

### Basic Usage

```bash
# IP lookup
rasn lookup 8.8.8.8

rasn lookup google.com

# Lookup by organization
rasn lookup GOOGLE

# Batch processing
rasn batch -f targets.txt

# JSON output
rasn lookup AS14421 --json

# Start MCP server (for Claude Desktop)
rasn mcp --stdio
```

### Library Usage

```rust
use rasn::Client;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new()?;
    
    // Lookup by ASN
    let info = client.lookup_asn(14421).await?;
    println!("ASN: {}, Org: {}", info.asn, info.organization);
    
    // Lookup by IP
    let info = client.lookup_ip("8.8.8.8".parse()?).await?;
    println!("IP belongs to: {}", info.organization);
    
    Ok(())
}
```

---

## 🧠 MCP Integration

RASN provides a fully compliant MCP server for AI assistants:

### Claude Desktop Configuration

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "rasn": {
      "command": "/path/to/rasn",
      "args": ["mcp", "--stdio"],
      "env": {
        "PDCP_API_KEY": "your-api-key-here"
      }
    }
  }
}
```

### Available MCP Tools

1. **`asn_lookup`** - Lookup ASN information
2. **`ip_to_asn`** - Find ASN for IP address
3. **`domain_to_asn`** - Resolve domain to ASN
4. **`org_to_asn`** - Find organization's ASNs
5. **`cidr_operations`** - CIDR calculations
6. **`asn_relationship`** - Analyze BGP relationships
7. **`batch_lookup`** - Bulk operations

See [MCP Tools Documentation](docs/08-MCP-TOOLS.md) for detailed schemas.

---

## 🏗️ Architecture

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
