# Columnar Storage Strategy with Apache Arrow

## Overview

For maximum performance, RASN will use **Apache Arrow** for in-memory columnar storage of reference data, enabling:
- Sub-microsecond lookups
- Massive SIMD acceleration (4-16x)
- Minimal memory footprint (20-30 MB)
- Zero-copy data sharing

---

## Why Columnar + Arrow?

### 1. Memory Efficiency

**Row-oriented (traditional):**
```
Record 1: [ip_start, ip_end, asn, country, org]
Record 2: [ip_start, ip_end, asn, country, org]
```
- Poor cache locality
- Hard to vectorize

**Column-oriented (Arrow):**
```
ip_start:  [1.0.0.0, 1.0.1.0, 1.0.4.0, ...]
ip_end:    [1.0.0.255, 1.0.3.255, 1.0.7.255, ...]
asn:       [13335, 0, 38803, ...]
country:   [US, None, AU, ...]
org:       [CLOUDFLARE, Not routed, GTELECOM, ...]
```
- Excellent cache locality
- Perfect for SIMD
- Dictionary encoding for strings

### 2. SIMD Potential

**IP Range Search (core operation):**
```rust
// Traditional: O(log n) binary search
fn find_ip_range(ip: u32) -> Option<usize> {
    ranges.binary_search_by(|r| {
        if ip < r.start { Ordering::Greater }
        else if ip > r.end { Ordering::Less }
        else { Ordering::Equal }
    })
}

// Columnar SIMD: Compare 8 IPs at once with AVX2
fn find_ip_range_simd(ip: u32, starts: &[u32], ends: &[u32]) -> Option<usize> {
    use std::arch::x86_64::*;
    
    let search = _mm256_set1_epi32(ip as i32);
    
    for chunk in starts.chunks(8).zip(ends.chunks(8)) {
        let starts_vec = _mm256_loadu_si256(chunk.0.as_ptr() as *const __m256i);
        let ends_vec = _mm256_loadu_si256(chunk.1.as_ptr() as *const __m256i);
        
        // Compare 8 ranges simultaneously
        let ge_start = _mm256_cmpgt_epi32(search, starts_vec);
        let le_end = _mm256_cmpgt_epi32(ends_vec, search);
        let in_range = _mm256_and_si256(ge_start, le_end);
        
        if _mm256_movemask_epi8(in_range) != 0 {
            // Found match, extract index
        }
    }
}
```

**Speedup: 4-8x with AVX2, 8-16x with AVX-512**

### 3. Memory Footprint

Based on reconnaissance data:

| Dataset | Rows | Row Format | Columnar (Arrow) | Savings |
|---------|------|------------|------------------|---------|
| ip2asn-v4 | 510k | 28 MB | 12 MB | 57% |
| ip2asn-v6 | 167k | 14 MB | 6 MB | 57% |
| asn-info | 130k | 5.7 MB | 2.5 MB | 56% |
| **Total** | **807k** | **48 MB** | **20-22 MB** | **54%** |

**Techniques:**
- **Dictionary encoding:** Country codes (200 unique → 1 byte index)
- **Run-length encoding:** ASN ranges (many consecutive 0s)
- **Bit packing:** ASN numbers (21 bits instead of 32)

---

## Architecture Design

### Hybrid Storage Model

```
┌─────────────────────────────────────────────────────┐
│                 RASN Lookup Engine                  │
└─────────────────────────────────────────────────────┘
                         │
        ┌────────────────┴────────────────┐
        │                                 │
┌───────▼────────┐              ┌────────▼──────────┐
│  Hot Path      │              │   Cold Path       │
│  (Arrow)       │              │   (RocksDB)       │
│                │              │                   │
│ • IP ranges    │              │ • Historical data │
│ • ASN metadata │              │ • Bulk storage    │
│ • In-memory    │              │ • Overflow        │
│ • SIMD lookup  │              │                   │
│ • <1µs latency │              │ • ~50µs latency   │
└────────────────┘              └───────────────────┘
```

### Data Loading Strategy

**Startup sequence:**
1. Load Arrow files from disk (mmap or read)
2. Validate checksums
3. Build acceleration structures (if needed)
4. Fallback to RocksDB for misses

**Runtime:**
- 99.9% of queries hit Arrow in-memory
- 0.1% fall through to RocksDB (rare ASNs, historical data)

---

## Implementation Plan

### Crate: `rasn-arrow`

**Dependencies:**
```toml
[dependencies]
arrow = "51.0"
arrow-array = "51.0"
arrow-schema = "51.0"
parquet = "51.0"  # For on-disk format
memmap2 = "0.9"   # Memory-mapped files
rayon = "1.8"     # Parallel processing
```

### Core Types

```rust
use arrow::array::*;
use arrow::datatypes::*;

pub struct IpRangeTable {
    // Column arrays
    start_ips: UInt32Array,   // IPv4 starts (or UInt128 for IPv6)
    end_ips: UInt32Array,     // IPv4 ends
    asns: UInt32Array,        // ASN numbers
    countries: DictionaryArray<UInt8Type, Utf8Type>,  // Country codes
    orgs: DictionaryArray<UInt16Type, Utf8Type>,      // Organizations
    
    // Metadata
    len: usize,
    version: u32,
}

impl IpRangeTable {
    /// Load from Parquet file (compressed Arrow format)
    pub fn from_parquet(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let reader = ParquetRecordBatchReader::try_new(file, 1024)?;
        
        // Read all batches into single table
        let batches: Vec<_> = reader.collect::<Result<_>>()?;
        Self::from_batches(&batches)
    }
    
    /// Memory-mapped loading (zero-copy)
    pub fn from_mmap(path: &Path) -> Result<Self> {
        let mmap = unsafe { Mmap::map(&File::open(path)?)? };
        // Parse Arrow IPC format directly from mmap
        Self::from_ipc_bytes(&mmap)
    }
    
    /// Find IP address in range (SIMD-accelerated)
    #[inline]
    pub fn find_ip(&self, ip: Ipv4Addr) -> Option<AsnInfo> {
        let ip_u32 = u32::from(ip);
        
        #[cfg(target_feature = "avx2")]
        {
            self.find_ip_simd_avx2(ip_u32)
        }
        
        #[cfg(not(target_feature = "avx2"))]
        {
            self.find_ip_binary_search(ip_u32)
        }
    }
    
    #[cfg(target_feature = "avx2")]
    fn find_ip_simd_avx2(&self, ip: u32) -> Option<AsnInfo> {
        use std::arch::x86_64::*;
        
        let starts = self.start_ips.values();
        let ends = self.end_ips.values();
        let search = unsafe { _mm256_set1_epi32(ip as i32) };
        
        // Process 8 ranges at once
        for (i, (start_chunk, end_chunk)) in 
            starts.chunks_exact(8).zip(ends.chunks_exact(8)).enumerate() 
        {
            unsafe {
                let starts_vec = _mm256_loadu_si256(
                    start_chunk.as_ptr() as *const __m256i
                );
                let ends_vec = _mm256_loadu_si256(
                    end_chunk.as_ptr() as *const __m256i
                );
                
                // ip >= start
                let ge = _mm256_cmpgt_epi32(search, starts_vec);
                // ip <= end
                let le = _mm256_cmpgt_epi32(ends_vec, search);
                // in range
                let mask = _mm256_and_si256(ge, le);
                let result = _mm256_movemask_epi8(mask);
                
                if result != 0 {
                    // Found match, extract index
                    let lane = result.trailing_zeros() / 4;
                    let idx = i * 8 + lane as usize;
                    return Some(self.get_asn_info(idx));
                }
            }
        }
        
        // Handle remainder with binary search
        self.find_ip_binary_search(ip)
    }
    
    fn find_ip_binary_search(&self, ip: u32) -> Option<AsnInfo> {
        // Standard binary search fallback
        let idx = self.start_ips.values()
            .binary_search(&ip)
            .ok()?;
        Some(self.get_asn_info(idx))
    }
    
    #[inline]
    fn get_asn_info(&self, idx: usize) -> AsnInfo {
        AsnInfo {
            asn: self.asns.value(idx),
            country: self.countries.value(idx).to_string(),
            org: self.orgs.value(idx).to_string(),
        }
    }
}
```

### Conversion Script: Python → Parquet

```python
# scripts/convert_to_arrow.py

import pandas as pd
import pyarrow as pa
import pyarrow.parquet as pq
from pathlib import Path

def convert_ip2asn_to_parquet(tsv_path: Path, output_path: Path):
    """Convert IPtoASN TSV to optimized Parquet format"""
    
    # Read TSV
    df = pd.read_csv(
        tsv_path, 
        sep='\t', 
        names=['start_ip', 'end_ip', 'asn', 'country', 'org'],
        dtype={'asn': 'uint32'}
    )
    
    # Convert IP strings to uint32
    df['start_ip_int'] = df['start_ip'].apply(ip_to_int)
    df['end_ip_int'] = df['end_ip'].apply(ip_to_int)
    
    # Create Arrow table with optimized schema
    schema = pa.schema([
        pa.field('start_ip', pa.uint32()),
        pa.field('end_ip', pa.uint32()),
        pa.field('asn', pa.uint32()),
        pa.field('country', pa.dictionary(pa.uint8(), pa.utf8())),
        pa.field('org', pa.dictionary(pa.uint16(), pa.utf8())),
    ])
    
    table = pa.Table.from_pandas(
        df[['start_ip_int', 'end_ip_int', 'asn', 'country', 'org']],
        schema=schema
    )
    
    # Write with maximum compression
    pq.write_table(
        table,
        output_path,
        compression='zstd',
        compression_level=9,
        use_dictionary=True,
        write_statistics=True,
    )
    
    print(f"Converted {len(df):,} records")
    print(f"Original: {tsv_path.stat().st_size / 1024 / 1024:.1f} MB")
    print(f"Parquet: {output_path.stat().st_size / 1024 / 1024:.1f} MB")

def ip_to_int(ip_str: str) -> int:
    parts = ip_str.split('.')
    return (int(parts[0]) << 24) + (int(parts[1]) << 16) + \
           (int(parts[2]) << 8) + int(parts[3])
```

---

## Performance Benchmarks (Projected)

### Lookup Latency

| Approach | Latency | Throughput | Notes |
|----------|---------|------------|-------|
| RocksDB (hot cache) | 10-50 µs | 20k-100k ops/s | Block cache overhead |
| RocksDB (cold) | 100-500 µs | 2k-10k ops/s | Disk I/O |
| **Arrow (binary search)** | **0.5-1 µs** | **1-2M ops/s** | Pure memory |
| **Arrow (SIMD AVX2)** | **0.1-0.2 µs** | **5-10M ops/s** | 8-way parallel |
| **Arrow (SIMD AVX-512)** | **0.05-0.1 µs** | **10-20M ops/s** | 16-way parallel |

### Memory Usage

| Component | Size | Notes |
|-----------|------|-------|
| IPv4 ranges (Arrow) | 12 MB | Dictionary encoded |
| IPv6 ranges (Arrow) | 6 MB | Compressed |
| ASN metadata (Arrow) | 2.5 MB | Dictionary encoded |
| Acceleration indexes | 2 MB | Optional B-tree |
| **Total** | **22-24 MB** | Fits in L3 cache |

### Compression Ratios

| Format | Size | Ratio | Load Time |
|--------|------|-------|-----------|
| CSV/TSV | 48 MB | 1.0x | ~100ms |
| JSON | 72 MB | 1.5x | ~200ms |
| MessagePack | 35 MB | 0.73x | ~80ms |
| **Arrow IPC** | **28 MB** | **0.58x** | **~10ms** |
| **Parquet** | **18 MB** | **0.38x** | **~30ms** |

---

## Hybrid Strategy: Arrow + RocksDB

### Use Cases

**Arrow (in-memory, hot path):**
- ✅ IP to ASN lookups (99.9% of queries)
- ✅ ASN metadata lookups
- ✅ Country/org reverse lookups (small index)
- ✅ Recent data only (last 30-90 days)

**RocksDB (on-disk, cold path):**
- ✅ Historical data (> 90 days old)
- ✅ Bulk storage for enrichment
- ✅ Write-heavy operations
- ✅ Data that doesn't fit in memory
- ✅ Persistent cache

### Implementation

```rust
pub struct HybridLookup {
    // Hot path: Arrow in-memory tables
    arrow_ipv4: Arc<IpRangeTable>,
    arrow_ipv6: Arc<IpRangeTable>,
    arrow_asn: Arc<AsnMetadataTable>,
    
    // Cold path: RocksDB persistent storage
    rocksdb: Arc<RocksDB>,
    
    // Metrics
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
}

impl HybridLookup {
    pub async fn lookup_ip(&self, ip: IpAddr) -> Result<AsnInfo> {
        // Try Arrow first (hot path)
        if let Some(info) = self.arrow_lookup(ip) {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(info);
        }
        
        // Fallback to RocksDB (cold path)
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
        self.rocksdb_lookup(ip).await
    }
    
    fn arrow_lookup(&self, ip: IpAddr) -> Option<AsnInfo> {
        match ip {
            IpAddr::V4(ipv4) => self.arrow_ipv4.find_ip(ipv4),
            IpAddr::V6(ipv6) => self.arrow_ipv6.find_ip(ipv6),
        }
    }
}
```

---

## Migration Plan

### Phase 1: Add Arrow Support (Week 4-5)
1. Create `rasn-arrow` crate
2. Implement basic Arrow table loading
3. Add conversion script: CSV/TSV → Parquet
4. Benchmark vs RocksDB

### Phase 2: SIMD Optimization (Week 6-7)
1. Implement AVX2 SIMD search
2. Add AVX-512 support (optional)
3. Runtime CPU feature detection
4. Benchmark SIMD gains

### Phase 3: Hybrid Integration (Week 8)
1. Combine Arrow + RocksDB
2. Automatic hot/cold data tiering
3. Metrics and monitoring
4. Production testing

---

## Caveats & Considerations

### Advantages
- ✅ **10-100x faster** than RocksDB for hot data
- ✅ **Minimal memory** (20-30 MB)
- ✅ **SIMD-friendly** data layout
- ✅ **Zero-copy** with mmap
- ✅ **Standard format** (Apache Arrow)

### Disadvantages
- ⚠️ **Read-only** (no updates without reload)
- ⚠️ **Requires rebuild** for data updates
- ⚠️ **Memory resident** (not ideal for huge datasets)
- ⚠️ **Complex** compared to simple RocksDB

### When to Use Arrow
- ✅ Reference data that updates infrequently (daily/weekly)
- ✅ Hot path queries (IP lookups, ASN metadata)
- ✅ Systems with ample RAM (>1 GB)
- ✅ CPU-bound workloads benefiting from SIMD

### When to Use RocksDB
- ✅ Frequently updated data
- ✅ Large datasets (>1 GB)
- ✅ Write-heavy workloads
- ✅ Persistent state across restarts
- ✅ Historical archives

---

## Conclusion

**Recommendation:** Use **Arrow for hot path** (99.9% of queries) + **RocksDB for cold path**.

This hybrid approach delivers:
- Sub-microsecond latency for common queries
- 5-20M ops/sec throughput with SIMD
- 20-30 MB memory footprint
- Fallback to RocksDB for edge cases

The Arrow columnar format with SIMD will make RASN one of the fastest ASN lookup tools ever built!
