#!/usr/bin/env bash
set -euo pipefail

REPO_URL="https://github.com/ThomasNowProductions/SGIT.git"
INSTALL_DIR="${SGIT_INSTALL_DIR:-${HOME}/.local/bin}"
TEMP_DIR="$(mktemp -d)"

cleanup() {
    rm -rf "$TEMP_DIR" 2>/dev/null
}
trap cleanup EXIT

status() {
    printf "\r\033[K%s" "$1"
}

status "Cloning SGIT repository..."
git clone --depth 1 "$REPO_URL" "$TEMP_DIR" 2>/dev/null

status "Building SGIT (this may take a moment)..."
cargo build --release --manifest-path "$TEMP_DIR/sgit/Cargo.toml" 2>/dev/null >/dev/null

TARGET_BIN="$TEMP_DIR/sgit/target/release/sgit"
if [[ ! -f "$TARGET_BIN" ]]; then
    printf "\r\033[KERROR: release binary not found\n"
    exit 1
fi

status "Installing SGIT to $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"
if install -Dm755 "$TARGET_BIN" "$INSTALL_DIR/sgit" 2>/dev/null; then
    printf "\r\033[K🎉 SGIT is installed 🎉\n"
else
    printf "\r\033[KInstall failed; try setting SGIT_INSTALL_DIR to a writable directory\n"
    exit 1
fi
