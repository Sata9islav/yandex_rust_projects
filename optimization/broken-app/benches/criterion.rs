use broken_app::{algo, sum_even, leak_buffer};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark(c: &mut Criterion) {
    let data: Vec<i64> = (0..50_000).collect();
    let dedup: Vec<u64> = (0..5_000).flat_map(|n| [n, n]).collect();
    let large_dedup: Vec<u64> = (0..25_000).flat_map(|n| [n, n]).collect();
    let bytes: Vec<u8> = (0..100_000).map(|v| (v % 256) as u8).collect();
    c.bench_function("sum_even_50000", |b| b.iter(|| black_box(sum_even(black_box(&data)))));
    c.bench_function("fib_32", |b| b.iter(|| black_box(algo::slow_fib(black_box(32)))));
    c.bench_function("dedup_10000", |b| b.iter(|| black_box(algo::slow_dedup(black_box(&dedup)))));
    c.bench_function("dedup_50000", |b| b.iter(|| black_box(algo::slow_dedup(black_box(&large_dedup)))));
    c.bench_function("count_nonzero_100000", |b| b.iter(|| black_box(leak_buffer(black_box(&bytes)))));
}
criterion_group!(benches, benchmark);
criterion_main!(benches);
