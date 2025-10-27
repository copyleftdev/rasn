use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rasn_resolver::DnsResolver;

fn bench_resolver_creation(c: &mut Criterion) {
    c.bench_function("dns_resolver_creation", |b| {
        b.iter(|| black_box(DnsResolver::new().unwrap()))
    });
}

fn bench_cache_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let resolver = DnsResolver::new().unwrap();

    c.bench_function("dns_cache_stats", |b| {
        b.iter(|| rt.block_on(async { black_box(resolver.cache_stats().await) }))
    });
}

criterion_group!(benches, bench_resolver_creation, bench_cache_operations);
criterion_main!(benches);
