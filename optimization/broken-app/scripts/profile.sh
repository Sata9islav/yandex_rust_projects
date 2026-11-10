#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts
cargo build --release --bin demo
perf record -F 99 -g --call-graph dwarf -o artifacts/perf.data -- ./target/release/demo
perf report --stdio -i artifacts/perf.data > artifacts/perf_report.txt
if command -v inferno-collapse-perf >/dev/null && command -v inferno-flamegraph >/dev/null; then
    perf script -i artifacts/perf.data | inferno-collapse-perf | inferno-flamegraph > artifacts/flamegraph.svg
fi
