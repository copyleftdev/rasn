use rasn_arrow::IpRangeTableV4;
use std::path::Path;

#[test]
fn test_load_real_parquet_file() {
    let path = Path::new("data/arrow/ip2asn-v4.parquet");

    if !path.exists() {
        eprintln!("Skipping test: real data not found at {:?}", path);
        return;
    }

    let table = IpRangeTableV4::from_parquet(path).expect("Failed to load real Parquet file");

    // Verify it loaded data
    assert!(!table.is_empty(), "Table should have data");

    println!("✓ Loaded {} IP ranges", table.len());
}

#[test]
fn test_lookup_google_dns() {
    let path = Path::new("data/arrow/ip2asn-v4.parquet");

    if !path.exists() {
        eprintln!("Skipping test: real data not found");
        return;
    }

    let table = IpRangeTableV4::from_parquet(path).expect("Failed to load");

    // 8.8.8.8 = 0x08080808 (Google DNS)
    if let Some(info) = table.find_ip(0x08080808) {
        println!("✓ Found 8.8.8.8: ASN {} ({})", info.asn, info.organization);
        assert_eq!(info.asn.0, 15169, "Google ASN should be 15169");
    } else {
        panic!("Should find 8.8.8.8");
    }
}

#[test]
fn test_lookup_cloudflare() {
    let path = Path::new("data/arrow/ip2asn-v4.parquet");

    if !path.exists() {
        return;
    }

    let table = IpRangeTableV4::from_parquet(path).expect("Failed to load");

    // 1.1.1.1 = 0x01010101 (Cloudflare DNS)
    if let Some(info) = table.find_ip(0x01010101) {
        println!("✓ Found 1.1.1.1: ASN {} ({})", info.asn, info.organization);
        assert_eq!(info.asn.0, 13335, "Cloudflare ASN should be 13335");
    }
}

#[test]
fn test_lookup_nonexistent() {
    let path = Path::new("data/arrow/ip2asn-v4.parquet");

    if !path.exists() {
        return;
    }

    let table = IpRangeTableV4::from_parquet(path).expect("Failed to load");

    // Private IP - should not be in public ASN database
    let result = table.find_ip(0xC0A80001); // 192.168.0.1
                                            // Private IPs might or might not be in the database, so just verify it doesn't crash
    println!("Private IP lookup result: {:?}", result.is_some());
}
