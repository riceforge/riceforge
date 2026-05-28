#!/usr/bin/env bash
# Install shell completions for riceforge
# Usage: ./scripts/completions.sh [zsh|bash|fish]
set -euo pipefail

SHELL_TYPE="${1:-$(basename "$SHELL")}"

case "$SHELL_TYPE" in
    zsh)
        DIR="${ZSH_COMPLETIONS_DIR:-${ZDOTDIR:-$HOME}/.zsh/completions}"
        mkdir -p "$DIR"
        riceforge completions zsh > "$DIR/_riceforge"
        echo "Installed zsh completions to $DIR/_riceforge"
        echo "Add to .zshrc if not already present: fpath=($DIR \$fpath)"
        ;;
    bash)
        DIR="${BASH_COMPLETION_USER_DIR:-${XDG_DATA_HOME:-$HOME/.local/share}/bash-completion/completions}"
        mkdir -p "$DIR"
        riceforge completions bash > "$DIR/riceforge"
        echo "Installed bash completions to $DIR/riceforge"
        ;;
    fish)
        DIR="${XDG_DATA_HOME:-$HOME/.local/share}/fish/vendor_completions.d"
        mkdir -p "$DIR"
        riceforge completions fish > "$DIR/riceforge.fish"
        echo "Installed fish completions to $DIR/riceforge.fish"
        ;;
    *)
        echo "Unknown shell: $SHELL_TYPE"
        echo "Usage: $0 [zsh|bash|fish]"
        exit 1
        ;;
esac
