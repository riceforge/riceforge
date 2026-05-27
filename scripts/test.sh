#!/usr/bin/env bash
set -euo pipefail

echo "Running tests..."
cargo test --workspace

echo "Running clippy..."
cargo clippy --workspace -- -D warnings

echo "All checks passed."
