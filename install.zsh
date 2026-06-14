#!/usr/bin/env zsh
set -euo pipefail

SCRIPT_DIR=${0:A:h}
BIN_DIR=${SPOX_BIN:-$HOME/.local/bin}
TARGET=$BIN_DIR/spox

cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml"

mkdir -p "$BIN_DIR"

if [[ -L $TARGET ]]; then
    rm "$TARGET"
fi

ln -s "$SCRIPT_DIR/target/release/spox" "$TARGET"
echo "linked: $TARGET -> $SCRIPT_DIR/target/release/spox"

if ! (( $+commands[spox] )); then
    echo "note: $BIN_DIR is not in your PATH"
    echo "      add this to your .zshrc:  export PATH=\"\$HOME/.local/bin:\$PATH\""
fi
