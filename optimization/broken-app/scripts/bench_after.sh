#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts
cargo bench --bench criterion -- --save-baseline after 2>&1 | tee artifacts/criterion_after.txt
