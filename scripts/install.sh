#!/usr/bin/env bash
set -euo pipefail

PREFIX="${PREFIX:-/usr/local}"
BIN="$PREFIX/bin"
GUI="${GUI:-0}"

if [[ "$GUI" == "1" ]]; then
    echo "Installing system dependencies for GUI..."
    sudo pacman -S --needed --noconfirm webkit2gtk-4.1 gtk3 libsoup3
fi

echo "Building riceforge CLI..."
cargo build --release -p rf-cli

echo "Installing to $BIN/riceforge..."
sudo install -Dm755 target/release/riceforge "$BIN/riceforge"

echo "Building rf-index..."
cargo build --release -p rf-index
sudo install -Dm755 target/release/rf-index "$BIN/rf-index"

if [[ "$GUI" == "1" ]]; then
    echo "Building rf-gui..."
    cargo build --release -p rf-gui
    sudo install -Dm755 target/release/rf-gui "$BIN/rf-gui"

    # Install .desktop entry
    DESKTOP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
    mkdir -p "$DESKTOP_DIR"
    install -Dm644 rf-gui/assets/riceforge.desktop "$DESKTOP_DIR/riceforge.desktop"
    echo "Desktop entry installed to $DESKTOP_DIR/riceforge.desktop"
fi

echo ""
echo "Done."
echo "  riceforge update    — fetch the rice registry"
echo "  riceforge --help    — show all commands"
if [[ "$GUI" == "1" ]]; then
    echo "  rf-gui              — launch the GUI"
fi
