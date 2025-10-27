#!/usr/bin/env bash
set -e

echo "🔍 Running pre-commit checks..."
echo ""

echo "📦 Building all packages..."
cargo build --all-features
echo "✅ Build successful"
echo ""

echo "🧹 Checking formatting..."
cargo fmt --all -- --check
echo "✅ Formatting check passed"
echo ""

echo "📎 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings
echo "✅ Clippy passed"
echo ""

echo "🧪 Running all tests..."
cargo test --all-features
echo "✅ All tests passed"
echo ""

echo "📚 Building documentation..."
cargo doc --no-deps --all-features
echo "✅ Documentation built"
echo ""

echo "✨ All checks passed! Safe to commit and push."
