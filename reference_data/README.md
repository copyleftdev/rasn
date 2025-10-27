# Reference Data

This directory contains free, open-source ASN and geolocation databases for offline lookup fallback.

## Downloaded Datasets

### 1. IPtoASN Database (Updated Hourly)

**Source:** https://iptoasn.com/

**Files:**
- `ip2asn-v4.tsv.gz` / `ip2asn-v4.tsv` - IPv4 to ASN mappings
- `ip2asn-v6.tsv.gz` / `ip2asn-v6.tsv` - IPv6 to ASN mappings

**Format:** TSV (Tab-Separated Values)
```
start_ip    end_ip    asn    country    organization
1.0.0.0     1.0.0.255 13335  US         CLOUDFLARENET
```

**Update Frequency:** Hourly  
**License:** Free to use

### 2. ASN Information Database

**Source:** https://github.com/ipverse/asn-info

**Files:**
- `asn-info.csv` - ASN metadata (number, handle, description)

**Format:** CSV with header
```
asn,handle,description
1,LVLT-1,Level 3 Parent LLC
```

**Update Frequency:** Weekly  
**License:** MIT

### 3. IP Location Database (sapics)

**Source:** https://github.com/sapics/ip-location-db

**Files:**
- `asn-country-ipv4.csv` - IPv4 ranges with country codes
- `geo-whois-asn-country-ipv4.csv` - Combined geo/whois/asn data
- `dbip-asn-lite.mmdb` - MaxMind DB format for fast lookups

**Format:** CSV (no header) and MMDB
```
start_ip,end_ip,country
1.0.0.0,1.0.0.255,AU
```

**Update Frequency:** Daily  
**License:** CC0 1.0 (Public Domain)

---

## Data Statistics

| Dataset | Records | Size (Compressed) | Size (Uncompressed) |
|---------|---------|-------------------|---------------------|
| ip2asn-v4 | ~500k | 6.4 MB | ~25 MB |
| ip2asn-v6 | ~140k | 1.8 MB | ~6 MB |
| asn-info | ~70k | 5.7 MB | 5.7 MB |
| asn-country-ipv4 | ~400k | 4.1 MB | 4.1 MB |
| geo-whois-asn-country | ~750k | 7.9 MB | 7.9 MB |

---

## Usage in RASN

These databases serve as fallback when:
1. ProjectDiscovery API is unavailable
2. Offline mode is enabled
3. Rate limits are exceeded
4. Local-first privacy is required

### Priority Order:
1. Memory cache (< 1ms)
2. Disk cache (~ 5ms)
3. **Local reference data (< 10ms)** ← These files
4. PD Cloud API (~ 100ms)
5. WHOIS (~ 500ms)

---

## Updating Reference Data

### Manual Update:
```bash
cd reference_data

# Update IPtoASN data (hourly updates)
curl -L -o ip2asn-v4.tsv.gz "https://iptoasn.com/data/ip2asn-v4.tsv.gz"
curl -L -o ip2asn-v6.tsv.gz "https://iptoasn.com/data/ip2asn-v6.tsv.gz"
gunzip -f ip2asn-v4.tsv.gz ip2asn-v6.tsv.gz

# Update ASN info
curl -L -o asn-info.csv "https://raw.githubusercontent.com/ipverse/asn-info/master/as.csv"

# Update sapics databases
curl -L -o asn-country-ipv4.csv "https://raw.githubusercontent.com/sapics/ip-location-db/main/asn-country/asn-country-ipv4.csv"
curl -L -o geo-whois-asn-country-ipv4.csv "https://raw.githubusercontent.com/sapics/ip-location-db/main/geo-whois-asn-country/geo-whois-asn-country-ipv4.csv"
```

### Automated Update (via cron):
```bash
# Add to crontab (daily at 3 AM)
0 3 * * * cd /path/to/rasn/reference_data && ./update.sh
```

---

## Data Schema

### ip2asn-v4.tsv / ip2asn-v6.tsv
| Column | Type | Description |
|--------|------|-------------|
| start_ip | IP Address | Start of IP range |
| end_ip | IP Address | End of IP range |
| asn | Integer | ASN number (0 = not routed) |
| country | String(2) | ISO country code |
| organization | String | AS organization name |

### asn-info.csv
| Column | Type | Description |
|--------|------|-------------|
| asn | Integer | ASN number |
| handle | String | ASN handle/identifier |
| description | String | Organization description |

### asn-country-ipv4.csv
| Column | Type | Description |
|--------|------|-------------|
| start_ip | IP Address | Start of IP range |
| end_ip | IP Address | End of IP range |
| country | String(2) | ISO country code |

---

## Import to RocksDB

The `rasn-db` crate will import these files into RocksDB format for fast lookups:

```bash
# Import reference data to local database
rasn db import --source reference_data/ip2asn-v4.tsv
rasn db import --source reference_data/asn-info.csv

# Check database status
rasn db info
```

---

## License & Attribution

All datasets used are free and open-source:

- **IPtoASN:** Free to use, data sourced from RIR databases
- **ipverse/asn-info:** MIT License
- **sapics/ip-location-db:** CC0 1.0 Universal (Public Domain)

Please credit original sources when redistributing.

---

## Additional Resources

- **MaxMind GeoLite2:** https://dev.maxmind.com/geoip/geolite2-free-geolocation-data
- **RIPE NCC:** https://www.ripe.net/analyse/internet-measurements/routing-information-service-ris
- **BGP Data:** https://www.routeviews.org/
