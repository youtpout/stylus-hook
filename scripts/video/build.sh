#!/usr/bin/env bash
# Build the hackathon submission video from the slide deck.
#
#   ./scripts/video/build.sh            # everything
#   ./scripts/video/build.sh shots      # re-shoot the slides only
#   ./scripts/video/build.sh voice      # re-record narration only
#
# Output: scripts/video/out/stylus-hook.mp4 — 1920x1080, H.264, AAC.
#
# There is no live app to film. The deck is the film, so every frame comes from
# `docs/deck.html?video#N` rendered headless: no cursor, no browser chrome, and
# the same file that is published on the docs site.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
OUT="$HERE/out"
DECK="$ROOT/docs/deck.html"
CHROME="${CHROME:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}"

mkdir -p "$OUT/png" "$OUT/mp3" "$OUT/seg"
step="${1:-all}"
export LC_NUMERIC=C

# --- 1. one PNG per slide ----------------------------------------------------
# The deck reads its slide index off the URL hash, so each still is a clean
# 1080p frame. `?video` drops the progress bar, the help line and the card
# border, which are for a presenter and not for a viewer.
if [[ "$step" == "all" || "$step" == "shots" ]]; then
  echo "==> shooting slides"
  n=$(grep -c '<section class="slide"' "$DECK")
  for ((i=0; i<n; i++)); do
    "$CHROME" --headless --disable-gpu --hide-scrollbars \
      --window-size=1920,1080 --force-device-scale-factor=1 \
      --virtual-time-budget=1500 \
      --screenshot="$OUT/png/$(printf '%02d' $((i+1))).png" \
      "file://$DECK?video#$i" >/dev/null 2>&1
  done
  echo "    $n slides -> $OUT/png"
fi

# --- 2. narration ------------------------------------------------------------
if [[ "$step" == "all" || "$step" == "voice" ]]; then
  echo "==> recording narration"
  python3 "$HERE/say.py" "$HERE/narration.json" "$OUT/mp3"
fi

# --- 3. one segment per slide, then concatenate ------------------------------
# Each still is held for exactly as long as its narration, so picture and voice
# cannot drift. `-t` rather than `-shortest`: with a looped still, `-shortest`
# runs on to the end of its GOP and welds ~2s of silence onto every slide.
echo "==> assembling"
: > "$OUT/list.txt"
for png in "$OUT/png"/*.png; do
  id=$(basename "$png" .png)
  mp3=$(ls "$OUT/mp3/$id"-*.mp3 2>/dev/null | head -1 || true)
  [[ -n "$mp3" ]] || { echo "    !! no narration for slide $id"; exit 1; }
  len=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$mp3")
  ffmpeg -y -loglevel error -loop 1 -i "$png" -i "$mp3" \
    -t "$len" -c:v libx264 -pix_fmt yuv420p -r 30 -tune stillimage \
    -c:a aac -b:a 160k "$OUT/seg/$id.mp4"
  printf '    %s  %5.1fs\n' "$id" "$len"
  echo "file '$OUT/seg/$id.mp4'" >> "$OUT/list.txt"
done

ffmpeg -y -loglevel error -f concat -safe 0 -i "$OUT/list.txt" \
  -c:v libx264 -pix_fmt yuv420p -crf 18 -preset slow -c:a aac -b:a 160k \
  "$OUT/stylus-hook.mp4"

dur=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$OUT/stylus-hook.mp4")
printf '\ndone: %s\n' "$OUT/stylus-hook.mp4"
awk -v d="$dur" 'BEGIN{printf "length: %d:%02d  (%.1fs)\n", d/60, d%60, d}'
