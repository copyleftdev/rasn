# Multi-stage Docker build for RASN
# Optimized for production with minimal image size

# Build stage
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

# Set working directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml ./
COPY Cargo.lock* ./
COPY crates ./crates

# Build release binary
RUN cargo build --release --bin rasn

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Copy binary from builder
COPY --from=builder /app/target/release/rasn /usr/local/bin/rasn

# Create non-root user
RUN adduser -D -u 1000 rasn

# Switch to non-root user
USER rasn

# Set entrypoint
ENTRYPOINT ["rasn"]
CMD ["--help"]

# Metadata
LABEL org.opencontainers.image.title="RASN"
LABEL org.opencontainers.image.description="High-performance ASN mapper"
LABEL org.opencontainers.image.vendor="RASN Project"
LABEL org.opencontainers.image.source="https://github.com/copyleftdev/rasn"
