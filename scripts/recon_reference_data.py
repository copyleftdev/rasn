#!/usr/bin/env python3
"""
Reference Data Reconnaissance Script
Analyzes ASN and geolocation data files to design optimal RocksDB schema
"""

import os
import sys
import gzip
import json
import pandas as pd
import numpy as np
from pathlib import Path
from datetime import datetime
from collections import Counter, defaultdict
import re

# Rich output if available
try:
    from rich.console import Console
    from rich.table import Table
    from rich.progress import track
    from rich.panel import Panel
    console = Console()
    RICH_AVAILABLE = True
except ImportError:
    RICH_AVAILABLE = False
    print("💡 Tip: Install 'rich' for better output: pip install rich")

class DataRecon:
    def __init__(self, reference_dir: Path):
        self.reference_dir = reference_dir
        self.report = {
            'timestamp': datetime.now().isoformat(),
            'files': {},
            'recommendations': {}
        }
        
    def log(self, message: str, style: str = ""):
        """Log with rich formatting if available"""
        if RICH_AVAILABLE:
            console.print(message, style=style)
        else:
            print(message)
    
    def analyze_all(self):
        """Main analysis entry point"""
        self.log("\n╔════════════════════════════════════════════════════════════════╗", "bold blue")
        self.log("║         RASN Reference Data Reconnaissance                     ║", "bold blue")
        self.log("╚════════════════════════════════════════════════════════════════╝\n", "bold blue")
        
        self.log(f"📂 Directory: {self.reference_dir}", "cyan")
        self.log(f"🕐 Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n", "cyan")
        
        # Find all data files
        files = self._find_data_files()
        
        if not files:
            self.log("❌ No data files found!", "red bold")
            return
        
        self.log(f"Found {len(files)} data files\n", "green")
        
        # Analyze each file
        for file_path in files:
            self._analyze_file(file_path)
        
        # Generate recommendations
        self._generate_recommendations()
        
        # Save report
        self._save_report()
        
        self.log("\n✅ Reconnaissance complete!", "green bold")
        
    def _find_data_files(self) -> list:
        """Find all analyzable data files"""
        patterns = ['*.tsv', '*.csv', '*.tsv.gz', '*.csv.gz', '*.mmdb']
        files = []
        
        for pattern in patterns:
            files.extend(self.reference_dir.glob(pattern))
        
        return sorted(files)
    
    def _analyze_file(self, file_path: Path):
        """Analyze a single file"""
        filename = file_path.name
        self.log(f"\n{'='*70}", "dim")
        self.log(f"📄 Analyzing: {filename}", "bold yellow")
        self.log(f"{'='*70}", "dim")
        
        file_info = {
            'name': filename,
            'path': str(file_path),
            'size_bytes': file_path.stat().st_size,
            'size_human': self._human_size(file_path.stat().st_size),
        }
        
        # Determine file type and analyze
        if filename.endswith('.gz'):
            file_info['compressed'] = True
            # Check if uncompressed version exists
            uncompressed = file_path.with_suffix('')
            if uncompressed.exists():
                file_info['uncompressed_available'] = True
                self._analyze_tabular(uncompressed, file_info)
            else:
                self.log("  Decompressing for analysis...", "dim")
                self._analyze_compressed(file_path, file_info)
        elif filename.endswith('.tsv'):
            file_info['compressed'] = False
            self._analyze_tabular(file_path, file_info, delimiter='\t')
        elif filename.endswith('.csv'):
            file_info['compressed'] = False
            self._analyze_tabular(file_path, file_info, delimiter=',')
        elif filename.endswith('.mmdb'):
            self._analyze_mmdb(file_path, file_info)
        
        self.report['files'][filename] = file_info
    
    def _analyze_compressed(self, file_path: Path, file_info: dict):
        """Analyze compressed file"""
        # Determine delimiter from extension
        if '.tsv' in file_path.name:
            delimiter = '\t'
        else:
            delimiter = ','
        
        with gzip.open(file_path, 'rt') as f:
            # Read first chunk for analysis
            lines = [next(f) for _ in range(min(1000, sum(1 for _ in f)))]
            
        # Create temporary dataframe
        from io import StringIO
        df = pd.read_csv(StringIO('\n'.join(lines)), 
                        delimiter=delimiter, 
                        header=None, 
                        on_bad_lines='skip')
        
        self._analyze_dataframe(df, file_info, sample_only=True)
    
    def _analyze_tabular(self, file_path: Path, file_info: dict, delimiter: str = '\t'):
        """Analyze TSV/CSV file with pandas"""
        try:
            # Try reading with header first
            df_with_header = pd.read_csv(file_path, delimiter=delimiter, nrows=5)
            
            # Check if first row looks like header (all strings, no numbers)
            first_row = df_with_header.iloc[0] if len(df_with_header) > 0 else None
            has_header = False
            
            if first_row is not None:
                # If first row values are not numeric and look like names
                if df_with_header.columns[0].replace('_', '').isalpha():
                    has_header = True
            
            if has_header:
                # Read with header
                df = pd.read_csv(file_path, delimiter=delimiter, nrows=10000)
                file_info['has_header'] = True
                file_info['columns'] = list(df.columns)
            else:
                # Read without header
                df = pd.read_csv(file_path, delimiter=delimiter, header=None, nrows=10000)
                file_info['has_header'] = False
                file_info['columns'] = [f'col_{i}' for i in range(len(df.columns))]
                df.columns = file_info['columns']
            
            file_info['total_rows'] = len(df)
            
            # Get total row count efficiently
            with open(file_path, 'r') as f:
                file_info['total_rows_actual'] = sum(1 for _ in f)
            
            self._analyze_dataframe(df, file_info)
            
        except Exception as e:
            self.log(f"  ⚠️  Error analyzing file: {e}", "red")
            file_info['error'] = str(e)
    
    def _analyze_dataframe(self, df: pd.DataFrame, file_info: dict, sample_only: bool = False):
        """Deep analysis of pandas DataFrame"""
        
        # Basic stats
        self.log(f"  📊 Shape: {df.shape[0]} rows × {df.shape[1]} columns", "cyan")
        if sample_only:
            self.log(f"  ℹ️  Note: Sample analysis (first 1000 rows)", "dim")
        
        # Column analysis
        columns_info = []
        
        for col in df.columns:
            col_info = self._analyze_column(df[col], col)
            columns_info.append(col_info)
        
        file_info['columns_analysis'] = columns_info
        
        # Display table
        self._display_column_table(columns_info)
        
        # Memory usage
        memory_mb = df.memory_usage(deep=True).sum() / (1024 * 1024)
        file_info['memory_usage_mb'] = round(memory_mb, 2)
        self.log(f"\n  💾 Memory Usage: {memory_mb:.2f} MB", "magenta")
        
        # Data quality checks
        self._check_data_quality(df, file_info)
        
        # Sample rows
        self.log(f"\n  🔍 Sample Rows (first 3):", "yellow")
        if RICH_AVAILABLE:
            sample_table = Table(show_header=True)
            for col in df.columns:
                sample_table.add_column(str(col)[:20])
            for _, row in df.head(3).iterrows():
                sample_table.add_row(*[str(v)[:30] for v in row])
            console.print(sample_table)
        else:
            print(df.head(3).to_string())
    
    def _analyze_column(self, series: pd.Series, col_name: str) -> dict:
        """Analyze a single column"""
        col_info = {
            'name': str(col_name),
            'dtype': str(series.dtype),
            'null_count': int(series.isnull().sum()),
            'null_percent': round(series.isnull().sum() / len(series) * 100, 2),
            'unique_count': int(series.nunique()),
            'sample_values': []
        }
        
        # Sample values (non-null)
        non_null = series.dropna()
        if len(non_null) > 0:
            col_info['sample_values'] = [str(v) for v in non_null.head(3).tolist()]
        
        # Detect data type
        col_info['detected_type'] = self._detect_type(series)
        
        # Stats for numeric columns
        if pd.api.types.is_numeric_dtype(series):
            col_info['min'] = float(series.min()) if not series.empty else None
            col_info['max'] = float(series.max()) if not series.empty else None
            col_info['mean'] = float(series.mean()) if not series.empty else None
        
        # Length stats for string columns
        if pd.api.types.is_string_dtype(series) or series.dtype == 'object':
            lengths = series.astype(str).str.len()
            col_info['avg_length'] = round(lengths.mean(), 1) if not lengths.empty else None
            col_info['max_length'] = int(lengths.max()) if not lengths.empty else None
        
        return col_info
    
    def _detect_type(self, series: pd.Series) -> str:
        """Detect semantic type of column"""
        sample = series.dropna().astype(str).head(100)
        
        if len(sample) == 0:
            return 'empty'
        
        # Patterns
        ipv4_pattern = re.compile(r'^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$')
        ipv6_pattern = re.compile(r'^[0-9a-fA-F:]+$')
        asn_pattern = re.compile(r'^(AS)?\d+$')
        country_pattern = re.compile(r'^[A-Z]{2}$')
        
        # Check patterns
        ipv4_count = sum(1 for v in sample if ipv4_pattern.match(str(v)))
        ipv6_count = sum(1 for v in sample if ipv6_pattern.match(str(v)))
        asn_count = sum(1 for v in sample if asn_pattern.match(str(v)))
        country_count = sum(1 for v in sample if country_pattern.match(str(v)))
        
        # Determine type
        if ipv4_count / len(sample) > 0.8:
            return 'ipv4'
        elif ipv6_count / len(sample) > 0.8:
            return 'ipv6'
        elif asn_count / len(sample) > 0.8:
            return 'asn'
        elif country_count / len(sample) > 0.8:
            return 'country_code'
        elif pd.api.types.is_numeric_dtype(series):
            return 'numeric'
        else:
            return 'string'
    
    def _display_column_table(self, columns_info: list):
        """Display column analysis table"""
        self.log("\n  📋 Column Details:", "green")
        
        if RICH_AVAILABLE:
            table = Table(show_header=True, header_style="bold magenta")
            table.add_column("#", style="dim")
            table.add_column("Column")
            table.add_column("Type", style="cyan")
            table.add_column("Detected", style="yellow")
            table.add_column("Nulls", justify="right")
            table.add_column("Unique", justify="right")
            table.add_column("Sample")
            
            for i, col in enumerate(columns_info, 1):
                table.add_row(
                    str(i),
                    col['name'][:20],
                    col['dtype'],
                    col['detected_type'],
                    f"{col['null_count']} ({col['null_percent']}%)",
                    str(col['unique_count']),
                    col['sample_values'][0][:30] if col['sample_values'] else "N/A"
                )
            
            console.print(table)
        else:
            for i, col in enumerate(columns_info, 1):
                print(f"    {i}. {col['name']}: {col['detected_type']} "
                      f"(nulls: {col['null_percent']}%, unique: {col['unique_count']})")
    
    def _check_data_quality(self, df: pd.DataFrame, file_info: dict):
        """Check data quality issues"""
        issues = []
        
        # Check for duplicates
        dup_count = df.duplicated().sum()
        if dup_count > 0:
            issues.append(f"Found {dup_count} duplicate rows ({dup_count/len(df)*100:.1f}%)")
        
        # Check for missing values
        null_cols = df.columns[df.isnull().any()].tolist()
        if null_cols:
            issues.append(f"Columns with nulls: {', '.join(null_cols[:5])}")
        
        # Check for empty strings
        for col in df.select_dtypes(include=['object']).columns:
            empty_count = (df[col].astype(str).str.strip() == '').sum()
            if empty_count > len(df) * 0.01:  # More than 1%
                issues.append(f"Column '{col}' has {empty_count} empty strings")
        
        file_info['quality_issues'] = issues
        
        if issues:
            self.log(f"\n  ⚠️  Quality Issues:", "yellow bold")
            for issue in issues:
                self.log(f"    • {issue}", "yellow")
        else:
            self.log(f"\n  ✅ No major quality issues detected", "green")
    
    def _analyze_mmdb(self, file_path: Path, file_info: dict):
        """Analyze MMDB file"""
        file_info['format'] = 'MaxMind DB (MMDB)'
        
        self.log("  📦 Binary MaxMind DB format", "cyan")
        self.log("  ℹ️  Requires maxminddb library for detailed analysis", "dim")
        
        try:
            import maxminddb
            
            reader = maxminddb.open_database(str(file_path))
            file_info['mmdb_metadata'] = {
                'database_type': reader.metadata().database_type,
                'ip_version': reader.metadata().ip_version,
                'node_count': reader.metadata().node_count,
                'record_size': reader.metadata().record_size,
            }
            
            self.log(f"  • Database Type: {reader.metadata().database_type}", "green")
            self.log(f"  • IP Version: {reader.metadata().ip_version}", "green")
            self.log(f"  • Node Count: {reader.metadata().node_count:,}", "green")
            
            reader.close()
        except ImportError:
            self.log("  💡 Install maxminddb for detailed analysis: pip install maxminddb", "yellow")
        except Exception as e:
            self.log(f"  ⚠️  Error reading MMDB: {e}", "red")
    
    def _generate_recommendations(self):
        """Generate RocksDB schema recommendations based on analysis"""
        self.log("\n" + "="*70, "bold blue")
        self.log("🎯 RocksDB Schema Recommendations", "bold blue")
        self.log("="*70 + "\n", "bold blue")
        
        recommendations = {
            'key_patterns': [],
            'column_families': [],
            'optimization_strategies': []
        }
        
        # Analyze files and suggest schema
        for filename, info in self.report['files'].items():
            if 'ip2asn' in filename:
                recommendations['key_patterns'].append({
                    'name': 'IP to ASN Lookup',
                    'key_format': 'ip:{ip_address}',
                    'value_type': 'IpInfo struct (asn, country, org)',
                    'source_file': filename,
                    'estimated_keys': info.get('total_rows_actual', 'unknown')
                })
            
            elif 'asn-info' in filename:
                recommendations['key_patterns'].append({
                    'name': 'ASN Metadata',
                    'key_format': 'asn:{number}',
                    'value_type': 'AsnInfo struct (handle, description)',
                    'source_file': filename,
                    'estimated_keys': info.get('total_rows_actual', 'unknown')
                })
            
            elif 'country' in filename:
                recommendations['key_patterns'].append({
                    'name': 'Country Lookups',
                    'key_format': 'country:{code}:{asn}',
                    'value_type': 'Empty (use key existence)',
                    'source_file': filename,
                    'estimated_keys': info.get('total_rows_actual', 'unknown')
                })
        
        # Column families
        recommendations['column_families'] = [
            {'name': 'ip_ranges', 'purpose': 'IP to ASN mappings', 'compression': 'LZ4'},
            {'name': 'asn_metadata', 'purpose': 'ASN information', 'compression': 'Snappy'},
            {'name': 'indexes', 'purpose': 'Reverse indexes (org→asn, country→asn)', 'compression': 'Snappy'},
        ]
        
        # Optimizations
        recommendations['optimization_strategies'] = [
            'Use prefix bloom filters for IP range lookups',
            'Enable block cache (256MB recommended)',
            'Use column families to separate hot/cold data',
            'Compress values with MessagePack or Bincode',
            'Batch writes during import (10k records per batch)',
            'Use memtable size of 128MB for import',
        ]
        
        self.report['recommendations'] = recommendations
        
        # Display recommendations
        self._display_recommendations(recommendations)
    
    def _display_recommendations(self, recommendations: dict):
        """Display recommendations"""
        self.log("📌 Key Patterns:", "cyan bold")
        for pattern in recommendations['key_patterns']:
            self.log(f"\n  • {pattern['name']}", "yellow")
            self.log(f"    Key: {pattern['key_format']}", "dim")
            self.log(f"    Value: {pattern['value_type']}", "dim")
            self.log(f"    Source: {pattern['source_file']}", "dim")
            self.log(f"    Est. Keys: {pattern['estimated_keys']}", "dim")
        
        self.log("\n📁 Column Families:", "cyan bold")
        for cf in recommendations['column_families']:
            self.log(f"  • {cf['name']}: {cf['purpose']} ({cf['compression']})", "green")
        
        self.log("\n⚡ Optimization Strategies:", "cyan bold")
        for strategy in recommendations['optimization_strategies']:
            self.log(f"  • {strategy}", "green")
    
    def _save_report(self):
        """Save report to JSON and Markdown"""
        output_dir = Path(__file__).parent
        
        # Save JSON
        json_path = output_dir / 'recon_report.json'
        with open(json_path, 'w') as f:
            json.dump(self.report, f, indent=2, default=str)
        self.log(f"\n💾 JSON report saved: {json_path}", "green")
        
        # Save Markdown
        md_path = output_dir / 'recon_report.md'
        self._generate_markdown_report(md_path)
        self.log(f"📄 Markdown report saved: {md_path}", "green")
    
    def _generate_markdown_report(self, output_path: Path):
        """Generate markdown report"""
        with open(output_path, 'w') as f:
            f.write(f"# Reference Data Reconnaissance Report\n\n")
            f.write(f"**Generated:** {self.report['timestamp']}  \n")
            f.write(f"**Directory:** {self.reference_dir}\n\n")
            f.write(f"---\n\n")
            
            # Files section
            f.write(f"## Analyzed Files\n\n")
            for filename, info in self.report['files'].items():
                f.write(f"### {filename}\n\n")
                f.write(f"- **Size:** {info['size_human']}\n")
                if 'total_rows_actual' in info:
                    f.write(f"- **Rows:** {info['total_rows_actual']:,}\n")
                if 'has_header' in info:
                    f.write(f"- **Header:** {'Yes' if info['has_header'] else 'No'}\n")
                
                if 'columns_analysis' in info:
                    f.write(f"\n**Columns:**\n\n")
                    f.write(f"| # | Name | Type | Detected | Nulls | Unique |\n")
                    f.write(f"|---|------|------|----------|-------|--------|\n")
                    for i, col in enumerate(info['columns_analysis'], 1):
                        f.write(f"| {i} | {col['name']} | {col['dtype']} | {col['detected_type']} | ")
                        f.write(f"{col['null_percent']}% | {col['unique_count']} |\n")
                
                f.write(f"\n")
            
            # Recommendations section
            f.write(f"## RocksDB Schema Recommendations\n\n")
            recs = self.report['recommendations']
            
            f.write(f"### Key Patterns\n\n")
            for pattern in recs['key_patterns']:
                f.write(f"**{pattern['name']}**\n")
                f.write(f"- Key Format: `{pattern['key_format']}`\n")
                f.write(f"- Value Type: {pattern['value_type']}\n")
                f.write(f"- Source: {pattern['source_file']}\n\n")
            
            f.write(f"### Column Families\n\n")
            for cf in recs['column_families']:
                f.write(f"- **{cf['name']}**: {cf['purpose']} (Compression: {cf['compression']})\n")
            
            f.write(f"\n### Optimization Strategies\n\n")
            for strategy in recs['optimization_strategies']:
                f.write(f"- {strategy}\n")
    
    @staticmethod
    def _human_size(size_bytes: int) -> str:
        """Convert bytes to human readable format"""
        for unit in ['B', 'KB', 'MB', 'GB']:
            if size_bytes < 1024.0:
                return f"{size_bytes:.1f} {unit}"
            size_bytes /= 1024.0
        return f"{size_bytes:.1f} TB"


def main():
    """Main entry point"""
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    reference_dir = project_root / 'reference_data'
    
    if not reference_dir.exists():
        print(f"❌ Reference data directory not found: {reference_dir}")
        sys.exit(1)
    
    recon = DataRecon(reference_dir)
    recon.analyze_all()


if __name__ == '__main__':
    main()
