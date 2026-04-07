#!/usr/bin/env bash
# Take a region screenshot with hyprshot, then open the annotation editor

SCREENSHOT_DIR="/tmp"
SCREENSHOT_FILE="screenshot_$(date +%Y%m%d_%H%M%S).png"
SCREENSHOT_PATH="${SCREENSHOT_DIR}/${SCREENSHOT_FILE}"

# Take the screenshot (-s suppresses hyprshot notification)
hyprshot -m region -o "$SCREENSHOT_DIR" -f "$SCREENSHOT_FILE" -s

# If hyprshot was cancelled (no file produced), exit
[ -f "$SCREENSHOT_PATH" ] || exit 0

# Wait for file to be fully written (size stops changing)
prev_size=-1
while true; do
    curr_size=$(stat -c%s "$SCREENSHOT_PATH" 2>/dev/null) || exit 0
    [ "$curr_size" = "$prev_size" ] && break
    prev_size=$curr_size
    sleep 0.05
done

# Launch the annotation editor (preview mode: small thumbnail, click to expand)
"$HOME/.local/bin/hyprsnap" --preview "$SCREENSHOT_PATH"
