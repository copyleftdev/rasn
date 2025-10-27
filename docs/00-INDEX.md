# RASN Documentation Index

**Project:** RASN - Rust ASN Mapper  
**Status:** Planning Phase  
**Last Updated:** October 26, 2025

---

## Project Overview

RASN is a complete Rust rewrite of ProjectDiscovery's ASNmap, designed as both a high-performance CLI tool and a fully compliant MCP (Model Context Protocol) server for AI agent integration.

**Goals:** 10-100x performance improvement + AI-first design + enterprise features

---

## Documentation Structure

### Planning & Requirements
1. **[01-PRD.md](01-PRD.md)** - Product Requirements
   - Problem statement
   - Success metrics
   - Feature requirements
   - User stories

2. **[02-TRD.md](02-TRD.md)** - Technical Requirements
   - System architecture
   - Technology stack
   - Core components
   - Data models

### Design Specifications

3. **[03-ALGORITHMS.md](03-ALGORITHMS.md)** - Algorithm Designs
   - Input detection
   - DNS resolution
   - CIDR operations
   - Cache strategies

4. **[04-SIMD-OPTIMIZATIONS.md](04-SIMD-OPTIMIZATIONS.md)** - SIMD Optimizations
   - IP address operations
   - String parsing
   - Batch processing
   - Platform-specific optimizations

5. **[05-NETWORK-OPTIMIZATIONS.md](05-NETWORK-OPTIMIZATIONS.md)** - Network Optimizations
   - Connection pooling
   - Request batching
   - Keep-alive strategies
   - Protocol optimizations

6. **[06-MEMORY-OPTIMIZATIONS.md](06-MEMORY-OPTIMIZATIONS.md)** - Memory Optimizations
   - Zero-copy techniques
   - Arena allocators
   - String interning
   - Cache-friendly data structures

### MCP Integration

7. **[07-MCP-DESIGN.md](07-MCP-DESIGN.md)** - MCP Server Design
   - Protocol overview
   - Tool definitions
   - Resource handlers
   - Transport layers

8. **[08-MCP-TOOLS.md](08-MCP-TOOLS.md)** - MCP Tool Specifications
   - All 7 tool schemas
   - Input validation
   - Response formats
   - Error handling

### API & Contracts

9. **[09-API-CONTRACTS.md](09-API-CONTRACTS.md)** - API Contracts
   - Public interfaces
   - Data types
   - Error types
   - Versioning

10. **[10-DATA-SOURCES.md](10-DATA-SOURCES.md)** - Data Source Integration
    - ProjectDiscovery API
    - Local ASN databases
    - WHOIS integration
    - BGP data sources

### Implementation

11. **[11-ROADMAP.md](11-ROADMAP.md)** - Implementation Roadmap
    - 6 development phases
    - Deliverables per phase
    - Timeline estimates
    - Dependencies

12. **[12-PROJECT-STRUCTURE.md](12-PROJECT-STRUCTURE.md)** - Project Structure
    - Workspace layout
    - Crate organization
    - Module hierarchy
    - File conventions

---

## Quick Navigation

**For Contributors:**
- Start with [PRD](01-PRD.md) and [TRD](02-TRD.md)
- Review [Roadmap](11-ROADMAP.md) for current phase

**For Performance Engineers:**
- Review optimization docs: [SIMD](04-SIMD-OPTIMIZATIONS.md), [Network](05-NETWORK-OPTIMIZATIONS.md), [Memory](06-MEMORY-OPTIMIZATIONS.md)
- Check [Algorithms](03-ALGORITHMS.md) for complexity analysis

**For MCP Integrators:**
- Read [MCP Design](07-MCP-DESIGN.md) and [MCP Tools](08-MCP-TOOLS.md)
- Review [API Contracts](09-API-CONTRACTS.md)

---

## Key Metrics

| Metric | Go (Current) | Rust (Target) |
|--------|--------------|---------------|
| Single ASN lookup | 500ms | <50ms |
| 1000 IP batch | 30s | <3s |
| Memory (10k IPs) | 500MB | <50MB |
| DNS queries/sec | 50 | >5000 |
| Binary size | 15MB | <5MB |
