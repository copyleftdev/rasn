# Installation Guide

## Quick Install

```bash
# Using install script (recommended - installs to ~/.local)
./install.sh

# Or using Makefile
make install
```

**Installs to your home directory** - no `sudo` required!

## What Gets Installed

1. **Binary**: `~/.local/bin/rasn`
2. **Data files**: `~/.local/share/rasn/`
   - ASN database (Arrow/Parquet)
   - Reference data
3. **Config**: `~/.config/rasn/config.toml`

### System-wide Install (optional)

```bash
sudo make install PREFIX=/usr/local
```

## Manual Installation

### 1. Build from source

```bash
cargo build --release
```

### 2. Install binary

```bash
# System-wide
sudo cp target/release/rasn /usr/local/bin/

# User-local
cp target/release/rasn ~/.local/bin/
```

### 3. Set up data directory

```bash
# Create data directory
mkdir -p ~/.local/share/rasn

# Download ASN data
curl -L https://iptoasn.com/data/ip2asn-v4.tsv.gz | \
  gunzip > ~/.local/share/rasn/ip2asn-v4.tsv
```

### 4. Configure

```bash
# Add to ~/.bashrc or ~/.zshrc
export RASN_DATA_DIR=~/.local/share/rasn
```

## Data Sources

### Required: ASN Database

```bash
# IP to ASN mapping (required for lookups)
curl -L https://iptoasn.com/data/ip2asn-v4.tsv.gz -o ip2asn-v4.tsv.gz
gunzip ip2asn-v4.tsv.gz
```

### Optional: GeoIP Database

```bash
# Download MaxMind GeoLite2
# Sign up at: https://dev.maxmind.com/geoip/geoip2/geolite2/

# Extract to data directory
cp GeoLite2-City.mmdb $RASN_DATA_DIR/
```

## Docker Installation

No data setup needed - included in image:

```bash
docker pull ghcr.io/copyleftdev/rasn:latest
docker run --rm rasn:latest lookup 8.8.8.8
```

## Verification

```bash
# Check installation
rasn --version

# Test with demo data
rasn lookup 8.8.8.8

# Check data directory
rasn auth info
```

## Troubleshooting

### "No data found"

Set the data directory:
```bash
export RASN_DATA_DIR=/usr/local/share/rasn
```

### Binary not found after install

Add `~/.local/bin` to your PATH:

```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.local/bin:$PATH"

# Reload shell
source ~/.bashrc
```

### Missing dependencies

Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Uninstall

```bash
# Using Makefile
make uninstall

# Manual
rm /usr/local/bin/rasn
rm -rf /usr/local/share/rasn
rm -rf ~/.config/rasn
```
