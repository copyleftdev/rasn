# Data Source Integration

RASN uses a hybrid approach with Apache Arrow columnar storage for hot-path queries and multiple fallback sources:

---

## Primary Storage: Apache Arrow/Parquet

**Source:** Preprocessed from IPtoASN, sapics, and other public databases  
**Format:** Parquet (columnar, ZSTD compressed)  
**Location:** `data/arrow/`  
**Update Frequency:** Daily (via `scripts/build_databases.py`)

### Files:
- `ip2asn-v4.parquet` (8.1 MB) - 510k IPv4 ranges → ASN
- `ip2asn-v6.parquet` (2.0 MB) - 167k IPv6 ranges → ASN
- `asn-metadata.parquet` (2.1 MB) - 130k ASN descriptions
- `country-index.parquet` (1.4 MB) - Country → IP mappings

### Advantages:
- **Sub-microsecond lookups** (in-memory)
- **SIMD-optimized** (AVX2/AVX-512)
- **Minimal memory** (13.6 MB total)
- **99.9% cache hit rate**

### Building Databases:
```bash
python3 scripts/build_databases.py
```

---

## Fallback Sources

### 1. ProjectDiscovery Cloud API

**Priority:** Primary  
**Type:** REST API  
**Endpoint:** `https://asn.projectdiscovery.io/api/v1/asnmap`

### Authentication
```rust
headers.insert("X-PDCP-Key", api_key);
```

### Rate Limits
- Free tier: 1000 req/day
- Paid: 10000 req/day
- Burst: 100 req/sec

---

### 2. Local ASN Database

**Priority:** Fallback  
**Type:** RocksDB embedded  
**Source:** IPtoASN.com, RIR data

### Schema
```
Key: asn:{number}
Value: MessagePack(AsnInfo)

Key: ip:{address}
Value: MessagePack(AsnInfo)
```

### Update Strategy
- Download: Weekly via cron
- Size: ~50MB compressed
- Format: TSV → RocksDB

---

### 3. WHOIS Integration

**Priority:** Enrichment  
**Type:** TCP port 43  
**Servers:** whois.arin.net, whois.ripe.net

### Query Format
```
AS14421
```

### Parse Response
```rust
fn parse_whois(response: &str) -> WhoisInfo {
    // Extract: org, contacts, dates
}
```

---

### 4. MaxMind GeoIP

**Priority:** Enrichment (optional)  
**Type:** Local MMDB file  
**License:** GeoLite2 (free)

### Usage
```rust
let reader = maxminddb::Reader::open_readfile(path)?;
let city: City = reader.lookup(ip)?;
```

---

## Data Source Priority

1. **Arrow in-memory** (< 0.001ms) - Hot path, 99.9% of queries
2. **Memory cache** (< 1ms) - API responses, enrichment data
3. **Disk cache** (~ 5ms) - Recent lookups
4. **RocksDB local** (~ 10ms) - Historical data, cold path
5. **PD Cloud API** (~ 100ms) - Live enrichment, updates
6. **WHOIS** (~ 500ms) - Fallback for unknowns

### Query Flow:
```
IP Lookup Request
    ↓
[Arrow Memory] → 99.9% HIT (0.001ms) ✓
    ↓ MISS (0.1%)
[Memory Cache] → Recent query? → HIT (1ms) ✓
    ↓ MISS
[RocksDB] → Historical? → HIT (10ms) ✓
    ↓ MISS
[PD API] → Fetch + Cache → (100ms) ✓
    ↓ FAIL
[WHOIS] → Last resort → (500ms) ✓
```
