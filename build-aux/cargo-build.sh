#!/bin/bash
set -euo pipefail

SOURCE_ROOT="$1"
OUTPUT="$(realpath -m "$2")"

cd "$SOURCE_ROOT"

if [ "${MESON_BUILD_TYPE:-release}" = "debug" ]; then
    cargo build
    cp target/debug/larmindon-gtk "$OUTPUT"
else
    cargo build --release
    cp target/release/larmindon-gtk "$OUTPUT"
fi
