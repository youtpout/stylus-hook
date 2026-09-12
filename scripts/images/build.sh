#!/usr/bin/env bash
# Render the press images: logo, cover and the charts, as PNG.
#
#   ./scripts/images/build.sh
#
# Output: scripts/images/out/*.png
#
# Each card declares its own pixel size in manifest.txt, and Chrome is given a
# window of exactly that size, so nothing is scaled or cropped after the fact.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="$HERE/out"
CARDS="$HERE/cards.html"
CHROME="${CHROME:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}"
mkdir -p "$OUT"

while read -r i name w h; do
  [ -n "${i:-}" ] || continue
  "$CHROME" --headless --disable-gpu --hide-scrollbars \
    --window-size="$w,$h" --force-device-scale-factor=2 \
    --virtual-time-budget=1500 \
    --screenshot="$OUT/$name.png" "file://$CARDS#$i" >/dev/null 2>&1
  printf '    %-24s %sx%s @2x\n' "$name.png" "$w" "$h"
done < "$HERE/manifest.txt"

echo "done: $OUT"
