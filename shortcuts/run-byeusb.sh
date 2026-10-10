#!/usr/bin/env bash
set -euo pipefail

BIN="/var/lib/byeusb/bin/ByeUSB"

if [[ ! -x "$BIN" ]]; then
    echo "[ERROR] binary not found or not executable: $BIN" >&2
    exit 1
fi

if [[ $EUID -ne 0 ]]; then
    echo "[INFO] requesting admin privileges..."
    exec pkexec "$BIN" "$@"
fi

echo "[INFO] starting ByeUSB monitor as root"
exec "$BIN" "$@"
