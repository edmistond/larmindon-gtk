#!/bin/bash
# Generate sized icons from the source PNG, making white corner areas transparent.
#
# Usage: ./generate-icons.sh [source.png]
#   Defaults to com.davidedmiston.Larmindon.Source.png in the same directory.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_ID="com.davidedmiston.Larmindon"
SOURCE="${1:-${SCRIPT_DIR}/${APP_ID}.Source.png}"
SIZES=(256 128 64 48)

if [ ! -f "$SOURCE" ]; then
    echo "Error: source icon not found: $SOURCE" >&2
    exit 1
fi

echo "Source: $SOURCE ($(identify -format '%wx%h' "$SOURCE"))"

# Make white/near-white corners transparent via flood-fill from each corner,
# then resize to each target size.
for size in "${SIZES[@]}"; do
    out="${SCRIPT_DIR}/${APP_ID}-${size}.png"
    magick "$SOURCE" \
        -fuzz 10% \
        -fill none \
        -floodfill +0+0 white \
        -floodfill +1023+0 white \
        -floodfill +0+1023 white \
        -floodfill +1023+1023 white \
        -resize "${size}x${size}" \
        "$out"
    echo "  ${size}x${size} -> $out"
done

echo "Done."
