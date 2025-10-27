# Cargo Development Workflow

## Quick Reference Commands

### Daily Development
```bash
# Check code compiles
cargo check --all-targets

# Run tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run linter
cargo clippy

# Format code
cargo fmt
```

### Pre-Commit (MANDATORY)
```bash
# Full quality check
./scripts/pre-commit-check.sh

# Or manually:
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

### Performance
```bash
# Run benchmarks
cargo bench

# Profile with flamegraph
cargo flamegraph --bin rasn -- lookup 8.8.8.8

# Check binary size
cargo bloat --release

# Assembly inspection
cargo asm rasn_arrow::find_ip
```

### Dependency Management
```bash
# Update dependencies
cargo update

# Check outdated packages
cargo outdated

# Security audit
cargo audit

# Tree of dependencies
cargo tree
```

### Documentation
```bash
# Build docs
cargo doc --open

# Build with private items
cargo doc --document-private-items

# Check doc links
cargo doc --no-deps
```

### Release
```bash
# Build optimized binary
cargo build --release

# Run release tests
cargo test --release

# Package crate
cargo package

# Dry run publish
cargo publish --dry-run
```

---

## Cargo.toml Best Practices

### Workspace Root
```toml
[workspace]
members = [
    "crates/rasn-cli",
    "crates/rasn-core",
    "crates/rasn-arrow",
    # ... all crates
]

resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"  # MSRV
authors = ["Your Name <email@example.com>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/copyleftdev/rasn"

[workspace.dependencies]
# Centralize version management
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
anyhow = "1.0"

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true
panic = "abort"

[profile.bench]
inherits = "release"
```

### Individual Crate
```toml
[package]
name = "rasn-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Use workspace dependencies
tokio.workspace = true
serde.workspace = true
thiserror.workspace = true

# Crate-specific dependencies
arrow = "51.0"

[dev-dependencies]
criterion = "0.5"
proptest = "1.4"

[[bench]]
name = "lookup_bench"
harness = false
```

---

## Testing Patterns

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_case() {
        let result = function(input);
        assert_eq!(result, expected);
    }

    #[test]
    #[should_panic(expected = "error message")]
    fn test_error_case() {
        function_that_panics();
    }

    #[test]
    fn test_result_ok() {
        let result = fallible_function();
        assert!(result.is_ok());
    }
}
```

### Integration Tests
```rust
// tests/integration_test.rs
use rasn_core::*;

#[test]
fn test_end_to_end_lookup() {
    let ip = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
    let result = lookup_ip(ip).expect("lookup failed");
    assert_eq!(result.asn, Asn(15169));
}
```

### Property-Based Tests
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_roundtrip(ip in any::<u32>()) {
        let encoded = encode_ip(ip);
        let decoded = decode_ip(&encoded);
        assert_eq!(ip, decoded);
    }
}
```

### Benchmarks
```rust
// benches/lookup_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_lookup(c: &mut Criterion) {
    let table = load_test_table();
    
    c.bench_function("lookup_ip", |b| {
        b.iter(|| table.find_ip(black_box(0x08080808)))
    });
}

criterion_group!(benches, bench_lookup);
criterion_main!(benches);
```

---

## Common Cargo Commands

### Build Commands
```bash
cargo build                  # Debug build
cargo build --release        # Optimized build
cargo build --all-targets    # Build all targets (bins, tests, benches)
cargo build --features feat  # With specific features
cargo build --no-default-features  # Minimal build
```

### Test Commands
```bash
cargo test                   # Run all tests
cargo test --doc            # Run doc tests only
cargo test --lib            # Run lib tests only
cargo test --bin binary     # Run binary tests
cargo test pattern          # Run tests matching pattern
cargo test -- --ignored     # Run ignored tests
cargo test -- --test-threads=1  # Single-threaded
```

### Clean Commands
```bash
cargo clean                  # Clean all artifacts
cargo clean -p crate-name   # Clean specific package
```

### Maintenance Commands
```bash
cargo fix                    # Auto-fix compiler warnings
cargo clippy --fix          # Auto-fix clippy warnings
cargo update                # Update dependencies
cargo vendor                # Vendor dependencies locally
```

---

## Feature Flags

```toml
[features]
default = ["std"]
std = []
simd = []
mcp = ["dep:jsonrpc-core"]
```

```bash
# Build with features
cargo build --features simd
cargo build --all-features
cargo build --no-default-features --features std,mcp
```

---

## Environment Variables

```bash
# Increase compiler parallelism
export CARGO_BUILD_JOBS=8

# Use lld linker (faster)
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"

# Colored output
export CARGO_TERM_COLOR=always

# Build cache
export CARGO_TARGET_DIR=/tmp/cargo-target
```

---

## CI Integration

### GitHub Actions Workflow
```yaml
name: CI

on: [push, pull_request]

env:
  CARGO_TERM_COLOR: always

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      
      - name: Cache
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Format
        run: cargo fmt --all -- --check
      
      - name: Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings
      
      - name: Test
        run: cargo test --all-features
      
      - name: Doc
        run: cargo doc --no-deps
```

---

## Troubleshooting

### Common Issues

**Issue: Compilation errors after `git pull`**
```bash
cargo clean
cargo update
cargo build
```

**Issue: Tests fail locally but pass in CI**
```bash
# Ensure clean state
cargo clean
rm -rf target
cargo test
```

**Issue: Slow compilation**
```bash
# Use faster linker
sudo apt install lld  # Ubuntu
brew install llvm      # macOS

# Add to ~/.cargo/config.toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

**Issue: Out of disk space**
```bash
# Clean old artifacts
cargo clean

# Remove old versions
cargo install cargo-cache
cargo cache --autoclean
```
