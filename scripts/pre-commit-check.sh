#!/bin/bash
# Pre-commit quality check script
# Run this before every commit to ensure code quality

set -e  # Exit on first error

echo "🔍 Running pre-commit checks..."
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track failures
FAILED=0

# Function to run check
run_check() {
    local name=$1
    local cmd=$2
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Running: $name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    if eval "$cmd"; then
        echo -e "${GREEN}✅ $name PASSED${NC}"
    else
        echo -e "${RED}❌ $name FAILED${NC}"
        FAILED=1
    fi
    echo ""
}

# 1. Format Check
run_check "Code Formatting" "cargo fmt --all -- --check"

# 2. Clippy (strict mode)
run_check "Clippy Linting" "cargo clippy --all-targets --all-features -- -D warnings"

# 3. Unit Tests
run_check "Unit Tests" "cargo test --all-features"

# 4. Documentation Build
run_check "Documentation" "cargo doc --no-deps --all-features 2>&1 | grep -v 'warning: unused'"

# 5. Benchmark Compilation
run_check "Benchmark Compilation" "cargo bench --no-run 2>&1 | grep -v 'warning'"

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All checks passed! Ready to commit.${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 0
else
    echo -e "${RED}❌ Some checks failed. Please fix issues before committing.${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 1
fi
