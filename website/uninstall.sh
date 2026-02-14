#!/bin/sh
set -eu

INSTALL_DIR="${SGIT_INSTALL_DIR:-${HOME}/.local/bin}"
TARGET_PATH="$INSTALL_DIR/sgit"

status() {
    printf "\r\033[K%s" "$1"
}

if [ ! -e "$TARGET_PATH" ]; then
    printf "\r\033[KSGIT is not installed at %s\n" "$TARGET_PATH"
    exit 1
fi

status "Uninstalling SGIT..."
rm -f "$TARGET_PATH"

printf "\r\033[K👋 SGIT has been uninstalled 👋\n"
