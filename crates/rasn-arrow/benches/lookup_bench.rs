use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rasn_arrow::IpRangeTableV4;
use std::path::Path;

fn benchmark_lookup(c: &mut Criterion) {
    let paths = [
        Path::new("data/arrow/ip2asn-v4.parquet"),
        Path::new("../../data/arrow/ip2asn-v4.parquet"),
    ];

    let table = paths
        .iter()
        .find(|p| p.exists())
        .and_then(|p| IpRangeTableV4::from_parquet(p).ok());

    let table = match table {
        Some(t) => t,
        None => {
            eprintln!("Skipping benchmark: test data not found");
            return;
        }
    };

    c.bench_function("arrow_ipv4_lookup", |b| {
        b.iter(|| {
            // 8.8.8.8 = Google DNS
            table.find_ip(black_box(0x08080808))
        })
    });
}

criterion_group!(benches, benchmark_lookup);
criterion_main!(benches);
