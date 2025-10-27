# Contributing to RASN

## Development Workflow

### Before Every Commit

**Always run the pre-commit checks locally:**

```bash
./scripts/check.sh
```

Or manually run each step:

```bash
# 1. Build
cargo build --all-features

# 2. Format check
cargo fmt --all -- --check

# 3. Clippy (linting)
cargo clippy --all-targets --all-features -- -D warnings

# 4. Tests
cargo test --all-features

# 5. Documentation
cargo doc --no-deps --all-features
```

### Auto-formatting

To automatically fix formatting issues:

```bash
cargo fmt --all
```

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench arrow_lookup
```

### Quality Standards

- **Zero warnings**: All code must compile without warnings
- **Zero clippy violations**: Fix all clippy suggestions
- **All tests passing**: 100% test success required
- **Formatted code**: Run `cargo fmt` before committing
- **Documentation**: Public APIs must have doc comments

### Git Workflow

1. Create feature branch: `git checkout -b feature/issue-X-description`
2. Make changes
3. **Run checks locally**: `./scripts/check.sh`
4. Commit: `git commit -m "feat(scope): description"`
5. Push: `git push origin feature/issue-X-description`
6. Create PR
7. Wait for CI to pass
8. Merge with squash

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

Fixes #<issue-number>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Test additions/changes
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks

**Examples:**
```
feat(arrow): add SIMD AVX2 acceleration
fix(db): correct RocksDB API usage
docs(readme): update installation instructions
test(resolver): add cache expiry tests
```

## Running CI Checks Locally

Our CI runs on:
- Ubuntu (Linux)
- macOS
- Windows

All checks must pass on your local machine before pushing.

## Need Help?

- Check existing issues
- Read the documentation in `/docs`
- Ask questions in discussions
