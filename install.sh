#!/usr/bin/env bash
set -euo pipefail

# Build SGIT and install the released binary into a location on PATH.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_BIN="$SCRIPT_DIR/sgit/target/release/sgit"
INSTALL_DIR="${SGIT_INSTALL_DIR:-${HOME}/.local/bin}"

echo "Building SGIT..."
cargo build --release --manifest-path "$SCRIPT_DIR/sgit/Cargo.toml"

if [[ ! -f "$TARGET_BIN" ]]; then
    echo "ERROR: release binary not found at $TARGET_BIN"
    exit 1
fi

echo "Installing SGIT to $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"

if install -Dm755 "$TARGET_BIN" "$INSTALL_DIR/sgit"; then
    echo "SGIT installed at $INSTALL_DIR/sgit."
else
    echo "Install failed; try setting SGIT_INSTALL_DIR to a writable directory (e.g., \$HOME/.local/bin) or rerun with sudo."
    exit 1
fi

echo "Re-run this script to rebuild and update the binary."
