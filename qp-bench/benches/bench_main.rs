use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use tokio::runtime::Runtime;

pub mod async_pool;

fn bench_async_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_pool");
    for pool_size in [4, 8, 16].into_iter() {
        for workers in [1, 4, 16, 64].into_iter() {
            let params = (pool_size as usize, workers as usize);
            group.bench_with_input(
                BenchmarkId::new("deadpool", format!("pool={} worker={}", pool_size, workers)),
                &params,
                |b, &p| {
                    b.to_async(Runtime::new().unwrap())
                        .iter(|| async_pool::deadpool::run_with(p.0, p.1))
                },
            );
            group.bench_with_input(
                BenchmarkId::new("qp", format!("pool={} worker={}", pool_size, workers)),
                &params,
                |b, &p| {
                    b.to_async(Runtime::new().unwrap())
                        .iter(|| async_pool::qp::run_with(p.0, p.1))
                },
            );
        }
    }
    group.finish();
}

criterion_group!(benches, bench_async_pool);
criterion_main!(benches);
