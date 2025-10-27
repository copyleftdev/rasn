use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rasn_arrow::IpRangeTableV4;
use std::path::Path;

fn benchmark_lookup(c: &mut Criterion) {
    // Only run if test file exists
    let path = Path::new("data/arrow/ip2asn-v4.parquet");
    if !path.exists() {
        eprintln!("Skipping benchmark: test data not found");
        return;
    }

    let table = IpRangeTableV4::from_parquet(path)
        .expect("Failed to load test data");

    c.bench_function("arrow_ipv4_lookup", |b| {
        b.iter(|| {
            // 8.8.8.8 = Google DNS
            table.find_ip(black_box(0x08080808))
        })
    });
}

criterion_group!(benches, benchmark_lookup);
criterion_main!(benches);
