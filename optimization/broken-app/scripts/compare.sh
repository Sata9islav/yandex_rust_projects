#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
mkdir -p "$ROOT/broken-app/artifacts"
# Only safe original algorithms. Do not benchmark original sum_even (UB).
cd "$ROOT/broken-app-original"
cargo bench --bench comparison -- --save-baseline before 2>&1 | tee "$ROOT/broken-app/artifacts/comparison_before.txt"
cd "$ROOT/broken-app"
cargo bench --bench comparison -- --save-baseline after 2>&1 | tee artifacts/comparison_after.txt
# Results are from two separate build directories; use their outputs to
# calculate ratios. Criterion baseline names alone do not compare across crates.
