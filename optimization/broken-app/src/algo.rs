use std::collections::HashSet;

/// Optimized deduplication. Output is sorted to match the original/reference semantics.
pub fn slow_dedup(values: &[u64]) -> Vec<u64> {
    let mut seen = HashSet::with_capacity(values.len());
    let mut out = Vec::with_capacity(values.len());
    for &value in values {
        if seen.insert(value) {
            out.push(value);
        }
    }
    out.sort_unstable();
    out
}

/// Iterative O(n) Fibonacci, preserving the public function name.
/// Like the reference implementation, u64 inputs above 93 overflow.
pub fn slow_fib(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let (mut a, mut b) = (0_u64, 1_u64);
            for _ in 2..=n {
                let next = a + b;
                a = b;
                b = next;
            }
            b
        }
    }
}
