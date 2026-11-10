use broken_app::{algo, average_positive, concurrency, leak_buffer, normalize, sum_even, use_after_free};

#[test]
fn sum_even_regression_empty_singleton_and_last() {
    assert_eq!(sum_even(&[]), 0);
    assert_eq!(sum_even(&[1]), 0);
    assert_eq!(sum_even(&[2]), 2);
    assert_eq!(sum_even(&[1, 2, 3, 4]), 6);
    assert_eq!(sum_even(&[-4, -3, 2]), -2);
}

#[test]
fn no_buffer_leak_and_nonzero_count() {
    assert_eq!(leak_buffer(&[]), 0);
    assert_eq!(leak_buffer(&[0, 1, 0, 2, 3]), 3);
    for _ in 0..100 { assert_eq!(leak_buffer(&[1, 0, 2]), 2); }
}

#[test]
fn normalizes_whitespace_including_tabs_and_newlines() {
    assert_eq!(normalize(" Hello World "), "helloworld");
    assert_eq!(normalize(" A\tB\nC "), "abc");
    assert_eq!(normalize(" \u{2003}Hello \tRuSt "), "hellorust");
}

#[test]
fn positive_average_and_empty_cases() {
    assert_eq!(average_positive(&[-5, 5, 15]), 10.0);
    assert_eq!(average_positive(&[-2, 0]), 0.0);
    assert_eq!(average_positive(&[]), 0.0);
    assert_eq!(average_positive(&[2, 4, 6]), 4.0);
    assert_eq!(average_positive(&[i64::MAX, i64::MAX]), i64::MAX as f64);
}

#[test]
fn use_after_free_has_no_dangling_dereference() {
    // SAFETY: the implementation has no caller-side preconditions.
    assert_eq!(unsafe { use_after_free() }, 84);
}

#[test]
fn dedup_is_sorted_and_unique() {
    assert_eq!(algo::slow_dedup(&[5, 5, 1, 2, 2, 3]), vec![1, 2, 3, 5]);
    assert_eq!(algo::slow_dedup(&[]), Vec::<u64>::new());
    assert_eq!(algo::slow_dedup(&[9, 9, 9]), vec![9]);
}

#[test]
fn fibonacci_regressions() {
    assert_eq!(algo::slow_fib(0), 0);
    assert_eq!(algo::slow_fib(1), 1);
    assert_eq!(algo::slow_fib(10), 55);
    assert_eq!(algo::slow_fib(32), 2_178_309);
    assert_eq!(algo::slow_fib(93), 12_200_160_415_121_876_738);
}

#[test]
fn atomic_increment_and_reset() {
    assert_eq!(concurrency::race_increment(1_000, 4), 4_000);
    assert_eq!(concurrency::read_after_sleep(), 4_000);
    concurrency::reset_counter();
    assert_eq!(concurrency::read_after_sleep(), 0);
    assert_eq!(concurrency::race_increment(0, 3), 0);
}
