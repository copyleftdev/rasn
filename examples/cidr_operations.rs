//! CIDR operations example
//!
//! Run with: cargo run --example cidr_operations

use rasn_cidr::Cidr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("RASN - CIDR Operations Example\n");

    // Parse CIDR block
    let cidr = Cidr::parse("192.168.1.0/24")?;

    println!("CIDR: {}", "192.168.1.0/24");
    println!("─────────────────────────────");
    println!("Network:       {}", format_ip(cidr.network()));
    println!("Broadcast:     {}", format_ip(cidr.broadcast()));
    println!("First usable:  {}", format_ip(cidr.first_usable()));
    println!("Last usable:   {}", format_ip(cidr.last_usable()));
    println!("Total IPs:     {}", cidr.size());
    println!("Prefix length: /{}", cidr.prefix_len());

    println!("\nChecking IP containment:");
    println!("192.168.1.100 in range? {}", cidr.contains_ip(parse_ip("192.168.1.100")));
    println!("192.168.2.1 in range?   {}", cidr.contains_ip(parse_ip("192.168.2.1")));

    println!("\nFirst 5 IPs in range:");
    for (i, ip) in cidr.iter().take(5).enumerate() {
        println!("  {}: {}", i + 1, format_ip(ip));
    }

    Ok(())
}

fn parse_ip(ip: &str) -> u32 {
    let parts: Vec<u8> = ip.split('.').filter_map(|s| s.parse().ok()).collect();
    if parts.len() == 4 {
        u32::from_be_bytes([parts[0], parts[1], parts[2], parts[3]])
    } else {
        0
    }
}

fn format_ip(ip: u32) -> String {
    let bytes = ip.to_be_bytes();
    format!("{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3])
}
