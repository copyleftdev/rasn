# Product Requirements Document

**Project:** RASN - Rust ASN Mapper  
**Version:** 1.0  
**Date:** October 26, 2025

---

## Executive Summary

Rewrite ProjectDiscovery's ASNmap in Rust to create a high-performance ASN reconnaissance tool with native MCP server capabilities. Target: 10-100x performance improvement while adding AI agent integration.

---

## Problem Statement

### Current Limitations (Go Implementation)

**Performance Issues:**
- Sequential DNS resolution (no concurrency)
- No API request batching or parallelization  
- Missing connection pooling
- High memory overhead for large datasets

**Feature Gaps:**
- No offline/local database support
- No caching mechanism
- Single data source (PD API only)
- No CIDR optimization or aggregation
- No AI/agent integration
- Limited data enrichment

**Security Concerns:**
- TLS verification disabled globally (`InsecureSkipVerify`)
- API keys stored in plaintext
- No audit logging
- No certificate pinning

---

## Target Users

1. **Security Researchers** - Need fast, comprehensive ASN mapping
2. **Red Team Operators** - Require offline capability and stealth
3. **AI Agents (Claude, GPT)** - Need structured MCP access
4. **DevOps Engineers** - CI/CD integration, reliability
5. **Threat Intel Analysts** - Need enriched data (GeoIP, relationships)

---

## Success Criteria

### Quantitative Metrics

| Metric | Current (Go) | Target (Rust) | Improvement |
|--------|--------------|---------------|-------------|
| Single ASN lookup | 500ms | <50ms | **10x** |
| 1000 IP batch | 30s | <3s | **10x** |
| Memory (10k IPs) | 500MB | <50MB | **10x** |
| DNS queries/sec | 50 | >5000 | **100x** |
| Binary size | 15MB | <5MB | **3x** |
| Cold start | 100ms | <10ms | **10x** |
| Test coverage | ~30% | >80% | **Industry** |

### Qualitative Goals

- ✅ Drop-in CLI replacement for asnmap
- ✅ Native MCP server (Anthropic schema compliant)
- ✅ Offline mode (local ASN database)
- ✅ Rich library API with examples
- ✅ Production-ready observability
- ✅ Secure by default

---

## Core Features

### Phase 1: MVP (Parity)

**Input Types:**
- ASN (AS14421, 14421)
- IP (IPv4/IPv6)
- Domain (DNS resolution)
- Organization name
- STDIN support
- File input

**Output Formats:**
- CIDR list (default)
- JSON (structured)
- CSV (pipe-delimited)

**Essential:**
- API authentication (PDCP)
- Error handling
- Progress indicators

### Phase 2: Performance

**Concurrency:**
- Parallel DNS (1000+ concurrent)
- Batched API requests
- Connection pooling with keep-alive
- Adaptive rate limiting

**Caching:**
- In-memory LRU (1000 entries)
- Persistent disk cache (RocksDB)
- Redis support (distributed)
- Configurable TTL (default 24h)

**Resilience:**
- Smart retry with exponential backoff
- Circuit breaker pattern
- Graceful degradation (API → local DB)
- Timeout configuration

### Phase 3: Advanced Features

**Multiple Data Sources:**
- ProjectDiscovery Cloud API (primary)
- Local ASN database/RIR data (fallback)
- WHOIS enrichment
- MaxMind GeoIP
- BGP route tables (optional)

**CIDR Operations:**
- Automatic aggregation (merge adjacent)
- Supernet calculation
- Overlap detection
- Set operations (union, intersection, diff)
- Subnet splitting
- IP range validation

**Data Enrichment:**
- GeoIP (country, city, coordinates)
- AS-path analysis
- BGP communities
- Historical ownership
- Abuse contacts
- Registration dates
- Peering relationships

### Phase 4: MCP Server

**7 MCP Tools:**
1. `asn_lookup` - Query by ASN
2. `ip_to_asn` - Find ASN for IP
3. `domain_to_asn` - Resolve domain
4. `org_to_asn` - Find org's ASNs
5. `cidr_operations` - CIDR math
6. `asn_relationship` - Peering/transit
7. `batch_lookup` - Bulk operations

**MCP Resources:**
- `asn://{asn}/info`
- `asn://{asn}/prefixes`
- `asn://{asn}/history`
- `cache://stats`

**Transports:**
- STDIO (primary - Claude Desktop)
- HTTP/WebSocket (secondary)
- Unix socket (optional)

### Phase 5: Enterprise

**Security:**
- API key encryption (system keyring)
- Certificate pinning
- mTLS support
- Audit logging (JSON)

**Observability:**
- Structured logging
- Prometheus metrics
- OpenTelemetry tracing
- Health endpoints

**Developer Experience:**
- Shell completions (bash/zsh/fish/powershell)
- Man pages
- Interactive TUI
- Comprehensive docs

---

## User Stories

### Story 1: Security Researcher
**As a** security researcher  
**I want to** map all IPs for a target org  
**So that** I can identify attack surface

**Acceptance:**
- Input: org name
- Output: All ASNs + CIDR ranges (IPv4/IPv6)
- Performance: <5s for typical org

### Story 2: AI Agent
**As an** AI agent (Claude)  
**I want to** query ASN info via MCP  
**So that** I can assist with network recon

**Acceptance:**
- MCP server starts via stdio
- 7 tools discoverable
- Anthropic schema compliance
- Streaming for large results

### Story 3: Red Team
**As a** red teamer  
**I want** offline ASN lookups  
**So that** I maintain opsec

**Acceptance:**
- Downloadable local DB
- Fully offline operation
- Separate update command
- Performance <2x online mode

### Story 4: DevOps
**As a** DevOps engineer  
**I want** CI/CD ASN integration  
**So that** I automate network inventory

**Acceptance:**
- Scriptable (exit codes)
- JSON output (parseable)
- Cached results
- Redis support

---

## Non-Functional Requirements

**Performance:**
- P95 latency <100ms (cached)
- >10k queries/sec (local DB)
- 1000+ concurrent operations
- O(1) memory (streaming mode)

**Scalability:**
- Handle millions of IPs
- Stream results (no OOM)
- 100k+ cache entries

**Reliability:**
- Graceful API downtime handling
- Data validation
- Automatic retry

**Security:**
- Secure credential storage
- TLS 1.3 minimum
- Audit all sensitive ops
- Sandbox untrusted sources

---

## Out of Scope (Future)

- Web UI dashboard
- Real-time BGP monitoring
- Plugin system
- Database clustering
- GraphQL API
- Blockchain/Web3 tracking

---

## Release Criteria

**v0.1.0 (MVP):**
- Core CLI functionality
- All input types
- JSON/CSV/CIDR output
- API client working
- >60% test coverage

**v0.5.0 (Performance):**
- Concurrent processing
- Caching layer
- Local database
- 10x benchmark improvement

**v1.0.0 (Production):**
- MCP server complete
- All 7 tools
- Security hardened
- Observability complete
- >80% test coverage
- Full documentation

---

## Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| PD API changes | High | Low | Version pin, local fallback |
| MCP spec evolves | High | Medium | Track Anthropic updates |
| Performance targets | Medium | Low | Early benchmarking |
| Local DB size | Medium | Medium | Compression, optional |
| Adoption | Medium | Medium | Marketing, docs, examples |
