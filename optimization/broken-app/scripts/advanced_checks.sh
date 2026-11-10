#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts
rustup component add miri --toolchain nightly
cargo +nightly miri test 2>&1 | tee artifacts/miri.txt
cargo test --tests --no-run
TEST_BINARY=$(find target/debug/deps -maxdepth 1 -type f -name 'integration-*' -executable | head -1)
if [[ -z "$TEST_BINARY" ]]; then echo "integration test binary not found" >&2; exit 1; fi
valgrind --leak-check=full --show-leak-kinds=all --error-exitcode=1 "$TEST_BINARY" --test-threads=1 2>&1 | tee artifacts/valgrind.txt
