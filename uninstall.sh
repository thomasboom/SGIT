#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="${SGIT_INSTALL_DIR:-${HOME}/.local/bin}"
TARGET_PATH="$INSTALL_DIR/sgit"

if [[ ! -e "$TARGET_PATH" ]]; then
    echo "SGIT is not installed at $TARGET_PATH."
    exit 1
fi

rm -f "$TARGET_PATH"

echo "SGIT removed from $TARGET_PATH."
