# API Contracts

**Project:** RASN - Rust ASN Mapper  
**Version:** 1.0  
**Date:** October 26, 2025

---

## Core Library Types

```rust
pub struct AsnInfo {
    pub asn: u32,
    pub name: String,
    pub organization: String,
    pub country: String,
    pub ip_ranges: Vec<IpRange>,
    pub enrichment: Option<Enrichment>,
}

pub struct IpRange {
    pub start: IpAddr,
    pub end: IpAddr,
    pub cidr: Vec<IpNet>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("ASN {0} not found")]
    AsnNotFound(u32),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}
```

---

## Client API

```rust
pub struct Client {
    config: Config,
}

impl Client {
    pub fn new() -> Result<Self>;
    pub async fn lookup_asn(&self, asn: u32) -> Result<AsnInfo>;
    pub async fn lookup_ip(&self, ip: IpAddr) -> Result<AsnInfo>;
    pub async fn lookup_org(&self, org: &str) -> Result<Vec<AsnInfo>>;
    pub async fn lookup_batch<I>(&self, inputs: I) -> Vec<Result<AsnInfo>>;
}
```

---

## CLI Interface

```bash
# Commands
rasn lookup <ASN|IP|DOMAIN|ORG>
rasn batch -f <FILE>
rasn cidr <OPERATION> <CIDRS...>
rasn mcp [--stdio|--http]

# Global options
-j, --json      JSON output
--csv           CSV output
-v, --verbose   Verbose mode
```

---

## JSON Output

```json
{
  "asn": 14421,
  "name": "THERAVANCE",
  "country": "US",
  "ip_ranges": [{
    "cidr": ["216.101.17.0/24"]
  }]
}
```

---

## Exit Codes

- 0: Success
- 1: General error
- 2: Invalid input
- 3: Network error
