#!/usr/bin/env bash
set -euo pipefail

BIN="/var/lib/byeusb/bin/ByeUSBMsg"

if [[ ! -x "$BIN" ]]; then
    echo "[ERROR] binary not found or not executable: $BIN" >&2
    exit 1
fi

if [[ $EUID -ne 0 ]]; then
    exec pkexec env DISPLAY="$DISPLAY" XAUTHORITY="$XAUTHORITY" "$BIN" "$@"
fi

echo "[INFO] starting ByeUSBMsg as root"
exec "$BIN" "$@"
