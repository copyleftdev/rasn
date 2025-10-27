#!/usr/bin/env python3
"""
Build optimized database files for RASN

Converts reference data (TSV/CSV) into:
1. Apache Arrow/Parquet files (columnar, in-memory)
2. RocksDB-ready format (future)

Output structure:
    data/
    ├── arrow/              # Parquet files for in-memory loading
    │   ├── ip2asn-v4.parquet
    │   ├── ip2asn-v6.parquet
    │   ├── asn-metadata.parquet
    │   └── country-index.parquet
    ├── rocks/              # RocksDB database (populated by Rust)
    └── cache/              # Runtime cache directory
"""

import sys
import struct
import socket
from pathlib import Path
from datetime import datetime
from typing import Optional

import pandas as pd
import pyarrow as pa
import pyarrow.parquet as pq
import numpy as np

try:
    from rich.console import Console
    from rich.progress import track, Progress
    from rich.table import Table
    console = Console()
    RICH = True
except ImportError:
    RICH = False
    print("💡 Tip: Install 'rich' for better output: pip install rich")


class DatabaseBuilder:
    def __init__(self, reference_dir: Path, output_dir: Path):
        self.reference_dir = reference_dir
        self.output_dir = output_dir
        self.arrow_dir = output_dir / "arrow"
        self.rocks_dir = output_dir / "rocks"
        self.cache_dir = output_dir / "cache"
        
        # Create directories
        self.arrow_dir.mkdir(parents=True, exist_ok=True)
        self.rocks_dir.mkdir(parents=True, exist_ok=True)
        self.cache_dir.mkdir(parents=True, exist_ok=True)
        
        self.stats = {}
        
    def log(self, message: str, style: str = ""):
        if RICH:
            console.print(message, style=style)
        else:
            print(message)
    
    def build_all(self):
        """Build all database files"""
        self.log("\n╔════════════════════════════════════════════════════════════╗", "bold blue")
        self.log("║         RASN Database Builder                              ║", "bold blue")
        self.log("╚════════════════════════════════════════════════════════════╝\n", "bold blue")
        
        self.log(f"📂 Input:  {self.reference_dir}", "cyan")
        self.log(f"📂 Output: {self.output_dir}\n", "cyan")
        
        # Build each database
        self.build_ipv4_database()
        self.build_ipv6_database()
        self.build_asn_metadata()
        self.build_country_index()
        
        # Summary
        self.print_summary()
        
        self.log("\n✅ Database build complete!", "green bold")
    
    def build_ipv4_database(self):
        """Convert ip2asn-v4.tsv to Parquet"""
        self.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", "dim")
        self.log("📊 Building IPv4 Database", "yellow bold")
        self.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n", "dim")
        
        input_file = self.reference_dir / "ip2asn-v4.tsv"
        output_file = self.arrow_dir / "ip2asn-v4.parquet"
        
        if not input_file.exists():
            self.log(f"⚠️  File not found: {input_file}", "yellow")
            return
        
        self.log(f"  Reading: {input_file.name}")
        
        # Read TSV
        df = pd.read_csv(
            input_file,
            sep='\t',
            names=['start_ip', 'end_ip', 'asn', 'country', 'org'],
            dtype={'asn': 'uint32'},
            na_values=['None', 'nan', ''],
            keep_default_na=True
        )
        
        self.log(f"  Loaded: {len(df):,} records")
        
        # Convert IP strings to uint32
        self.log("  Converting IP addresses to integers...")
        df['start_ip_int'] = df['start_ip'].apply(self.ipv4_to_int)
        df['end_ip_int'] = df['end_ip'].apply(self.ipv4_to_int)
        
        # Sort by start IP for efficient binary search
        self.log("  Sorting by IP range...")
        df = df.sort_values('start_ip_int').reset_index(drop=True)
        
        # Clean up data
        df['country'] = df['country'].fillna('').astype(str)
        df['org'] = df['org'].fillna('Unknown').astype(str)
        df['asn'] = df['asn'].fillna(0).astype('uint32')
        
        # Create Arrow table with optimized schema
        self.log("  Creating Arrow table...")
        
        # Rename columns to match schema
        df_arrow = df[['start_ip_int', 'end_ip_int', 'asn', 'country', 'org']].copy()
        df_arrow.columns = ['start_ip', 'end_ip', 'asn', 'country', 'org']
        
        schema = pa.schema([
            pa.field('start_ip', pa.uint32(), metadata={'description': 'Start IP as uint32'}),
            pa.field('end_ip', pa.uint32(), metadata={'description': 'End IP as uint32'}),
            pa.field('asn', pa.uint32(), metadata={'description': 'ASN number'}),
            pa.field('country', pa.dictionary(pa.uint8(), pa.utf8()), metadata={'description': 'ISO country code'}),
            pa.field('org', pa.utf8(), metadata={'description': 'Organization name'}),  # Too many unique orgs for dictionary
        ])
        
        table = pa.Table.from_pandas(df_arrow, schema=schema)
        
        # Write Parquet with maximum optimization
        self.log("  Writing Parquet file...")
        pq.write_table(
            table,
            output_file,
            compression='zstd',
            compression_level=9,
            use_dictionary=True,
            write_statistics=True,
            row_group_size=100000,  # Optimize for bulk reads
        )
        
        # Stats
        original_size = input_file.stat().st_size
        compressed_size = output_file.stat().st_size
        ratio = (1 - compressed_size / original_size) * 100
        
        self.stats['ipv4'] = {
            'records': len(df),
            'original_mb': original_size / 1024 / 1024,
            'compressed_mb': compressed_size / 1024 / 1024,
            'ratio': ratio,
        }
        
        self.log(f"  ✅ Created: {output_file.name}", "green")
        self.log(f"     Records: {len(df):,}", "dim")
        self.log(f"     Original: {original_size / 1024 / 1024:.1f} MB", "dim")
        self.log(f"     Compressed: {compressed_size / 1024 / 1024:.1f} MB ({ratio:.1f}% reduction)\n", "dim")
    
    def build_ipv6_database(self):
        """Convert ip2asn-v6.tsv to Parquet"""
        self.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", "dim")
        self.log("📊 Building IPv6 Database", "yellow bold")
        self.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n", "dim")
        
        input_file = self.reference_dir / "ip2asn-v6.tsv"
        output_file = self.arrow_dir / "ip2asn-v6.parquet"
        
        if not input_file.exists():
            self.log(f"⚠️  File not found: {input_file}", "yellow")
            return
        
        self.log(f"  Reading: {input_file.name}")
        
        # Read TSV
        df = pd.read_csv(
            input_file,
            sep='\t',
            names=['start_ip', 'end_ip', 'asn', 'country', 'org'],
            dtype={'asn': 'uint32'},
            na_values=['None', 'nan', ''],
            keep_default_na=True
        )
        
        self.log(f"  Loaded: {len(df):,} records")
        
        # Convert IPv6 strings to uint128 (stored as bytes)
        self.log("  Converting IPv6 addresses to integers...")
        df['start_ip_bytes'] = df['start_ip'].apply(self.ipv6_to_bytes)
        df['end_ip_bytes'] = df['end_ip'].apply(self.ipv6_to_bytes)
        
        # Sort by start IP
        self.log("  Sorting by IP range...")
        df = df.sort_values('start_ip_bytes').reset_index(drop=True)
        
        # Clean up data
        df['country'] = df['country'].fillna('').astype(str)
        df['org'] = df['org'].fillna('Unknown').astype(str)
        df['asn'] = df['asn'].fillna(0).astype('uint32')
        
        # Create Arrow table
        self.log("  Creating Arrow table...")
        
        # Rename columns to match schema
        df_arrow = df[['start_ip_bytes', 'end_ip_bytes', 'asn', 'country', 'org']].copy()
        df_arrow.columns = ['start_ip', 'end_ip', 'asn', 'country', 'org']
        
        schema = pa.schema([
            pa.field('start_ip', pa.binary(16), metadata={'description': 'Start IPv6 as 16 bytes'}),
            pa.field('end_ip', pa.binary(16), metadata={'description': 'End IPv6 as 16 bytes'}),
            pa.field('asn', pa.uint32(), metadata={'description': 'ASN number'}),
            pa.field('country', pa.dictionary(pa.uint8(), pa.utf8()), metadata={'description': 'ISO country code'}),
            pa.field('org', pa.utf8(), metadata={'description': 'Organization name'}),  # Too many unique orgs for dictionary
        ])
        
        table = pa.Table.from_pandas(df_arrow, schema=schema)
        
        # Write Parquet
        self.log("  Writing Parquet file...")
        pq.write_table(
            table,
            output_file,
            compression='zstd',
            compression_level=9,
            use_dictionary=True,
            write_statistics=True,
            row_group_size=50000,
        )
        
        # Stats
        original_size = input_file.stat().st_size
        compressed_size = output_file.stat().st_size
        ratio = (1 - compressed_size / original_size) * 100
        
        self.stats['ipv6'] = {
            'records': len(df),
            'original_mb': original_size / 1024 / 1024,
            'compressed_mb': compressed_size / 1024 / 1024,
            'ratio': ratio,
        }
        
        self.log(f"  ✅ Created: {output_file.name}", "green")
        self.log(f"     Records: {len(df):,}", "dim")
        self.log(f"     Original: {original_size / 1024 / 1024:.1f} MB", "dim")
        self.log(f"     Compressed: {compressed_size / 1024 / 1024:.1f} MB ({ratio:.1f}% reduction)\n", "dim")
    
    def build_asn_metadata(self):
        """Convert asn-info.csv to Parquet"""
        self.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", "dim")
        self.log("📊 Building ASN Metadata Database", "yellow bold")
        self.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n", "dim")
        
        input_file = self.reference_dir / "asn-info.csv"
        output_file = self.arrow_dir / "asn-metadata.parquet"
        
        if not input_file.exists():
            self.log(f"⚠️  File not found: {input_file}", "yellow")
            return
        
        self.log(f"  Reading: {input_file.name}")
        
        # Read CSV with error handling
        try:
            df = pd.read_csv(
                input_file,
                dtype={'asn': 'uint32'},
                on_bad_lines='skip'  # Skip malformed lines
            )
        except Exception as e:
            self.log(f"  ⚠️  Error reading CSV: {e}", "yellow")
            # Try with more lenient parsing
            df = pd.read_csv(
                input_file,
                dtype={'asn': 'uint32'},
                on_bad_lines='skip',
                quoting=1,  # QUOTE_ALL
                encoding='utf-8',
                encoding_errors='ignore'
            )
        
        self.log(f"  Loaded: {len(df):,} records")
        
        # Clean and validate
        df = df.dropna(subset=['asn'])
        df['asn'] = df['asn'].astype('uint32')
        df['handle'] = df['handle'].fillna('').astype(str)
        df['description'] = df['description'].fillna('').astype(str)
        
        # Remove duplicates (keep first)
        df = df.drop_duplicates(subset=['asn'], keep='first')
        
        # Sort by ASN for efficient lookups
        self.log("  Sorting by ASN...")
        df = df.sort_values('asn').reset_index(drop=True)
        
        # Create Arrow table
        self.log("  Creating Arrow table...")
        schema = pa.schema([
            pa.field('asn', pa.uint32(), metadata={'description': 'ASN number'}),
            pa.field('handle', pa.utf8(), metadata={'description': 'ASN handle/identifier'}),
            pa.field('description', pa.utf8(), metadata={'description': 'Organization description'}),
        ])
        
        table = pa.Table.from_pandas(df, schema=schema)
        
        # Write Parquet
        self.log("  Writing Parquet file...")
        pq.write_table(
            table,
            output_file,
            compression='zstd',
            compression_level=9,
            use_dictionary=False,  # Text fields are unique
            write_statistics=True,
        )
        
        # Stats
        original_size = input_file.stat().st_size
        compressed_size = output_file.stat().st_size
        ratio = (1 - compressed_size / original_size) * 100
        
        self.stats['asn_metadata'] = {
            'records': len(df),
            'original_mb': original_size / 1024 / 1024,
            'compressed_mb': compressed_size / 1024 / 1024,
            'ratio': ratio,
        }
        
        self.log(f"  ✅ Created: {output_file.name}", "green")
        self.log(f"     Records: {len(df):,}", "dim")
        self.log(f"     Original: {original_size / 1024 / 1024:.1f} MB", "dim")
        self.log(f"     Compressed: {compressed_size / 1024 / 1024:.1f} MB ({ratio:.1f}% reduction)\n", "dim")
    
    def build_country_index(self):
        """Build country→ASN reverse index from geo data"""
        self.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", "dim")
        self.log("📊 Building Country Index", "yellow bold")
        self.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n", "dim")
        
        input_file = self.reference_dir / "asn-country-ipv4.csv"
        output_file = self.arrow_dir / "country-index.parquet"
        
        if not input_file.exists():
            self.log(f"⚠️  File not found: {input_file}", "yellow")
            return
        
        self.log(f"  Reading: {input_file.name}")
        
        # Read CSV
        df = pd.read_csv(
            input_file,
            names=['start_ip', 'end_ip', 'country'],
            dtype={'country': 'str'}
        )
        
        self.log(f"  Loaded: {len(df):,} records")
        
        # Convert IPs to integers for the IPv4 ranges
        self.log("  Converting IP addresses...")
        df['start_ip_int'] = df['start_ip'].apply(self.ipv4_to_int)
        df['end_ip_int'] = df['end_ip'].apply(self.ipv4_to_int)
        
        # Clean countries
        df['country'] = df['country'].fillna('').astype(str)
        df = df[df['country'] != '']
        
        # Sort by country then IP
        self.log("  Sorting and indexing...")
        df = df.sort_values(['country', 'start_ip_int']).reset_index(drop=True)
        
        # Create Arrow table
        self.log("  Creating Arrow table...")
        
        # Rename columns to match schema
        df_arrow = df[['country', 'start_ip_int', 'end_ip_int']].copy()
        df_arrow.columns = ['country', 'start_ip', 'end_ip']
        
        schema = pa.schema([
            pa.field('country', pa.dictionary(pa.uint8(), pa.utf8()), metadata={'description': 'ISO country code'}),
            pa.field('start_ip', pa.uint32(), metadata={'description': 'Start IP as uint32'}),
            pa.field('end_ip', pa.uint32(), metadata={'description': 'End IP as uint32'}),
        ])
        
        table = pa.Table.from_pandas(df_arrow, schema=schema)
        
        # Write Parquet
        self.log("  Writing Parquet file...")
        pq.write_table(
            table,
            output_file,
            compression='zstd',
            compression_level=9,
            use_dictionary=True,
            write_statistics=True,
        )
        
        # Stats
        original_size = input_file.stat().st_size
        compressed_size = output_file.stat().st_size
        ratio = (1 - compressed_size / original_size) * 100
        
        self.stats['country_index'] = {
            'records': len(df),
            'countries': df['country'].nunique(),
            'original_mb': original_size / 1024 / 1024,
            'compressed_mb': compressed_size / 1024 / 1024,
            'ratio': ratio,
        }
        
        self.log(f"  ✅ Created: {output_file.name}", "green")
        self.log(f"     Records: {len(df):,}", "dim")
        self.log(f"     Countries: {df['country'].nunique()}", "dim")
        self.log(f"     Compressed: {compressed_size / 1024 / 1024:.1f} MB ({ratio:.1f}% reduction)\n", "dim")
    
    def print_summary(self):
        """Print build summary"""
        self.log("\n" + "="*60, "bold blue")
        self.log("📊 Build Summary", "bold blue")
        self.log("="*60 + "\n", "bold blue")
        
        if RICH:
            table = Table(show_header=True, header_style="bold magenta")
            table.add_column("Database", style="cyan")
            table.add_column("Records", justify="right")
            table.add_column("Original", justify="right")
            table.add_column("Compressed", justify="right")
            table.add_column("Ratio", justify="right")
            
            for name, stats in self.stats.items():
                table.add_row(
                    name,
                    f"{stats['records']:,}",
                    f"{stats['original_mb']:.1f} MB",
                    f"{stats['compressed_mb']:.1f} MB",
                    f"{stats['ratio']:.1f}%"
                )
            
            # Total row
            total_records = sum(s['records'] for s in self.stats.values())
            total_original = sum(s['original_mb'] for s in self.stats.values())
            total_compressed = sum(s['compressed_mb'] for s in self.stats.values())
            total_ratio = (1 - total_compressed / total_original) * 100
            
            table.add_row(
                "TOTAL",
                f"{total_records:,}",
                f"{total_original:.1f} MB",
                f"{total_compressed:.1f} MB",
                f"{total_ratio:.1f}%",
                style="bold green"
            )
            
            console.print(table)
        else:
            for name, stats in self.stats.items():
                print(f"{name}:")
                print(f"  Records: {stats['records']:,}")
                print(f"  Original: {stats['original_mb']:.1f} MB")
                print(f"  Compressed: {stats['compressed_mb']:.1f} MB")
                print(f"  Ratio: {stats['ratio']:.1f}%")
                print()
        
        # File listing
        self.log("\n📁 Output Files:", "cyan bold")
        for file in sorted(self.arrow_dir.glob("*.parquet")):
            size_mb = file.stat().st_size / 1024 / 1024
            self.log(f"   {file.name:<30} {size_mb:>8.1f} MB", "dim")
    
    @staticmethod
    def ipv4_to_int(ip_str: str) -> int:
        """Convert IPv4 string to uint32"""
        try:
            return struct.unpack("!I", socket.inet_aton(ip_str))[0]
        except:
            return 0
    
    @staticmethod
    def ipv6_to_bytes(ip_str: str) -> bytes:
        """Convert IPv6 string to 16 bytes"""
        try:
            return socket.inet_pton(socket.AF_INET6, ip_str)
        except:
            return b'\x00' * 16


def main():
    """Main entry point"""
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    reference_dir = project_root / "reference_data"
    output_dir = project_root / "data"
    
    if not reference_dir.exists():
        print(f"❌ Reference data directory not found: {reference_dir}")
        sys.exit(1)
    
    builder = DatabaseBuilder(reference_dir, output_dir)
    builder.build_all()


if __name__ == '__main__':
    main()
