#!/usr/bin/env bash
set -euo pipefail

if ! command -v cargo-watch &> /dev/null; then
    echo "Installing cargo-watch..."
    cargo install cargo-watch
fi

echo "Watching for changes..."
cargo watch -x "test --workspace" -x "clippy --workspace -- -D warnings"
