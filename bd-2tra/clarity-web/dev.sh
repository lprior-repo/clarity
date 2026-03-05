#!/bin/bash
# Development server wrapper for clarity-web
# Handles Tailwind CSS asset copying automatically

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🎨 Ensuring Tailwind CSS assets are available...${NC}"

# Function to copy CSS to build output
copy_css() {
    local source="assets/tailwind.css"
    local dest_dir="target/dx/clarity-web/debug/web/public/assets"

    if [ -f "$source" ]; then
        mkdir -p "$dest_dir"
        cp "$source" "$dest_dir/"
        echo -e "${GREEN}✓ Copied $source to $dest_dir/${NC}"
    else
        echo "Warning: $source not found. Dioxus will create it on first build."
    fi
}

# Copy CSS initially
copy_css

# Function to watch for CSS changes and copy
watch_css() {
    local source="assets/tailwind.css"
    local dest_dir="target/dx/clarity-web/debug/web/public/assets"

    if command -v inotifywait &> /dev/null; then
        echo -e "${BLUE}👀 Watching for CSS changes...${NC}"
        while inotifywait -q -e modify "$source" 2>/dev/null; do
            if [ -f "$source" ]; then
                mkdir -p "$dest_dir"
                cp "$source" "$dest_dir/"
                echo -e "${GREEN}✓ CSS updated${NC}"
            fi
        done &
    else
        echo -e "${BLUE}💡 Install inotify-tools for CSS hot-reload${NC}"
    fi
}

# Start CSS watcher in background
watch_css

# Start dx serve
echo -e "${BLUE}🚀 Starting Dioxus dev server...${NC}"
dx serve --platform web --package clarity-web --port 3000
