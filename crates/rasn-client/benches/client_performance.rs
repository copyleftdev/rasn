use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rasn_client::ApiClient;

fn bench_client_creation(c: &mut Criterion) {
    c.bench_function("api_client_creation", |b| {
        b.iter(|| black_box(ApiClient::new("test-api-key".to_string())))
    });
}

fn bench_client_with_config(c: &mut Criterion) {
    c.bench_function("api_client_with_config", |b| {
        b.iter(|| {
            black_box(ApiClient::with_config(
                "key".to_string(),
                "https://api.test.com".to_string(),
                std::time::Duration::from_secs(5),
            ))
        })
    });
}

criterion_group!(benches, bench_client_creation, bench_client_with_config);
criterion_main!(benches);
