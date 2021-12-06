use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode};
use std::time::Duration;

pub mod core;
pub mod postgres;

macro_rules! benchmark_id {
    ($fn_name:expr, $pool_size:expr, $workers:expr) => {
        BenchmarkId::new(
            $fn_name,
            format!("pool={:02} worker={:02}", $pool_size, $workers),
        )
    };
}

fn product(a: Vec<usize>, b: Vec<usize>) -> Vec<(usize, usize)> {
    let mut c = Vec::with_capacity(a.len() * b.len());
    for x in &a {
        for y in &b {
            c.push((*x, *y));
        }
    }
    c
}

pub fn bench_core(c: &mut Criterion) {
    c.bench_function("loop factorial 20", |b| b.iter(core::loop_factorial20));
    let mut group = c.benchmark_group("core");
    group
        .measurement_time(Duration::from_secs(5))
        .nresamples(10_000)
        .sample_size(100)
        .sampling_mode(SamplingMode::Flat)
        .warm_up_time(Duration::from_millis(100));
    let inputs = product(vec![4usize, 8, 16], vec![1usize, 4, 16, 64]);
    for input in inputs {
        group.bench_with_input(
            benchmark_id!("bb8", input.0, input.1),
            &input,
            core::bb8::bench_with_input,
        );
        group.bench_with_input(
            benchmark_id!("deadpool", input.0, input.1),
            &input,
            core::deadpool::bench_with_input,
        );
        group.bench_with_input(
            benchmark_id!("mobc", input.0, input.1),
            &input,
            core::mobc::bench_with_input,
        );
        group.bench_with_input(
            benchmark_id!("qp", input.0, input.1),
            &input,
            core::qp::bench_with_input,
        );
        group.bench_with_input(
            benchmark_id!("r2d2", input.0, input.1),
            &input,
            core::r2d2::bench_with_input,
        );
    }
    group.finish();
}

pub fn bench_postgres(c: &mut Criterion) {
    let mut group = c.benchmark_group("postgres");
    group
        .measurement_time(Duration::from_secs(10))
        .nresamples(10_000)
        .sample_size(100)
        .sampling_mode(SamplingMode::Flat)
        .warm_up_time(Duration::from_millis(100));
    let inputs = product(vec![4usize, 8, 16], vec![1usize, 4, 16, 64]);
    for input in inputs {
        /*group.bench_with_input(
            benchmark_id!("bb8", input.0, input.1),
            &input,
            postgres::bb8::bench_with_input,
        );*/
        group.bench_with_input(
            benchmark_id!("deadpool", input.0, input.1),
            &input,
            postgres::deadpool::bench_with_input,
        );
        group.bench_with_input(
            benchmark_id!("mobc", input.0, input.1),
            &input,
            postgres::mobc::bench_with_input,
        );
        group.bench_with_input(
            benchmark_id!("qp", input.0, input.1),
            &input,
            postgres::qp::bench_with_input,
        );
        group.bench_with_input(
            benchmark_id!("r2d2", input.0, input.1),
            &input,
            postgres::r2d2::bench_with_input,
        );
        group.bench_with_input(
            benchmark_id!("sqlx", input.0, input.1),
            &input,
            postgres::sqlx::bench_with_input,
        );
    }
    group.finish();
}

criterion_group!(benches, bench_core, bench_postgres);
criterion_main!(benches);
