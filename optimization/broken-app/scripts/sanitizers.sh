#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p artifacts
TARGET="${TARGET:-x86_64-unknown-linux-gnu}"
rustup toolchain install nightly
rustup component add rust-src --toolchain nightly
for sanitizer in address thread; do
  RUSTFLAGS="-Zsanitizer=$sanitizer" \
  cargo +nightly test -Zbuild-std --target "$TARGET" --tests \
  2>&1 | tee "artifacts/${sanitizer}_sanitizer.txt"
done
