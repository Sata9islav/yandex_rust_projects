use broken_app::algo;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
fn benchmark(c: &mut Criterion) {
    let dedup: Vec<u64> = (0..5_000).flat_map(|n| [n, n]).collect();
    let large_dedup: Vec<u64> = (0..25_000).flat_map(|n| [n, n]).collect();
    c.bench_function("fib_32", |b| b.iter(|| black_box(algo::slow_fib(black_box(32)))));
    c.bench_function("dedup_10000", |b| b.iter(|| black_box(algo::slow_dedup(black_box(&dedup)))));
    c.bench_function("dedup_50000", |b| b.iter(|| black_box(algo::slow_dedup(black_box(&large_dedup)))));
}
criterion_group!(benches, benchmark);
criterion_main!(benches);
