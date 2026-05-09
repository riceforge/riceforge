#!/usr/bin/env bash
set -euo pipefail

PREFIX="${PREFIX:-/usr/local}"
BIN="$PREFIX/bin"

echo "Building riceforge..."
cargo build --release -p riceforge

echo "Installing to $BIN/riceforge..."
install -Dm755 target/release/riceforge "$BIN/riceforge"

echo "Done. Run: riceforge --help"
