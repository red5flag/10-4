#!/bin/bash
# Pi-Kiosk browser wrapper
# Finds an available Chromium-based browser and launches it in kiosk mode.

BROWSER=""
for candidate in chromium-browser chromium google-chrome chrome; do
    if command -v "$candidate" &>/dev/null; then
        BROWSER="$candidate"
        break
    fi
done

if [ -z "$BROWSER" ]; then
    echo "ERROR: No Chromium-based browser found. Install one of: chromium-browser, chromium, google-chrome" >&2
    exit 1
fi

# Ensure DISPLAY is set
export DISPLAY="${DISPLAY:-:0}"

exec "$BROWSER" \
    --kiosk \
    --noerrdialogs \
    --disable-translate \
    --no-first-run \
    --fast \
    --fast-start \
    --disable-gpu \
    --no-sandbox \
    http://127.0.0.1:3000
