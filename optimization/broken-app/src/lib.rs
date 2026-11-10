pub mod algo;
pub mod concurrency;

/// Safe linear traversal, including empty slices.
pub fn sum_even(values: &[i64]) -> i64 {
    values.iter().copied().filter(|v| v % 2 == 0).sum()
}

/// Counts nonzero bytes without allocating or leaking memory.
pub fn leak_buffer(input: &[u8]) -> usize {
    input.iter().filter(|&&b| b != 0).count()
}

/// Matches reference-app: removes Unicode whitespace and lowercases the result.
pub fn normalize(input: &str) -> String {
    input.split_whitespace().collect::<String>().to_lowercase()
}

/// Computes the mean of strictly positive numbers only.
pub fn average_positive(values: &[i64]) -> f64 {
    let mut sum = 0_i128;
    let mut count = 0_usize;
    for &value in values {
        if value > 0 {
            sum += i128::from(value);
            count += 1;
        }
    }
    if count == 0 { 0.0 } else { sum as f64 / count as f64 }
}

/// Retains original signature for compatibility, but no longer accesses freed memory.
/// # Safety
/// No caller-side safety requirements; the function performs no unsafe operations.
pub unsafe fn use_after_free() -> i32 {
    let value = Box::new(42_i32);
    *value * 2
}
