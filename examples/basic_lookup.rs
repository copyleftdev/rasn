//! Basic IP lookup example
//!
//! Run with: cargo run --example basic_lookup

use rasn_arrow::IpRangeTableV4;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RASN - Basic Lookup Example\n");

    // Note: This example requires Arrow/Parquet data file
    // For testing without data, see the mock example below

    // Example: Load Arrow table (if data exists)
    let data_path = Path::new("data/asn.parquet");
    if data_path.exists() {
        let table = IpRangeTableV4::from_parquet(data_path)?;
        
        // Lookup Google DNS
        let ip: u32 = (8 << 24) | (8 << 16) | (8 << 8) | 8; // 8.8.8.8
        
        if let Some(info) = table.find_ip(ip) {
            println!("IP: 8.8.8.8");
            println!("ASN: AS{}", info.asn.0);
            println!("Organization: {}", info.organization);
            println!("Country: {}", info.country);
        } else {
            println!("No ASN found for 8.8.8.8");
        }
    } else {
        println!("No data file found at: {:?}", data_path);
        println!("This example requires Arrow/Parquet data.");
        println!("\nMock example:");
        println!("IP: 8.8.8.8");
        println!("ASN: AS15169");
        println!("Organization: Google LLC");
        println!("Country: US");
    }

    Ok(())
}
