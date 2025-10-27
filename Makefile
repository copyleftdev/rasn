.PHONY: install uninstall data clean test help

# Default to user's home directory (no sudo required)
PREFIX ?= $(HOME)/.local
BINDIR = $(PREFIX)/bin
DATADIR = $(PREFIX)/share/rasn

help:
	@echo "RASN Installation"
	@echo ""
	@echo "Targets:"
	@echo "  make install       - Install binary and data files"
	@echo "  make install-bin   - Install binary only"
	@echo "  make install-data  - Install data files only"
	@echo "  make uninstall     - Remove all installed files"
	@echo "  make data          - Download/prepare data files"
	@echo "  make test          - Run tests"
	@echo "  make clean         - Clean build artifacts"
	@echo ""
	@echo "Configuration:"
	@echo "  PREFIX=$(PREFIX)   (default: ~/.local)"
	@echo ""
	@echo "System-wide install:"
	@echo "  sudo make install PREFIX=/usr/local"

# Build the binary
build:
	cargo build --release --bin rasn

# Install binary only
install-bin: build
	@echo "Installing binary to $(BINDIR)..."
	@mkdir -p $(BINDIR)
	@install -m 755 target/release/rasn $(BINDIR)/rasn
	@echo "✓ Binary installed to $(BINDIR)/rasn"

# Install data files
install-data:
	@echo "Installing data files to $(DATADIR)..."
	@mkdir -p $(DATADIR)
	@if [ -d "data" ]; then \
		cp -r data/* $(DATADIR)/ 2>/dev/null || true; \
		echo "✓ Data files installed to $(DATADIR)"; \
	else \
		echo "⚠ No data directory found - run 'make data' first"; \
	fi
	@if [ -d "reference_data" ]; then \
		mkdir -p $(DATADIR)/reference; \
		cp -r reference_data/* $(DATADIR)/reference/ 2>/dev/null || true; \
		echo "✓ Reference data installed"; \
	fi

# Full installation
install: install-bin install-data
	@echo ""
	@echo "✓ RASN installed successfully!"
	@echo ""
	@echo "Binary:     $(BINDIR)/rasn"
	@echo "Data:       $(DATADIR)"
	@echo ""
	@echo "Set environment variable:"
	@echo "  export RASN_DATA_DIR=$(DATADIR)"
	@echo ""
	@echo "Or add to ~/.bashrc or ~/.zshrc"

# Uninstall
uninstall:
	@echo "Removing RASN installation..."
	@rm -f $(BINDIR)/rasn
	@rm -rf $(DATADIR)
	@echo "✓ RASN uninstalled"

# Download/prepare data files
data:
	@echo "Downloading ASN data..."
	@mkdir -p data
	@echo "Downloading IP2ASN database..."
	@curl -L https://iptoasn.com/data/ip2asn-v4.tsv.gz -o data/ip2asn-v4.tsv.gz 2>/dev/null || \
		echo "⚠ Failed to download - you may need to download manually"
	@if [ -f data/ip2asn-v4.tsv.gz ]; then \
		gunzip -f data/ip2asn-v4.tsv.gz; \
		echo "✓ Data downloaded and extracted"; \
	fi
	@echo ""
	@echo "Note: For GeoIP data, download MaxMind GeoLite2 separately:"
	@echo "  https://dev.maxmind.com/geoip/geoip2/geolite2/"

# Run tests
test:
	cargo test --all-features --workspace

# Clean build artifacts
clean:
	cargo clean

# Development setup
dev-setup:
	@echo "Setting up development environment..."
	@rustup component add rustfmt clippy
	@echo "✓ Development tools installed"
	@echo ""
	@echo "Run: make test"
