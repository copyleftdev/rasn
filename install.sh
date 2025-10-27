#!/usr/bin/env bash
set -e

# RASN Installation Script
# Installs binary and data files

PREFIX="${PREFIX:-/usr/local}"
BINDIR="$PREFIX/bin"
DATADIR="$PREFIX/share/rasn"

echo "==================================="
echo "RASN Installation"
echo "==================================="
echo ""
echo "Install location: $PREFIX"
echo "Binary: $BINDIR/rasn"
echo "Data:   $DATADIR"
echo ""

# Check for cargo
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found"
    echo "Install Rust: https://rustup.rs"
    exit 1
fi

# Build
echo "Building RASN..."
cargo build --release --bin rasn

# Install binary
echo ""
echo "Installing binary..."
mkdir -p "$BINDIR"
install -m 755 target/release/rasn "$BINDIR/rasn"
echo "✓ Binary installed: $BINDIR/rasn"

# Install data
echo ""
echo "Installing data files..."
mkdir -p "$DATADIR"

if [ -d "data" ]; then
    cp -r data/* "$DATADIR/" 2>/dev/null || true
    echo "✓ Data files installed"
fi

if [ -d "reference_data" ]; then
    mkdir -p "$DATADIR/reference"
    cp -r reference_data/* "$DATADIR/reference/" 2>/dev/null || true
    echo "✓ Reference data installed"
fi

# Create config
echo ""
echo "Creating configuration..."
mkdir -p ~/.config/rasn
cat > ~/.config/rasn/config.toml <<EOF
# RASN Configuration
data_dir = "$DATADIR"
EOF
echo "✓ Config created: ~/.config/rasn/config.toml"

# Success
echo ""
echo "==================================="
echo "✓ Installation complete!"
echo "==================================="
echo ""
echo "Add to your shell profile (~/.bashrc or ~/.zshrc):"
echo "  export RASN_DATA_DIR=$DATADIR"
echo ""
echo "Test installation:"
echo "  rasn --version"
echo "  rasn lookup 8.8.8.8"
echo ""
echo "Note: ASN data may need to be downloaded separately."
echo "Run: curl -L https://iptoasn.com/data/ip2asn-v4.tsv.gz | gunzip > $DATADIR/ip2asn-v4.tsv"
echo ""
