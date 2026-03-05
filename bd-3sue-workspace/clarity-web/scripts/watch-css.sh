#!/bin/bash
# Watch CSS script - automatically rebuilds Tailwind CSS when files change

cd "$(dirname "$0")/.."

echo "🎨 Watching Tailwind CSS..."
echo "Press Ctrl+C to stop"

while true; do
    npx tailwindcss -i assets/tailwind.css -o assets/output.css
    echo "✅ CSS rebuilt at $(date '+%H:%M:%S')"

    # Wait for file changes
    if command -v inotifywait >/dev/null 2>&1; then
        inotifywait -r -e modify,create,delete \
            assets/tailwind.css \
            tailwind.config.js \
            src/**/*.rs 2>/dev/null
    else
        # Fallback: sleep for 5 seconds
        sleep 5
    fi
done
