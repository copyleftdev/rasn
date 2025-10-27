# Implementation Roadmap

**Project:** RASN - Rust ASN Mapper  
**Version:** 1.0

---

## Phase 1: Foundation (Weeks 1-3)

### Deliverables
- ✅ Cargo workspace setup
- ✅ Core types (`Asn`, `IpNet`, `Response`)
- ✅ Error handling (`thiserror`)
- ✅ Basic CLI (`clap`)
- ✅ CI/CD (GitHub Actions)

### Crates
```toml
tokio = "1.35"
anyhow = "1.0"
clap = { version = "4.4", features = ["derive"] }
```

---

## Phase 2: Core Functionality (Weeks 4-6)

### Deliverables
- ✅ HTTP client + API auth
- ✅ DNS resolver (hickory-dns)
- ✅ Input parser
- ✅ JSON/CSV output
- ✅ Integration tests

---

## Phase 3: Performance (Weeks 7-9)

### Deliverables
- ✅ Parallel DNS
- ✅ Batched API
- ✅ RocksDB local DB
- ✅ LRU cache
- ✅ Benchmarks

---

## Phase 4: Advanced Features (Weeks 10-12)

### Deliverables
- ✅ CIDR operations
- ✅ GeoIP integration
- ✅ WHOIS client
- ✅ Rate limiting
- ✅ Retry logic

---

## Phase 5: MCP Server (Weeks 13-15)

### Deliverables
- ✅ JSON-RPC server
- ✅ 7 MCP tools
- ✅ STDIO transport
- ✅ HTTP transport
- ✅ Tool schemas

---

## Phase 6: Production (Weeks 16-18)

### Deliverables
- ✅ API key encryption
- ✅ Prometheus metrics
- ✅ Documentation
- ✅ Docker images
- ✅ Release automation

---

## Milestones

| Version | Date | Features |
|---------|------|----------|
| 0.1.0 | Week 6 | MVP CLI |
| 0.5.0 | Week 12 | Performance + features |
| 1.0.0 | Week 18 | MCP + production ready |
