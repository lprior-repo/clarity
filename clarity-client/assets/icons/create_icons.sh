#!/bin/bash
# Create placeholder icons for Clarity Desktop
# This script generates simple icon files for development/testing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ASSETS_DIR="$SCRIPT_DIR"
ICON_SIZES="16 32 48 64 128 256"

echo "Creating placeholder icons for Clarity Desktop..."

# Create a simple SVG icon
cat > "$ASSETS_DIR/icon.svg" << 'SVG_EOF'
<?xml version="1.0" encoding="UTF-8"?>
<svg width="256" height="256" viewBox="0 0 256 256" xmlns="http://www.w3.org/2000/svg">
  <rect width="256" height="256" fill="#2563eb"/>
  <text x="128" y="160" font-family="Arial, sans-serif" font-size="160" 
        fill="white" text-anchor="middle" font-weight="bold">C</text>
</svg>
SVG_EOF

echo "Created icon.svg"

# For now, we'll create a simple PNG using Python if available
if command -v python3 &> /dev/null; then
    python3 << 'PYTHON_EOF'
from PIL import Image, ImageDraw, ImageFont
import os

script_dir = os.path.dirname(os.path.abspath(__file__))
os.makedirs(script_dir, exist_ok=True)

# Create a simple blue square with "C"
for size in [16, 32, 48, 64, 128, 256]:
    img = Image.new('RGB', (size, size), color='#2563eb')
    draw = ImageDraw.Draw(img)
    
    # Draw "C" text
    font_size = int(size * 0.6)
    try:
        font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", font_size)
    except:
        font = ImageFont.load_default()
    
    text = "C"
    bbox = draw.textbbox((0, 0), text, font=font)
    text_width = bbox[2] - bbox[0]
    text_height = bbox[3] - bbox[1]
    
    position = ((size - text_width) // 2, (size - text_height) // 2)
    draw.text(position, text, fill='white', font=font)
    
    img.save(os.path.join(script_dir, f'icon.png'))
    print(f"Created {size}x{size} icon.png")
    break  # Only create 256x256 for now

print("Icon creation complete")
PYTHON_EOF
else
    echo "Python3/PIL not available, creating minimal placeholder"
    # Create a minimal valid PNG
    # For now, copy an existing asset as placeholder
    if [ -f "$ASSETS_DIR/../responsive.css" ]; then
        # This is just a placeholder - in production, you'd use real icons
        touch "$ASSETS_DIR/icon.png"
        echo "Created placeholder icon.png (empty file - replace with real icon)"
    fi
fi

echo "Icon creation complete"
echo "Note: For production, replace these with professionally designed icons"
