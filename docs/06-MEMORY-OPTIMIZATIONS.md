# Memory Optimizations

**Project:** RASN - Rust ASN Mapper  
**Version:** 1.0  
**Date:** October 26, 2025

---

## 1. Zero-Copy Deserialization

### Avoid Allocations with Borrowed Data

```rust
use serde::{Deserialize, Serialize};
use bytes::Bytes;

// Instead of String (heap allocation)
#[derive(Deserialize)]
pub struct AsnInfo<'a> {
    pub asn: u32,
    #[serde(borrow)]
    pub name: &'a str,  // Borrows from input buffer
    #[serde(borrow)]
    pub organization: &'a str,
    pub country: &'a str,
}

// Zero-copy JSON parsing with simd-json
pub fn parse_asn_info_zero_copy(data: &mut [u8]) -> Result<AsnInfo> {
    let value = simd_json::to_borrowed_value(data)?;
    let info: AsnInfo = simd_json::from_borrowed_value(&value)?;
    Ok(info)
}
```

**Benefits:**
- No heap allocations for strings
- 3-5x faster parsing
- 50% less memory usage

---

## 2. Arena Allocators

### Bulk Allocation for Request Lifetime

```rust
use bumpalo::Bump;

pub struct RequestArena {
    arena: Bump,
}

impl RequestArena {
    pub fn new() -> Self {
        Self { arena: Bump::new() }
    }
    
    // Allocate strings in arena
    pub fn alloc_str(&self, s: &str) -> &str {
        self.arena.alloc_str(s)
    }
    
    // All memory freed at once when arena drops
}

// Usage in request handler
pub async fn handle_request(input: String) -> Result<Response> {
    let arena = RequestArena::new();
    
    // All allocations use arena
    let parts: Vec<&str> = input
        .split(',')
        .map(|s| arena.alloc_str(s))
        .collect();
    
    process(parts).await
    
    // Arena dropped here - all memory freed in O(1)
}
```

**Benefits:**
- Allocation: O(1) bump pointer
- Deallocation: O(1) bulk free
- Better cache locality
- 10-20x faster than individual allocations

---

## 3. String Interning

### Deduplicate Common Strings

```rust
use std::sync::Arc;
use dashmap::DashMap;

pub struct StringInterner {
    cache: Arc<DashMap<String, Arc<str>>>,
}

impl StringInterner {
    pub fn intern(&self, s: impl AsRef<str>) -> Arc<str> {
        let s = s.as_ref();
        
        // Return existing if present
        if let Some(cached) = self.cache.get(s) {
            return cached.clone();
        }
        
        // Otherwise, intern it
        let interned: Arc<str> = Arc::from(s);
        self.cache.insert(s.to_string(), interned.clone());
        interned
    }
}

// Usage: Intern common values
pub struct AsnInfo {
    pub asn: u32,
    pub organization: Arc<str>,  // Many ASNs have same org
    pub country: Arc<str>,       // Many share country codes
}
```

**Benefits:**
- Memory: 90% reduction for repeated strings
- Comparison: O(1) pointer comparison
- Cache-friendly: Single storage

**Example Savings:**
- 10k ASNs for "GOOGLE" → 1 allocation vs 10k
- Country codes ("US", "GB") → 2 allocations vs 10k

---

## 4. Compact Data Structures

### BitFlags Instead of Bools

```rust
use bitflags::bitflags;

// Instead of:
pub struct Config {
    pub cache_enabled: bool,      // 1 byte + 7 padding
    pub ipv6_enabled: bool,       // 1 byte + 7 padding
    pub verbose: bool,            // 1 byte + 7 padding
    pub offline_mode: bool,       // 1 byte + 7 padding
}  // Total: 32 bytes (with padding)

// Use:
bitflags! {
    pub struct ConfigFlags: u8 {
        const CACHE_ENABLED = 0b0001;
        const IPV6_ENABLED  = 0b0010;
        const VERBOSE       = 0b0100;
        const OFFLINE_MODE  = 0b1000;
    }
}  // Total: 1 byte

pub struct CompactConfig {
    pub flags: ConfigFlags,
}  // Total: 1 byte (32x smaller!)
```

### Packed Structs

```rust
#[repr(C, packed)]
pub struct IpRange {
    pub start: u32,  // 4 bytes
    pub end: u32,    // 4 bytes
    pub asn: u16,    // 2 bytes
}  // Total: 10 bytes (no padding)

// vs unpacked: 16 bytes
```

---

## 5. Copy-on-Write (CoW)

### Avoid Cloning When Possible

```rust
use std::borrow::Cow;

pub fn process_domain(domain: Cow<str>) -> Cow<str> {
    if domain.contains("www.") {
        // Need modification - allocate
        Cow::Owned(domain.replace("www.", ""))
    } else {
        // No change - reuse input
        domain
    }
}

// Usage
let domain = "example.com";
let processed = process_domain(Cow::Borrowed(domain));
// No allocation!

let domain2 = "www.example.com";
let processed2 = process_domain(Cow::Borrowed(domain2));
// One allocation only when needed
```

---

## 6. SmallVec - Stack Allocation

### Avoid Heap for Small Collections

```rust
use smallvec::{SmallVec, smallvec};

// Most ASNs have < 8 IP ranges
pub struct AsnInfo {
    pub asn: u32,
    pub ip_ranges: SmallVec<[IpNet; 8]>,  // First 8 on stack
}

// Usage
let mut info = AsnInfo {
    asn: 14421,
    ip_ranges: smallvec![],
};

info.ip_ranges.push(ip1);  // Stack allocation
info.ip_ranges.push(ip2);  // Stack allocation
// ... up to 8 items, no heap allocation

info.ip_ranges.push(ip9);  // Spills to heap only if needed
```

**Benefits:**
- 0 allocations for common case
- Better cache locality
- Automatic spillover to heap

---

## 7. Bytes - Shared Buffers

### Reference-Counted Buffer Slices

```rust
use bytes::{Bytes, BytesMut};

pub struct ResponseCache {
    buffer: Bytes,  // Shared, immutable
}

impl ResponseCache {
    pub fn get_slice(&self, start: usize, end: usize) -> Bytes {
        // Cheap clone - just bumps ref count
        self.buffer.slice(start..end)
    }
}

// vs Vec<u8>::clone() which copies entire buffer
```

**Benefits:**
- Clone: O(1) vs O(n)
- Memory: Single allocation, multiple views
- Zero-copy slicing

---

## 8. Custom Allocators

### mimalloc - Fast General Purpose

```toml
[dependencies]
mimalloc = { version = "0.1", default-features = false }
```

```rust
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
```

**Performance:**
- 10-20% faster than system allocator
- Better fragmentation handling
- Thread-local caches

### jemalloc - For High Concurrency

```toml
[dependencies]
jemallocator = "0.5"
```

```rust
#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
```

---

## 9. Memory Pooling

### Reuse Buffers

```rust
use std::sync::Mutex;

pub struct BufferPool {
    pool: Mutex<Vec<Vec<u8>>>,
    buffer_size: usize,
}

impl BufferPool {
    pub fn acquire(&self) -> Vec<u8> {
        let mut pool = self.pool.lock().unwrap();
        pool.pop().unwrap_or_else(|| Vec::with_capacity(self.buffer_size))
    }
    
    pub fn release(&self, mut buffer: Vec<u8>) {
        buffer.clear();  // Reset but keep capacity
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < 100 {  // Max pool size
            pool.push(buffer);
        }
        // else: drop buffer
    }
}

// Usage
let pool = BufferPool::new(4096);

async fn process_request(pool: &BufferPool) {
    let mut buffer = pool.acquire();
    // Use buffer...
    pool.release(buffer);
}
```

**Benefits:**
- Reduced allocation frequency
- Amortized allocation cost
- Better memory reuse

---

## 10. Lazy Initialization

### Defer Allocations Until Needed

```rust
use once_cell::sync::Lazy;
use regex::Regex;

// Compiled once, globally
static DOMAIN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?i)[a-z0-9-_]+(\.[a-z0-9-]+)+\.?$").unwrap()
});

pub fn is_domain(s: &str) -> bool {
    DOMAIN_REGEX.is_match(s)
}

// vs compiling regex on every call
```

---

## 11. Streaming Instead of Buffering

### Constant Memory Usage

```rust
use tokio::io::{AsyncBufReadExt, BufReader};

// Bad: Load entire file
async fn process_file_buffered(path: &Path) -> Result<()> {
    let content = tokio::fs::read_to_string(path).await?;  // Entire file in RAM
    for line in content.lines() {
        process_line(line).await?;
    }
    Ok(())
}

// Good: Stream lines
async fn process_file_streaming(path: &Path) -> Result<()> {
    let file = tokio::fs::File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    while let Some(line) = lines.next_line().await? {
        process_line(&line).await?;
    }  // Constant memory: one line at a time
    
    Ok(())
}
```

**Comparison:**
- Buffered: O(n) memory (entire file)
- Streaming: O(1) memory (single line)

---

## 12. Compression

### Reduce In-Memory Size

```rust
use flate2::write::GzEncoder;
use flate2::Compression;

pub struct CompressedCache {
    data: HashMap<String, Vec<u8>>,  // Compressed data
}

impl CompressedCache {
    pub fn store(&mut self, key: String, value: &[u8]) {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(value).unwrap();
        let compressed = encoder.finish().unwrap();
        
        self.data.insert(key, compressed);
    }
    
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let compressed = self.data.get(key)?;
        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).ok()?;
        Some(decompressed)
    }
}
```

**Tradeoff:**
- Space: 3-5x reduction
- CPU: +10-20% overhead
- Good for: Large, infrequently accessed data

---

## 13. Memory Profiling Tools

### Detect Leaks and Hotspots

```bash
# Valgrind (comprehensive but slow)
valgrind --leak-check=full --show-leak-kinds=all ./rasn

# Heaptrack (faster, Linux)
heaptrack ./rasn
heaptrack --analyze heaptrack.rasn.*.gz

# DHAT (Rust-friendly)
cargo build --release
valgrind --tool=dhat --dhat-out-file=dhat.out ./target/release/rasn

# Memory profiling in code
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    let _profiler = dhat::Profiler::new_heap();
    // ... run code ...
}
```

---

## Performance Impact Summary

| Optimization | Memory Reduction | Speed Impact |
|--------------|------------------|--------------|
| Zero-copy deserialization | 50% | 3-5x faster |
| Arena allocators | 30% | 10-20x faster |
| String interning | 90% (repeated strings) | 1x |
| Compact structs | 2-32x | Neutral |
| SmallVec | 100% (< N items) | 1.1x faster |
| Bytes | 80% (shared) | 1x |
| mimalloc | 10-20% | 1.1-1.2x faster |
| Buffer pooling | 50% | 2-3x faster |
| Streaming | O(1) vs O(n) | Neutral |
| Compression | 3-5x | 0.8-0.9x |

**Overall:** 2-10x memory reduction, 2-5x performance improvement.
