#!/bin/bash
set -euo pipefail

BUILD_DIR="$1"
ICON_BASE="${DESTDIR:-}$2"
APP_ID="$3"
shift 3

for SIZE in "$@"; do
    DEST="${ICON_BASE}/${SIZE}x${SIZE}/apps"
    mkdir -p "$DEST"
    cp "${BUILD_DIR}/${APP_ID}-${SIZE}.png" "${DEST}/${APP_ID}.png"
done
