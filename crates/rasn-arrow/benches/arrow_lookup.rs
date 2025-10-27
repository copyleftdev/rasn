use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rasn_arrow::IpRangeTableV4;
use std::path::Path;

fn load_table() -> IpRangeTableV4 {
    let paths = [
        Path::new("data/arrow/ip2asn-v4.parquet"),
        Path::new("../../data/arrow/ip2asn-v4.parquet"),
        Path::new("../../../data/arrow/ip2asn-v4.parquet"),
    ];

    for path in &paths {
        if path.exists() {
            return IpRangeTableV4::from_parquet(path).expect("Failed to load test data");
        }
    }

    panic!("Could not find test data in any expected location");
}

fn bench_single_lookup(c: &mut Criterion) {
    let table = load_table();

    c.bench_function("arrow_single_lookup", |b| {
        b.iter(|| {
            // 8.8.8.8 = 0x08080808 (Google DNS)
            table.find_ip(black_box(0x08080808))
        })
    });
}

fn bench_scalar_vs_simd(c: &mut Criterion) {
    let table = load_table();

    let mut group = c.benchmark_group("lookup_comparison");

    group.bench_function("scalar_lookup", |b| {
        b.iter(|| table.find_ip_scalar(black_box(0x08080808)))
    });

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            group.bench_function("simd_lookup", |b| {
                b.iter(|| table.find_ip(black_box(0x08080808)))
            });
        }
    }

    group.finish();
}

fn bench_batch_lookups(c: &mut Criterion) {
    let table = load_table();

    // Test different batch sizes
    let sizes = [10, 100, 1000];

    let test_ips: Vec<u32> = vec![
        0x08080808, // 8.8.8.8 (Google)
        0x01010101, // 1.1.1.1 (Cloudflare)
        0x08080404, // 8.8.4.4 (Google)
        0x09090909, // 9.9.9.9 (Quad9)
        0xC0000201, // 192.0.2.1 (TEST-NET-1)
    ];

    let mut group = c.benchmark_group("batch_lookups");

    for size in sizes.iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                for i in 0..size {
                    let ip = test_ips[i % test_ips.len()];
                    black_box(table.find_ip(ip));
                }
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_single_lookup,
    bench_scalar_vs_simd,
    bench_batch_lookups
);
criterion_main!(benches);
