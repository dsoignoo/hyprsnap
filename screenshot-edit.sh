#!/usr/bin/env bash
# Take a region screenshot with hyprshot, then open the annotation editor

SCREENSHOT_DIR="/tmp"
SCREENSHOT_FILE="screenshot_$(date +%Y%m%d_%H%M%S).png"
SCREENSHOT_PATH="${SCREENSHOT_DIR}/${SCREENSHOT_FILE}"

# Take the screenshot (-s suppresses hyprshot notification)
hyprshot -m region -o "$SCREENSHOT_DIR" -f "$SCREENSHOT_FILE" -s

# If hyprshot was cancelled (no file produced), exit
[ -f "$SCREENSHOT_PATH" ] || exit 0

# Launch the annotation editor
hyprsnap "$SCREENSHOT_PATH"
