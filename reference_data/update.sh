#!/bin/bash
# Update reference data from free ASN/GeoIP sources
# Run this daily/weekly to keep local databases current

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "==> Updating ASN reference data..."
echo ""

# IPtoASN (Updated hourly)
echo "[1/6] Downloading IPtoASN IPv4 database..."
curl -L -o ip2asn-v4.tsv.gz "https://iptoasn.com/data/ip2asn-v4.tsv.gz"
gunzip -f ip2asn-v4.tsv.gz

echo "[2/6] Downloading IPtoASN IPv6 database..."
curl -L -o ip2asn-v6.tsv.gz "https://iptoasn.com/data/ip2asn-v6.tsv.gz"
gunzip -f ip2asn-v6.tsv.gz

# ASN Information
echo "[3/6] Downloading ASN metadata..."
curl -L -o asn-info.csv "https://raw.githubusercontent.com/ipverse/asn-info/master/as.csv"

# sapics IP Location Database
echo "[4/6] Downloading ASN-Country IPv4 mappings..."
curl -L -o asn-country-ipv4.csv "https://raw.githubusercontent.com/sapics/ip-location-db/main/asn-country/asn-country-ipv4.csv"

echo "[5/6] Downloading Geo-WHOIS-ASN combined data..."
curl -L -o geo-whois-asn-country-ipv4.csv "https://raw.githubusercontent.com/sapics/ip-location-db/main/geo-whois-asn-country/geo-whois-asn-country-ipv4.csv"

echo "[6/6] Downloading ASN MMDB database..."
curl -L -o asn.mmdb "https://github.com/sapics/ip-location-db/raw/main/asn/asn.mmdb"

echo ""
echo "==> Update complete!"
echo ""
echo "Statistics:"
wc -l *.{tsv,csv} 2>/dev/null | tail -1
du -h * | grep -E '\.(tsv|csv|mmdb)$'

echo ""
echo "Next: Import to RocksDB with 'rasn db import'"
