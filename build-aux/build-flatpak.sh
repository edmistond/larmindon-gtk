#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
APP_ID="com.davidedmiston.Larmindon"
OUTPUT="${1:-$PROJECT_ROOT/../$APP_ID.flatpak}"

cd "$PROJECT_ROOT"

echo "Building Flatpak..."
flatpak-builder --user --force-clean .flatpak-build "$APP_ID.yml"

echo "Exporting bundle..."
REPO=$(mktemp -d)
flatpak build-export "$REPO" .flatpak-build
flatpak build-bundle "$REPO" "$OUTPUT" "$APP_ID"
rm -rf "$REPO" .flatpak-build

echo "Done: $OUTPUT"
