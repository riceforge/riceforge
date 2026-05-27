#!/usr/bin/env bash
set -euo pipefail

if ! command -v cargo-tarpaulin &>/dev/null; then
    echo "Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

echo "Generating coverage report..."
cargo tarpaulin \
    --workspace \
    --exclude rf-gui \
    --out Html \
    --output-dir target/coverage \
    --timeout 120

echo "Coverage report: target/coverage/tarpaulin-report.html"
