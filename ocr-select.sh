#!/usr/bin/env bash
# Select a screen region, OCR it, copy text to clipboard

TMP="/tmp/ocr_region_$(date +%s).png"

hyprshot -m region -o /tmp -f "$(basename "$TMP")" -s

[ -f "$TMP" ] || exit 0

TEXT=$(tesseract "$TMP" stdout 2>/dev/null)
rm -f "$TMP"

TEXT=$(echo "$TEXT" | sed '/^$/d')

if [ -z "$TEXT" ]; then
    notify-send "OCR" "No text detected"
    exit 0
fi

echo -n "$TEXT" | wl-copy
notify-send "OCR copied to clipboard" "$TEXT"
