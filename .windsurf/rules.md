# Windsurf Development Rules for RASN

## Git Workflow Rules

### Branch Strategy
- **ALWAYS** create a feature branch for each GitHub issue
- Branch naming: `feature/issue-{number}-{short-description}`
  - Example: `feature/issue-1-cargo-workspace`
- **NEVER** commit directly to `main`
- One issue per branch (no mixing tasks)

### Development Workflow

#### 1. Start New Task
```bash
# Pull latest from main
git checkout main
git pull origin main

# Create feature branch from issue
git checkout -b feature/issue-{N}-{description}
```

#### 2. Development Cycle
- Read the GitHub issue acceptance criteria thoroughly
- Implement changes incrementally
- Run tests after each logical change
- Commit early and often with descriptive messages

#### 3. Pre-Commit Checklist
**MANDATORY - Must pass ALL before committing:**

```bash
# Format code
cargo fmt --all

# Run linter (must have zero warnings)
cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
cargo test --all-features

# Check documentation
cargo doc --no-deps --document-private-items

# Run benchmarks (if applicable)
cargo bench --no-run

# Security audit (periodically)
cargo audit
```

#### 4. Commit Standards
- Use conventional commits format:
  ```
  type(scope): subject
  
  body (optional)
  
  Fixes #issue-number
  ```

- **Types:**
  - `feat`: New feature
  - `fix`: Bug fix
  - `docs`: Documentation only
  - `style`: Code style (formatting, no logic change)
  - `refactor`: Code refactoring
  - `perf`: Performance improvement
  - `test`: Adding/updating tests
  - `chore`: Build process, dependencies

- **Examples:**
  ```
  feat(core): implement Asn and IpAddr types
  
  - Add Asn newtype wrapper around u32
  - Add IpAddr enum for IPv4/IPv6
  - Implement serde traits
  
  Fixes #1
  ```

#### 5. Acceptance Criteria Verification
**Before marking issue complete:**
- [ ] All checkboxes in issue are ticked
- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation is updated
- [ ] Performance targets met (if applicable)
- [ ] No new compiler warnings

#### 6. Merge Process
```bash
# Ensure all tests pass
cargo test --all-features

# Ensure clean linting
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all

# Push to remote
git push origin feature/issue-{N}-{description}

# Create PR on GitHub
gh pr create --title "Issue #{N}: {Title}" --body "Closes #N"

# After PR approval, merge
gh pr merge --squash --delete-branch

# Switch back to main and pull
git checkout main
git pull origin main
```

#### 7. Issue Closure
- Use GitHub keywords in commit/PR: `Fixes #N`, `Closes #N`, `Resolves #N`
- Verify issue auto-closes after merge
- Manually close if needed with summary comment

---

## Rust/Cargo Best Practices

### Code Quality Standards

#### 1. Compiler Warnings
- **ZERO tolerance for warnings**
- Use `#![deny(warnings)]` in CI
- Fix all warnings before committing

#### 2. Clippy Linting
```bash
# Run with all lints
cargo clippy --all-targets --all-features -- -D warnings

# Use clippy.toml for project-specific rules
```

**Key Clippy Rules:**
- No `unwrap()` in production code (use `?` or `expect()`)
- No `clone()` unless necessary
- Prefer iterators over loops
- Use `#[must_use]` on important return types

#### 3. Code Formatting
```bash
# Always run before commit
cargo fmt --all

# Check without modifying
cargo fmt --all -- --check
```

**rustfmt.toml settings:**
- `max_width = 100`
- `use_small_heuristics = "Max"`
- `edition = "2021"`

#### 4. Error Handling
- Use `thiserror` for error types
- Use `anyhow` for application errors
- **Never** use `panic!` or `unwrap()` in library code
- Use `expect()` with descriptive messages only when truly unreachable

```rust
// ❌ BAD
let value = map.get(&key).unwrap();

// ✅ GOOD
let value = map.get(&key)
    .ok_or(RasnError::KeyNotFound)?;

// ✅ ACCEPTABLE (with proof)
let value = map.get(&key)
    .expect("key must exist: inserted 5 lines above");
```

#### 5. Testing Standards
- **Minimum 80% code coverage**
- Unit tests in same file (`#[cfg(test)] mod tests`)
- Integration tests in `tests/` directory
- Property-based tests with `proptest` for complex logic
- Benchmarks in `benches/` directory

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_functionality() {
        // Arrange
        let input = setup_test_data();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

#### 6. Documentation
- **All public items MUST have doc comments**
- Use `///` for item documentation
- Use `//!` for module documentation
- Include examples in doc comments

```rust
/// Looks up ASN information for an IP address.
///
/// # Arguments
///
/// * `ip` - The IP address to query
///
/// # Returns
///
/// Returns `Some(AsnInfo)` if found, `None` otherwise.
///
/// # Examples
///
/// ```
/// use rasn_core::{IpAddr, Asn};
/// 
/// let ip = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
/// let info = lookup_ip(ip)?;
/// assert_eq!(info.asn, Asn(15169)); // Google
/// ```
pub fn lookup_ip(ip: IpAddr) -> Option<AsnInfo> {
    // Implementation
}
```

#### 7. Dependency Management
- Pin major versions in `Cargo.toml`
- Use `cargo outdated` to check for updates
- Run `cargo audit` for security vulnerabilities
- Minimize dependencies (prefer std when possible)

```toml
[dependencies]
tokio = { version = "1.35", features = ["full"] }  # Pin major
serde = { version = "1.0", features = ["derive"] }  # Pin major
```

#### 8. Performance
- Use `#[inline]` for hot path functions
- Prefer `&str` over `String` for parameters
- Use `Cow<str>` for conditional ownership
- Profile with `cargo flamegraph` or `perf`

```rust
#[inline]
pub fn find_ip(&self, ip: u32) -> Option<&AsnInfo> {
    // Hot path - inline to avoid function call overhead
}
```

#### 9. Workspace Organization
```toml
[workspace]
members = [
    "crates/*",
]

[workspace.dependencies]
# Shared dependencies go here
tokio = { version = "1.35", features = ["full"] }

[workspace.package]
edition = "2021"
rust-version = "1.75"
```

#### 10. CI/CD Integration
```yaml
# .github/workflows/ci.yml
- name: Format check
  run: cargo fmt --all -- --check

- name: Clippy
  run: cargo clippy --all-targets --all-features -- -D warnings

- name: Test
  run: cargo test --all-features

- name: Benchmark
  run: cargo bench --no-run

- name: Doc
  run: cargo doc --no-deps --all-features
```

---

## Performance Optimization Rules

### SIMD Code
- Test on both AVX2 and non-AVX2 systems
- Use `target_feature` checks
- Provide scalar fallback
- Verify with `cargo asm`

### Memory Management
- Use `Vec::with_capacity()` when size known
- Prefer stack allocation (`[T; N]`) over heap (`Vec<T>`)
- Use `Arc` for shared read-only data
- Profile with `cargo flamegraph`

### Async/Await
- Use `tokio::spawn` for CPU-bound tasks
- Use `rayon` for data parallelism
- Avoid `block_on` in async contexts
- Set appropriate timeouts

---

## Security Rules

### Input Validation
- Validate all external input
- Use strong types (newtype pattern)
- Sanitize user input for logging
- Rate limit API calls

### Sensitive Data
- Never log API keys or passwords
- Use OS keyring for key storage
- Encrypt data at rest
- Clear sensitive data from memory (zeroize)

---

## Pre-Release Checklist

Before marking issue as complete:
- [ ] `cargo fmt --all` - Code formatted
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` - No warnings
- [ ] `cargo test --all-features` - All tests pass
- [ ] `cargo doc --no-deps` - Documentation builds
- [ ] `cargo bench --no-run` - Benchmarks compile
- [ ] All acceptance criteria met
- [ ] Performance targets achieved
- [ ] No TODO comments (convert to issues)
- [ ] CHANGELOG.md updated (if applicable)
