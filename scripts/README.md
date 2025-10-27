# RASN Scripts

Utility scripts for data analysis, database management, and automation.

## Installation

```bash
# Install Python dependencies
pip install -r requirements.txt

# Optional: Install rich for beautiful output
pip install rich

# Optional: Install maxminddb for MMDB analysis
pip install maxminddb
```

## Scripts

### recon_reference_data.py

**Purpose:** Comprehensive reconnaissance of reference data files.

**Features:**
- Analyzes TSV/CSV/MMDB files with pandas
- Detects column types (IPv4, IPv6, ASN, country codes)
- Checks data quality (duplicates, nulls, empty values)
- Generates RocksDB schema recommendations
- Produces JSON and Markdown reports

**Usage:**
```bash
python scripts/recon_reference_data.py
```

**Output:**
- `scripts/recon_report.json`
- `scripts/recon_report.md`

## Future Scripts

- `import_to_rocksdb.py` - Import reference data to RocksDB
- `benchmark_lookups.py` - Benchmark lookup performance
- `validate_data.py` - Validate data consistency
