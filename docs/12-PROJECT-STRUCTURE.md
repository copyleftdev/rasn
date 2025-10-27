# Project Structure

**Project:** RASN - Rust ASN Mapper  
**Version:** 1.0

---

## Workspace Layout

```
rasn/
├── Cargo.toml           # Workspace root
├── README.md
├── LICENSE
├── docs/                # All documentation
├── crates/
│   ├── rasn-cli/       # CLI binary
│   ├── rasn-core/      # Core types
│   ├── rasn-client/    # API clients
│   ├── rasn-resolver/  # DNS resolution
│   ├── rasn-cidr/      # CIDR operations
│   ├── rasn-arrow/     # Arrow/Parquet loading
│   ├── rasn-db/        # Local database
│   ├── rasn-cache/     # Caching layer
│   └── rasn-mcp/       # MCP server
├── benches/            # Performance tests
├── data/               # Optimized databases
│   ├── arrow/          # Parquet files (13.6 MB)
│   ├── rocks/          # RocksDB (cold storage)
│   └── cache/          # Runtime cache
├── reference_data/     # Source data (TSV/CSV)
└── scripts/            # Utility scripts
```

---

## Crate Details

### rasn-arrow (NEW)
**Purpose:** Apache Arrow/Parquet columnar storage for hot-path lookups  
**Dependencies:** `arrow`, `parquet`, `memmap2`

**Key Types:**
- `IpRangeTable` - In-memory IPv4/IPv6 range table
- `AsnMetadataTable` - ASN information table
- `CountryIndex` - Country → IP range index

**Features:**
- Memory-mapped Parquet loading
- SIMD-accelerated binary search (AVX2/AVX-512)
- Sub-microsecond lookups
- Zero-copy data access

### rasn-cli
**Purpose:** Command-line interface  
**Dependencies:** clap, rasn-core, rasn-mcp

```
src/
├── main.rs
├── args.rs
├── output.rs
└── commands/
    ├── lookup.rs
    ├── batch.rs
    └── mcp.rs
```

### rasn-core
**Purpose:** Core types and traits  
**Dependencies:** serde, ipnet

```
src/
├── lib.rs
├── types.rs
├── error.rs
└── traits.rs
```

### rasn-mcp
**Purpose:** MCP server implementation  
**Dependencies:** jsonrpc-core, rasn-core

```
src/
├── lib.rs
├── server.rs
├── tools/
│   ├── asn_lookup.rs
│   ├── ip_to_asn.rs
│   └── ...
└── transport/
    ├── stdio.rs
    └── http.rs
```

---

## Module Organization

```rust
// rasn-core/src/lib.rs
pub mod types;
pub mod error;
pub mod traits;

pub use types::*;
pub use error::*;
```

---

## Build Configuration

```toml
[workspace]
members = [
    "crates/rasn-cli",
    "crates/rasn-core",
    "crates/rasn-client",
    "crates/rasn-resolver",
    "crates/rasn-cidr",
    "crates/rasn-db",
    "crates/rasn-cache",
    "crates/rasn-mcp",
]

[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

---

## File Conventions

- **Tests:** `tests/` directory or `#[cfg(test)]` modules
- **Benchmarks:** `benches/` directory
- **Examples:** `examples/` directory
- **Docs:** `docs/` at workspace root
