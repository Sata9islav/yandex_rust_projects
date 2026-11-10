use broken_app::{algo, sum_even};
use std::hint::black_box;
use std::time::Instant;

fn time_it(label: &str, mut f: impl FnMut()) {
    let start = Instant::now();
    f();
    println!("{label}: {:?}", start.elapsed());
}

fn main() {
    let data: Vec<i64> = (0..50_000).collect();
    let dedup_data: Vec<u64> = (0..5_000).flat_map(|n| [n, n]).collect();
    for _ in 0..3 {
        time_it("sum_even", || { black_box(sum_even(black_box(&data))); });
        time_it("slow_fib", || { black_box(algo::slow_fib(black_box(32))); });
        time_it("slow_dedup", || { black_box(algo::slow_dedup(black_box(&dedup_data))); });
    }
}
