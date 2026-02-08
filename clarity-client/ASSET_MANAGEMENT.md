# Asset Management Guide for Clarity Desktop

This guide explains how assets are managed and bundled in the Clarity Desktop application.

## Overview

Clarity Desktop uses compile-time asset embedding to create a self-contained binary. All assets (CSS, images, icons) are embedded in the executable at build time, eliminating runtime file dependencies.

## Architecture

### Asset Registry

The `AssetRegistry` in `src/assets.rs` manages all embedded assets:

```rust
use clarity_client::assets;

// Load text asset (CSS, JS, HTML)
let css = assets::get_text_asset("css/responsive.css")?;

// Load binary asset (images, icons)
let icon = assets::get_binary_asset("icons/icon.png")?;
```

### Compile-Time Embedding

Assets are embedded using Rust's built-in `include_bytes!` and `include_str!` macros:

```rust
// In src/assets.rs
assets.insert("css/responsive.css", include_bytes!("../assets/responsive.css"));
```

This means:
- Assets are part of the binary
- No runtime file loading
- Cross-platform compatibility
- Self-contained distribution

## Directory Structure

```
clarity-client/
├── assets/
│   ├── icons/
│   │   ├── icon.png          # Linux icon (256x256 PNG)
│   │   ├── icon.svg          # SVG icon (scalable)
│   │   └── create_icons.sh   # Icon generation script
│   ├── fonts/                # Custom fonts (future)
│   ├── images/               # Image assets (future)
│   └── responsive.css        # Responsive styles
├── public/
│   └── style.css             # Web styles
└── src/
    └── assets.rs             # Asset registry
```

## Adding New Assets

### 1. Place Asset File

Put your asset in the appropriate directory:

```bash
# CSS files
cp my-styles.css clarity-client/assets/

# Images
cp logo.png clarity-client/assets/images/

# Icons
cp app-icon.svg clarity-client/assets/icons/
```

### 2. Register in AssetRegistry

Edit `src/assets.rs` and add the asset:

```rust
pub const fn new() -> Self {
    let mut assets = HashMap::new();

    // Existing assets...
    assets.insert("css/responsive.css", include_bytes!("../assets/responsive.css"));

    // Add your new asset
    assets.insert("images/logo.png", include_bytes!("../assets/images/logo.png"));

    Self { assets }
}
```

### 3. Use in Components

```rust
use dioxus::prelude::*;
use clarity_client::assets;

#[component]
fn MyComponent() -> Element {
    let logo = assets::get_binary_asset("images/logo.png").ok();

    rsx! {
        div {
            img { src: "data:image/png;base64,{logo}" }
        }
    }
}
```

## Asset Types

### CSS Assets

Text-based stylesheets:

```rust
let css = assets::get_text_asset("css/responsive.css")?;
```

### Binary Assets

Images, icons, fonts:

```rust
let image = assets::get_binary_asset("images/logo.png")?;
```

### Platform-Specific Assets

Different assets per platform:

```rust
#[cfg(target_os = "macos")]
assets.insert("icons/icon.icns", include_bytes!("../assets/icons/icon.icns"));

#[cfg(target_os = "linux")]
assets.insert("icons/icon.png", include_bytes!("../assets/icons/icon.png"));

#[cfg(target_os = "windows")]
assets.insert("icons/icon.ico", include_bytes!("../assets/icons/icon.ico"));
```

## Best Practices

### 1. Asset Size Limits

- CSS: < 100 KB
- Icons: < 1 MB each
- Images: < 5 MB each
- Total embedded: < 10 MB

Large assets should be loaded at runtime (future enhancement).

### 2. Asset Validation

Always validate assets at compile time:

```rust
#[test]
fn test_assets_exist() {
    assert!(assets::get_text_asset("css/responsive.css").is_ok());
}
```

### 3. MIME Type Detection

The asset registry automatically detects MIME types:

```rust
let registry = assets::registry();
assert_eq!(registry.mime_type("style.css"), "text/css");
assert_eq!(registry.mime_type("logo.png"), "image/png");
```

### 4. Error Handling

All asset operations return `Result<T, AssetError>`:

```rust
match assets::get_text_asset("missing.css") {
    Ok(css) => println!("Loaded: {}", css),
    Err(AssetError::NotFound(path)) => eprintln!("Not found: {}", path),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Building for Distribution

### Development Build

```bash
# Regular development build
moon run :clarity-client:build

# Run directly
moon run :clarity-client:run
```

### Release Build

```bash
# Optimized release build
moon run :clarity-client:build --release

# Binary will be at:
# Linux: target/release/clarity-client
# macOS: target/release/clarity-client
# Windows: target/release/clarity-client.exe
```

### Platform-Specific Builds

```bash
# Linux
cargo build --release --target x86_64-unknown-linux-gnu

# macOS (Apple Silicon)
cargo build --release --target aarch64-apple-darwin

# macOS (Intel)
cargo build --release --target x86_64-apple-darwin

# Windows
cargo build --release --target x86_64-pc-windows-msvc
```

## Icon Requirements

### Linux (PNG)

- Size: 256x256 pixels minimum
- Format: PNG with transparency
- Path: `assets/icons/icon.png`

Create with:

```bash
cd clarity-client/assets/icons
./create_icons.sh
```

### macOS (ICNS)

- Required for App Store distribution
- Multiple sizes in one file
- Tool: Use `iconutil` (macOS) or online converters

### Windows (ICO)

- Multiple sizes in one file
- Sizes: 16x16, 32x32, 48x48, 256x256
- Tool: Use GIMP or online converters

## Testing

### Unit Tests

Test asset loading:

```bash
moon run :clarity-client:test
```

### Integration Tests

Test that binary is self-contained:

```bash
# Build release binary
cargo build --release

# Test on clean system (without assets/ directory)
./target/release/clarity-client
```

### Manual Testing

1. Verify CSS loads correctly
2. Check icons display properly
3. Test on all target platforms
4. Verify binary size is acceptable

## Troubleshooting

### Asset Not Found

**Problem**: `AssetError::NotFound`

**Solution**:
1. Verify file exists in `assets/` directory
2. Check asset is registered in `AssetRegistry::new()`
3. Run `cargo clean && cargo build` to rebuild

### Invalid UTF-8

**Problem**: `AssetError::InvalidUtf8`

**Solution**:
1. Ensure text files are UTF-8 encoded
2. Check for BOM (byte order mark)
3. Use `get_binary_asset()` for non-text files

### Binary Too Large

**Problem**: Binary size exceeds expectations

**Solution**:
1. Check asset sizes in `assets/` directory
2. Compress large images before embedding
3. Consider runtime loading for very large assets

### Platform-Specific Issues

**macOS**: Notarization may be required for distribution
**Windows**: SmartScreen may warn on first run
**Linux**: May need desktop entry file for app menu

## Future Enhancements

1. **Runtime Asset Loading**: Load large assets on demand
2. **Asset Compression**: Compress assets before embedding
3. **Asset Versioning**: Track asset versions in binary
4. **Hot Reload**: Automatic asset reloading in development
5. **CDN Support**: Optional CDN for large assets

## Resources

- [Dioxus Bundling Guide](https://dioxuslabs.com/learn/0.7/guides/deploy/config/)
- [Rust include_str! Documentation](https://doc.rust-lang.org/std/macro.include_str.html)
- [PNG Specification](https://www.w3.org/TR/PNG/)
- [Linux Desktop Entry Specification](https://specifications.freedesktop.org/desktop-entry-spec/)

## Contributing

When adding new assets:

1. Follow the directory structure
2. Keep file sizes reasonable
3. Add tests for new assets
4. Update this documentation
5. Ensure zero-panic compliance

## License

Assets are distributed under the same license as the Clarity project (MIT).
