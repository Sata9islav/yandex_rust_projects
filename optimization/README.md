# Debugging and Optimization

This project fixes correctness, memory-safety, and concurrency defects in the original `broken-app` and optimizes its computational hot paths. Public function names are preserved for compatibility. The reference implementation in `../reference-app` remains unchanged.

## Project layout

- `broken-app/` — fixed and optimized implementation, tests, benchmarks, scripts, and collected evidence.
- `reference-app/` — unchanged reference implementation.

## Defects fixed

| Function / module | Original issue | Fix |
|---|---|---|
| `sum_even` | Off-by-one loop and `get_unchecked` read beyond the slice | Safe slice iteration |
| `use_after_free` | Dereferenced a pointer after freeing its `Box` | Removed dangling-pointer access; returns `84` for the original example |
| `leak_buffer` | Leaked an allocation through `Box::into_raw` | Counts nonzero bytes without allocating |
| `normalize` | Removed only ordinary spaces, not other whitespace | Uses `split_whitespace` and lowercase conversion |
| `average_positive` | Included nonpositive values and used the wrong denominator | Averages positive values only, without a temporary vector; uses an `i128` accumulator |
| `concurrency` | Unsynchronized global mutable counter caused a data race | Uses `AtomicU64` and checks thread joins |
| `slow_fib` | Exponential recursive algorithm | Iterative **O(n) time / O(1) space** implementation |
| `slow_dedup` | Repeated linear searches and sorting on each insertion | `HashSet` membership checks and one final sort |

The functions `slow_fib` and `slow_dedup` retain their original names to avoid changing the public API, even though their implementations are now faster.

## How to run the checks (Linux)

From `optimization/broken-app/`:

```bash
cargo check
cargo test --tests
cargo run --bin demo

bash scripts/verify.sh
bash scripts/advanced_checks.sh   # Miri (nightly) and Valgrind
bash scripts/sanitizers.sh       # ASan and TSan (nightly and target toolchain required)
bash scripts/profile.sh          # Linux perf; may require perf permissions
bash scripts/bench_after.sh
bash scripts/compare.sh          # Original vs. optimized Fibonacci/dedup
```

Run the demo directly under Valgrind and save the result:

```bash
mkdir -p artifacts
cargo build
valgrind --leak-check=full --show-leak-kinds=all \
  --track-origins=yes --error-exitcode=1 \
  ./target/debug/demo > artifacts/valgrind_demo.txt 2>&1
```

**Notes:** Running `valgrind cargo test --tests` primarily analyzes the Cargo process, not necessarily the test executable. The supplied advanced-check script runs the test binary directly. Miri does not replace a thread-race detector; TSan is a separate check.

## Short verification report

Results below are from local Linux runs and uploaded logs, not from the environment in which the project archive was created.

| Check | Observed result | Evidence |
|---|---|---|
| `cargo check` | Passed | `artifacts/cargo_check.txt` |
| Integration tests | **8 passed, 0 failed** | `artifacts/cargo_test.txt` |
| Miri | **8 passed, 0 failed**; no UB reported in tested paths | `artifacts/miri.txt` |
| AddressSanitizer | **8 passed, 0 failed**; no sanitizer error reported | `artifacts/address_sanitizer.txt` |
| ThreadSanitizer | **8 passed, 0 failed**; no sanitizer error reported | `artifacts/thread_sanitizer.txt` |
| Valgrind: test executable | `0` definitely/indirectly lost; **48 bytes possibly lost**, `ERROR SUMMARY: 1` | `artifacts/valgrind.txt` |
| Valgrind: demo | **0 errors**, `0` definitely/indirectly/possibly lost; `548 bytes still reachable` in Rust runtime structures | `artifacts/valgrind_demo.txt` |
| perf / Flamegraph | Profiles and flamegraphs generated | `artifacts/perf_bench_report.txt`, `artifacts/flamegraph_bench.svg` |

**Valgrind interpretation:** The standalone demo check is clean. The earlier test-binary check had one *possibly lost* allocation, so it should not be described as a completely clean Valgrind test run. The demo's 548 *still reachable* bytes originate from Rust runtime/thread initialization and are not classified as lost memory.

## Performance results

The original and optimized versions were benchmarked with matching Criterion scenarios and `black_box` around inputs/results. Values below are Criterion's central time estimates from the uploaded results.

| Scenario | Before | After | Speedup |
|---|---:|---:|---:|
| Fibonacci `n = 32` | 3.7916 ms | 3.3574 ns | ~1,129,000× |
| Dedup, 10,000 elements | 10.316 ms | 73.872 µs | ~139.6× |
| Dedup, 50,000 elements | 254.43 ms | 398.13 µs | ~639.1× |

Speedup is calculated as `time_before / time_after` after converting to the same units. The Fibonacci result is extremely fast and should be interpreted with care despite the use of `black_box`; it is a comparison of the measured workloads, not a guaranteed speedup for every input or environment. The dedup results also demonstrate the benefit at a larger input size.

The perf profile identifies `broken_app::algo::slow_dedup` as a major CPU hotspot in the recorded benchmark. The profile includes benchmarking-framework overhead, so percentages are specific to this measurement setup. The original unsafe `sum_even` was deliberately excluded from before/after timing because its undefined behavior makes such measurements unreliable.

### Optimization rationale

- **Algorithmic:** iterative Fibonacci replaces exponential recursion; hash-based deduplication removes repeated linear searches and sorts once instead of after every insertion.
- **Micro:** eliminated unnecessary buffer allocation in `leak_buffer` and intermediate vector creation in `average_positive`.
- **Correctness:** regression tests cover the boundary, whitespace, averaging, memory, and concurrency cases discovered in the original code.

## Artifacts and reproducibility

Keep the human-readable diagnostic logs, benchmark outputs, and Flamegraph SVG in `broken-app/artifacts/` and commit them with the source code. Do **not** commit Cargo's `target/` directory. Large binary perf recordings (`*.data`) can be kept outside Git; retain the reports and Flamegraph SVG for review. If `valgrind_demo.txt` is not yet in the project, copy the separately collected log into `artifacts/` before submitting.

## Known limitations

- Fibonacci overflows `u64` for `n > 93`, as it does in the reference implementation.
- The atomic counter is shared global state. Independent concurrent calls to `race_increment` share the counter; sequential batches are the supported usage pattern.
- Diagnostic results establish behavior for the exercised test cases, not a formal proof of the absence of all possible defects.
