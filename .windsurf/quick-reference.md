# RASN Development Quick Reference

## 🚀 Starting a New Task

```bash
# 1. Pull latest
git checkout main && git pull origin main

# 2. Create feature branch
git checkout -b feature/issue-N-description

# 3. Read GitHub issue thoroughly
gh issue view N
```

## 💻 Development Loop

```bash
# Make changes...

# Quick check
cargo check

# Run tests
cargo test

# Format & lint
cargo fmt && cargo clippy
```

## ✅ Pre-Commit (MANDATORY)

```bash
# Run full check
./scripts/pre-commit-check.sh

# If fails, fix and repeat
```

## 📝 Committing

```bash
# Stage changes
git add .

# Commit with conventional format
git commit -m "feat(core): implement Asn type

- Add newtype wrapper around u32
- Implement serde traits
- Add unit tests

Fixes #1"
```

## 🔄 Creating PR

```bash
# Push branch
git push origin feature/issue-N-description

# Create PR
gh pr create --title "Issue #N: Title" --body "Closes #N"

# Wait for CI to pass
gh pr checks

# Merge when approved
gh pr merge --squash --delete-branch
```

## 🏁 After Merge

```bash
# Switch back to main
git checkout main

# Pull merged changes
git pull origin main

# Verify issue closed
gh issue view N
```

## 🔧 Common Commands

### Cargo
```bash
cargo build              # Compile
cargo test               # Run tests
cargo clippy             # Lint
cargo fmt                # Format
cargo doc --open         # Build & open docs
cargo bench              # Run benchmarks
```

### Git
```bash
git status               # Check status
git diff                 # Show changes
git log --oneline -10    # Recent commits
git branch -a            # List branches
git stash                # Stash changes
git stash pop            # Restore stash
```

### GitHub CLI
```bash
gh issue list            # List issues
gh issue view N          # View issue
gh pr list               # List PRs
gh pr status             # PR status
gh pr checks             # CI status
```

## 🎯 Acceptance Criteria Checklist

Before marking complete:
- [ ] All issue checkboxes ticked
- [ ] `cargo test` passes
- [ ] `cargo clippy` has no warnings
- [ ] `cargo fmt` applied
- [ ] Documentation updated
- [ ] Performance targets met
- [ ] Pre-commit check passes

## 🐛 Quick Fixes

```bash
# Auto-fix formatting
cargo fmt --all

# Auto-fix some clippy issues
cargo clippy --fix

# Auto-fix compiler warnings
cargo fix

# Clean build artifacts
cargo clean
```

## 📊 Performance Check

```bash
# Build optimized
cargo build --release

# Run benchmarks
cargo bench

# Profile (if flamegraph installed)
cargo flamegraph --bin rasn -- lookup 8.8.8.8

# Check binary size
cargo bloat --release
```

## 🔍 Debugging

```bash
# Run with debug output
RUST_LOG=debug cargo run -- lookup 8.8.8.8

# Run specific test with output
cargo test test_name -- --nocapture

# Backtrace on panic
RUST_BACKTRACE=1 cargo test
```

## 📚 Documentation

```bash
# Build docs
cargo doc

# Build with private items
cargo doc --document-private-items

# Open in browser
cargo doc --open
```
