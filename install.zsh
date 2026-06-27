#!/usr/bin/env zsh
set -euo pipefail

SCRIPT_DIR=${0:A:h}
BIN_DIR=${SPOX_BIN:-$HOME/.local/bin}
TARGET=$BIN_DIR/spox

if ! command -v cargo-bump &>/dev/null; then
    cargo install cargo-bump
fi

BINARY="$SCRIPT_DIR/target/release/spox"
CURRENT_VERSION=$(grep '^version' "$SCRIPT_DIR/Cargo.toml" | head -1 | grep -o '[0-9][^"]*')
if [[ ! -f "$BINARY" ]] || [[ -n "$(find "$SCRIPT_DIR/src" "$SCRIPT_DIR/skills" -newer "$BINARY" 2>/dev/null)" ]]; then
    cargo bump patch --manifest-path "$SCRIPT_DIR/Cargo.toml"
    NEW_VERSION=$(grep '^version' "$SCRIPT_DIR/Cargo.toml" | head -1 | grep -o '[0-9][^"]*')
    echo "version: $CURRENT_VERSION → $NEW_VERSION"
else
    echo "version: $CURRENT_VERSION (no changes, skipping bump)"
fi

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
