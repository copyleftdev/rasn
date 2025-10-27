# Technical Requirements Document

**Project:** RASN - Rust ASN Mapper  
**Version:** 1.0  
**Date:** October 26, 2025

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   CLI / MCP Interface                        │
│               (clap + JSON-RPC 2.0)                          │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────┴────────────────────────────────────────┐
│                Core Business Logic                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Resolver   │  │  ASN Lookup  │  │  CIDR Engine │      │
│  │   (async)    │  │   Service    │  │   (SIMD)     │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────┴────────────────────────────────────────┐
│               Infrastructure Layer                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │    Cache     │  │ Rate Limiter │  │   Metrics    │      │
│  │  (3-tier)    │  │  (Governor)  │  │ (Prometheus) │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────┴────────────────────────────────────────┐
│                  Data Source Layer                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   PD API     │  │   Local DB   │  │    WHOIS     │      │
│  │  (reqwest)   │  │  (RocksDB)   │  │   (custom)   │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

---

## Technology Stack

### Core Runtime
- **Async:** `tokio` 1.35+ (multi-threaded work-stealing)
- **Allocator:** `mimalloc` (10-20% faster than system malloc)
- **Errors:** `anyhow` (app), `thiserror` (libs)

### Networking
- **HTTP:** `reqwest` 0.11+ with connection pooling
- **DNS:** `hickory-dns` 0.24+ (async, high-perf)
- **TLS:** `rustls` 0.22+ (no OpenSSL dependency)
- **Proxy:** `tokio-socks` for SOCKS5

### Data Structures
- **IP/CIDR:** `ipnet` 2.9+
- **Concurrent Maps:** `dashmap` 5.5+ (lock-free)
- **Cache:** `lru` 0.12+
- **Radix Trees:** `radix_trie` 0.2+ (IP prefix lookups)
- **Bitmaps:** `roaring` 0.10+ (compressed IP ranges)

### Serialization
- **JSON:** `simd-json` 0.13+ (SIMD-accelerated)
- **Serde:** `serde` 1.0+ (derive macros)
- **CSV:** `csv` 1.3+
- **MessagePack:** `rmp-serde` 1.1+ (compact storage)

### Storage
- **Embedded DB:** `rocksdb` 0.22+
- **Redis:** `redis` 0.24+ (async)
- **GeoIP:** `maxminddb` 0.24+

### CLI & MCP
- **CLI:** `clap` 4.4+ (derive macros)
- **JSON-RPC:** `jsonrpc-core` 18.0+
- **TUI:** `ratatui` 0.25+ (interactive mode)
- **Colors:** `owo-colors` 3.5+

### Observability
- **Logging:** `tracing` 0.1+ + `tracing-subscriber`
- **Metrics:** `metrics` 0.21+ + Prometheus exporter
- **Tracing:** `opentelemetry` 0.21+ (optional)

### Performance
- **Data Parallel:** `rayon` 1.8+
- **SIMD:** `std::simd` (nightly) or `wide` (stable)
- **Rate Limit:** `governor` 0.6+
- **Backoff:** `backoff` 0.4+

### Security
- **Keyring:** `keyring` 2.1+ (OS credential storage)
- **Secrets:** `secrecy` 0.8+ (memory protection)
- **Certs:** `rustls-native-certs` 0.7+

---

## Core Components

### 1. Input Parser

```rust
pub enum InputType {
    Asn(u32),
    Ip(IpAddr),
    Domain(String),
    Organization(String),
}

pub trait InputParser {
    fn parse(&self, input: &str) -> Result<InputType>;
    fn parse_batch(&self, inputs: Vec<&str>) -> Vec<Result<InputType>>;
}
```

**Implementation:**
- Regex for domain validation (compile once)
- IP via `IpAddr::from_str` (stdlib, fast)
- ASN: strip "AS", parse u32
- Default to Organization

**Performance:**
- <1μs per input
- Zero allocations for valid inputs
- Parallelizable batches

### 2. DNS Resolver

```rust
pub struct DnsResolver {
    hickory: AsyncResolver,
    cache: Arc<DashMap<String, Vec<IpAddr>>>,
    config: ResolverConfig,
}

impl DnsResolver {
    pub async fn resolve(&self, domain: &str) -> Result<Vec<IpAddr>>;
    pub async fn resolve_batch(&self, domains: Vec<&str>) 
        -> Vec<Result<Vec<IpAddr>>>;
}
```

**Features:**
- Concurrent A + AAAA queries
- Built-in TTL-based caching
- Connection pooling (UDP sockets)
- 5s timeout, 2 retries

**Performance:**
- >5000 queries/sec (1000 concurrent)
- <50ms P99 uncached
- <1ms cached

### 3. ASN Lookup Service

```rust
pub trait AsnDataSource: Send + Sync {
    async fn lookup_asn(&self, asn: u32) -> Result<AsnInfo>;
    async fn lookup_ip(&self, ip: IpAddr) -> Result<AsnInfo>;
    async fn lookup_org(&self, org: &str) -> Result<Vec<AsnInfo>>;
}

pub struct AsnLookupService {
    sources: Vec<Box<dyn AsnDataSource>>,
    cache: Arc<dyn Cache>,
    rate_limiter: RateLimiter,
}
```

**Data Sources (Priority):**
1. Memory Cache - O(1)
2. Disk Cache - ~1ms
3. Local DB - <10ms
4. PD API - ~100-500ms
5. WHOIS - slow, last resort

**Features:**
- Try sources in order, first success wins
- Concurrent requests (`join_all`, max 100)
- Rate limiting (1000 req/s configurable)
- Cache TTL 24h

**Performance:**
- <1ms cached (memory)
- <10ms local DB
- <100ms P95 API

### 4. CIDR Engine

```rust
pub struct CidrEngine;

impl CidrEngine {
    // IP range → minimal CIDR set
    pub fn range_to_cidrs(start: IpAddr, end: IpAddr) -> Vec<IpNet>;
    
    // Aggregate adjacent/overlapping
    pub fn aggregate(cidrs: Vec<IpNet>) -> Vec<IpNet>;
    
    // IP membership test
    pub fn contains(cidrs: &[IpNet], ip: IpAddr) -> bool;
    
    // Find overlaps
    pub fn find_overlaps(cidrs: &[IpNet]) -> Vec<(IpNet, IpNet)>;
    
    // SIMD batch ops
    pub fn contains_batch_simd(cidrs: &[IpNet], ips: &[IpAddr]) 
        -> Vec<bool>;
}
```

**Algorithms:**
- Range → CIDR: Binary splitting, O(log n)
- Aggregation: Sort + merge, O(n log n)
- Overlap: Interval tree, O(n log n) build, O(log n) query
- Contains: Radix tree, O(log n)

**Performance:**
- Range to CIDR: <1μs
- Aggregate 10k: <100ms
- Contains: <100ns (radix tree)
- Overlap 1M: <500ms

### 5. Multi-Tier Cache

```
L1: Memory (LRU) → L2: Disk (RocksDB) → L3: Redis (Optional)
```

```rust
pub trait Cache: Send + Sync {
    async fn get(&self, key: &str) -> Option<Vec<u8>>;
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Duration);
    async fn delete(&self, key: &str);
    async fn clear(&self);
    fn stats(&self) -> CacheStats;
}
```

**Strategy:**
- L1: 1000 entries, LRU, <1ms
- L2: Unlimited, TTL, ~1-5ms
- L3: Shared, ~10ms

**Keys:**
- `asn:{number}`
- `ip:{address}`
- `domain:{name}`
- `org:{name}`

**Performance:**
- L1 hit: <1μs
- L2 hit: <5ms
- L3 hit: <10ms
- Target >80% hit rate

---

## Data Models

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsnInfo {
    pub asn: u32,
    pub name: String,
    pub organization: String,
    pub country: String,
    pub ip_ranges: Vec<IpRange>,
    pub last_updated: DateTime<Utc>,
    
    // Optional enrichment
    pub geo_location: Option<GeoLocation>,
    pub abuse_contacts: Option<Vec<Contact>>,
    pub bgp_prefixes: Option<Vec<BgpPrefix>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpRange {
    pub start: IpAddr,
    pub end: IpAddr,
    pub cidr: Vec<IpNet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub country: String,
    pub city: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
}
```

---

## Configuration (TOML)

```toml
[general]
log_level = "info"
cache_ttl = "24h"
timeout = "30s"

[api]
endpoint = "https://asn.projectdiscovery.io/api/v1/asnmap"
api_key = "${PDCP_API_KEY}"
rate_limit = 1000

[dns]
resolvers = ["8.8.8.8:53", "1.1.1.1:53"]
timeout = "5s"
retries = 2

[cache]
memory_size = 1000
disk_path = "~/.rasn/cache"
redis_url = "redis://localhost:6379"  # optional

[database]
path = "~/.rasn/asn.db"
auto_update = true
update_interval = "7d"

[mcp]
transport = "stdio"
http_port = 8080
enable_streaming = true

[performance]
max_concurrent_dns = 1000
max_concurrent_api = 100
worker_threads = 0  # 0 = auto
```

---

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum RasnError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("DNS resolution failed: {0}")]
    DnsResolution(String),
    
    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Timeout after {0:?}")]
    Timeout(Duration),
}
```

**Policy:**
- User errors → Clear message + suggestion
- Network errors → Retry with backoff
- API errors → Fall back to local DB
- Fatal errors → Graceful shutdown

---

## Testing Strategy

**Unit Tests:**
- All public functions
- Edge cases, error conditions
- Mock external dependencies
- Target: >80% coverage

**Integration Tests:**
- End-to-end CLI workflows
- MCP tool invocation
- Multi-source fetching
- Cache behavior

**Performance Tests:**
- DNS throughput
- CIDR operation latency
- API batching
- Memory profiling

**Load Tests:**
- 10k concurrent ops
- 1M IP batch
- Long-running cache
- Memory leak detection
