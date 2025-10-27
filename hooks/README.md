# Git Hooks

This directory contains git hooks that help maintain code quality.

## Installation

```bash
make install-hooks
# or
./scripts/install-hooks.sh
```

## Available Hooks

### pre-commit

Runs before every commit to ensure code quality:

1. **Format check** - `cargo fmt --all -- --check`
2. **Clippy** - `cargo clippy --all-features --workspace -- -D warnings`
3. **Tests** - `cargo test --all-features --workspace`
4. **Build** - `cargo build --release --bin rasn`
5. **TODO/FIXME check** - Warns about TODOs in staged files

## Skipping Hooks

If you need to skip hooks temporarily:

```bash
git commit --no-verify
```

**Note**: Only skip hooks when absolutely necessary. CI will still run all checks.

## Matching CI

These hooks mirror the CI pipeline in `.github/workflows/ci.yml` to catch issues before pushing.
