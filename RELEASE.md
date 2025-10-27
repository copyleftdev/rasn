# Release Process

## Automated Releases

Releases are automated through GitHub Actions. To create a new release:

### 1. Update version numbers

```bash
# Update version in Cargo.toml files
sed -i 's/version = "0.1.0"/version = "0.2.0"/' crates/*/Cargo.toml
```

### 2. Create and push tag

```bash
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0
```

### 3. GitHub Actions will automatically:

- Run all CI checks (test, clippy, fmt)
- Build binaries for Linux, macOS, Windows
- Create GitHub release
- Upload release binaries
- Build and push Docker images to GHCR

## Manual Release Steps

If manual intervention is needed:

### Build Release Binaries

```bash
# Linux
cargo build --release --target x86_64-unknown-linux-gnu

# macOS
cargo build --release --target x86_64-apple-darwin

# Windows
cargo build --release --target x86_64-pc-windows-msvc
```

### Publish to crates.io

```bash
# From workspace root
cargo publish -p rasn-core
cargo publish -p rasn-arrow
cargo publish -p rasn-cache
# ... etc for each crate
cargo publish -p rasn-cli
```

## Release Checklist

- [ ] All tests passing
- [ ] Clippy warnings resolved
- [ ] Format check passing
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version numbers bumped
- [ ] Git tag created
- [ ] Release notes written

## Versioning

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (0.x.0): Breaking API changes
- **MINOR** (x.1.0): New features, backward compatible
- **PATCH** (x.x.1): Bug fixes, backward compatible

## Docker Images

Images are automatically built and pushed to GitHub Container Registry:

```bash
ghcr.io/copyleftdev/rasn:latest
ghcr.io/copyleftdev/rasn:v0.2.0
ghcr.io/copyleftdev/rasn:0.2
ghcr.io/copyleftdev/rasn:0
```

## Release Artifacts

Each release includes:

1. Source code (zip, tar.gz)
2. Compiled binaries:
   - `rasn-linux-amd64`
   - `rasn-linux-musl-amd64`
   - `rasn-macos-amd64`
   - `rasn-windows-amd64.exe`
3. Docker images
4. Release notes

## Troubleshooting

### Release workflow fails

Check the Actions tab for detailed logs. Common issues:

- Version conflicts with existing tags
- Cargo.toml version mismatches
- Missing dependencies

### Docker build fails

Ensure Dockerfile is compatible with all architectures:

```bash
docker build --platform linux/amd64 -t rasn:test .
```
