# RASN Data Directory

Optimized database files for RASN lookups.

## Structure

```
data/
├── arrow/              # Parquet files (columnar, SIMD-optimized)
│   ├── ip2asn-v4.parquet
│   ├── ip2asn-v6.parquet
│   ├── asn-metadata.parquet
│   └── country-index.parquet
├── rocks/              # RocksDB database (cold path)
└── cache/              # Runtime cache
```

## Building Databases

```bash
# Build all databases from reference_data
python3 scripts/build_databases.py
```

## File Formats

- **Parquet**: Apache Arrow columnar format with ZSTD compression
- **RocksDB**: Key-value store (populated by Rust code)
