# Docker Deployment Guide

## Quick Start

### Build the image

```bash
docker build -t rasn:latest .
```

### Run the CLI

```bash
docker run --rm rasn:latest lookup 8.8.8.8
```

### Run MCP Server (STDIO)

```bash
docker run --rm -it rasn:latest mcp stdio
```

### Using Docker Compose

```bash
# Start services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

## Configuration

### Environment Variables

- `RASN_API_KEY` - API key for external services

Example:

```bash
export RASN_API_KEY=your_key_here
docker-compose up -d
```

### Volume Mounts

Mount Arrow/Parquet data files:

```bash
docker run -v $(pwd)/data:/data:ro rasn:latest mcp --data /data/asn.parquet stdio
```

## Image Details

- **Base:** Alpine Linux (minimal size)
- **Size:** ~20-50MB (compressed)
- **User:** Non-root (UID 1000)
- **Security:** No privileged access required

## Health Checks

The Docker Compose configuration includes health checks:

```yaml
healthcheck:
  test: ["CMD", "rasn", "auth", "status"]
  interval: 30s
  timeout: 10s
  retries: 3
```

## Production Deployment

### 1. Build optimized image

```bash
docker build --target builder -t rasn:builder .
docker build -t rasn:0.1.0 .
```

### 2. Tag and push to registry

```bash
docker tag rasn:0.1.0 ghcr.io/copyleftdev/rasn:0.1.0
docker tag rasn:0.1.0 ghcr.io/copyleftdev/rasn:latest
docker push ghcr.io/copyleftdev/rasn:0.1.0
docker push ghcr.io/copyleftdev/rasn:latest
```

### 3. Deploy

```bash
docker pull ghcr.io/copyleftdev/rasn:latest
docker run -d --name rasn-mcp ghcr.io/copyleftdev/rasn:latest mcp stdio
```

## Security Best Practices

1. **Non-root user** - Container runs as UID 1000
2. **Read-only data** - Mount data volumes as `:ro`
3. **No secrets in image** - Use environment variables
4. **Minimal attack surface** - Alpine base (~5MB)
5. **Regular updates** - Rebuild images regularly

## Troubleshooting

### Container won't start

```bash
docker logs rasn-mcp
```

### Build fails

```bash
# Clean build
docker build --no-cache -t rasn:latest .
```

### Permission issues

```bash
# Ensure data directory is readable
chmod -R 755 ./data
```

## Multi-Architecture Builds

Build for multiple architectures:

```bash
docker buildx create --use
docker buildx build --platform linux/amd64,linux/arm64 -t rasn:latest .
```
