#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts
cargo check 2>&1 | tee artifacts/cargo_check.txt
cargo test --tests 2>&1 | tee artifacts/cargo_test.txt
