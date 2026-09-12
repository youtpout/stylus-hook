#!/usr/bin/env bash
# Render the project's media: the logo, the cover and the charts.
#
#   ./scripts/images/build.sh
#
# Output: media/*.png — committed, because a logo the README points at should not
# need a build step to exist.
#
# Each card declares its own pixel size and background in manifest.txt, and Chrome
# is given a window of exactly that size, so nothing is scaled or cropped after the
# fact. Cards marked `transparent` keep their alpha, which is what makes the logo
# usable on any background.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
OUT="$ROOT/media"
CARDS="$HERE/cards.html"
CHROME="${CHROME:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}"
mkdir -p "$OUT"

while read -r i name w h bg; do
  [ -n "${i:-}" ] || continue
  args=(--headless --disable-gpu --hide-scrollbars
        --window-size="$w,$h" --force-device-scale-factor=2
        --virtual-time-budget=1500 --screenshot="$OUT/$name.png")
  [ "$bg" = transparent ] && args+=(--default-background-color=00000000)
  "$CHROME" "${args[@]}" "file://$CARDS#$i" >/dev/null 2>&1
  printf '    %-24s %sx%s @2x %s\n' "$name.png" "$w" "$h" "$bg"
done < "$HERE/manifest.txt"

echo "done: $OUT"
