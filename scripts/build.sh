#!/usr/bin/env bash
set -euo pipefail

echo "Building RiceForge..."
cargo build --release --workspace
echo "Done. Binaries:"
echo "  target/release/riceforge"
echo "  target/release/rf-index"
