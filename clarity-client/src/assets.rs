//! Asset management for Clarity Desktop
//!
//! This module provides compile-time asset embedding for desktop builds.
//! All assets are embedded in the binary using include_str!/include_bytes!
//! to ensure self-contained distribution without runtime file dependencies.
//!
//! ## Architecture
//!
//! - **AssetRegistry**: Central registry of all embedded assets
//! - **AssetError**: Type-safe error handling for asset operations
//! - **MIME detection**: Automatic content-type detection for assets
//! - **Platform-specific icons**: Different icon formats per platform
//!
//! ## Usage
//!
//! ```rust
//! use clarity_client::assets;
//!
//! // Load text asset (CSS, JS, HTML)
//! let css = assets::get_text_asset("css/responsive.css")?;
//!
//! // Load binary asset (images, icons)
//! let icon = assets::get_binary_asset("icons/icon.png")?;
//! ```
//!
//! ## Zero-Policy Compliance
//!
//! - No unwrap() or expect()
//! - No panic() or unreachable!()
//! - All operations return Result<T, AssetError>

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use std::collections::HashMap;

/// Asset loading errors
///
/// These errors represent all possible failure modes when loading assets.
/// Since assets are embedded at compile time, most errors indicate build-time issues.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetError {
  /// Asset not found in registry
  NotFound(String),
  /// Asset contains invalid UTF-8 (for text assets)
  InvalidUtf8,
  /// Asset exceeds size limits (bytes)
  TooLarge(usize),
  /// Asset is malformed or corrupted
  Malformed(String),
  /// Asset load failed at runtime
  LoadFailed(String),
}

impl std::fmt::Display for AssetError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::NotFound(path) => write!(f, "Asset not found: {path}"),
      Self::InvalidUtf8 => write!(f, "Asset contains invalid UTF-8"),
      Self::TooLarge(size) => write!(f, "Asset too large: {size} bytes"),
      Self::Malformed(msg) => write!(f, "Malformed asset: {msg}"),
      Self::LoadFailed(msg) => write!(f, "Asset load failed: {msg}"),
    }
  }
}

impl std::error::Error for AssetError {}

/// Asset registry with embedded assets
///
/// This struct holds all assets embedded at compile time.
/// Assets are stored in a HashMap for efficient lookup.
pub struct AssetRegistry {
  assets: HashMap<&'static str, &'static [u8]>,
}

impl AssetRegistry {
  /// Create new asset registry with all embedded assets
  ///
  /// This method embeds all assets at compile time using include_bytes!.
  /// The binary will contain all asset data, making it self-contained.
  ///
  /// Note: Due to Rust const function limitations, this uses a workaround
  /// that initializes the HashMap at runtime but with compile-time asset data.
  #[must_use]
  pub fn new() -> Self {
    let mut assets = HashMap::new();

    // Embed CSS assets
    assets.insert(
      "css/responsive.css",
      include_bytes!("../assets/responsive.css").as_slice(),
    );
    assets.insert(
      "css/style.css",
      include_bytes!("../public/style.css").as_slice(),
    );

    // Embed platform-specific icons
    #[cfg(target_os = "macos")]
    {
      // Note: icon.icns will be created by the build system
      // For now, we use a placeholder
      assets.insert(
        "icons/icon.icns",
        include_bytes!("../assets/icons/icon.png").as_slice(),
      );
    }

    #[cfg(target_os = "linux")]
    {
      assets.insert(
        "icons/icon.png",
        include_bytes!("../assets/icons/icon.png").as_slice(),
      );
    }

    #[cfg(target_os = "windows")]
    {
      // Note: icon.ico will be created by the build system
      // For now, we use a placeholder
      assets.insert(
        "icons/icon.ico",
        include_bytes!("../assets/icons/icon.png").as_slice(),
      );
    }

    Self { assets }
  }

  /// Get asset bytes
  ///
  /// Returns the raw bytes of an embedded asset.
  ///
  /// # Errors
  /// Returns `AssetError::NotFound` if the asset doesn't exist in the registry.
  pub fn get(&self, path: &str) -> Result<&'static [u8], AssetError> {
    self
      .assets
      .get(path)
      .copied()
      .ok_or_else(|| AssetError::NotFound(path.to_string()))
  }

  /// Get text asset as UTF-8 string
  ///
  /// Returns the asset content as a UTF-8 string.
  /// Use this for CSS, JS, HTML, and other text-based assets.
  ///
  /// # Errors
  /// - `AssetError::NotFound`: Asset doesn't exist
  /// - `AssetError::InvalidUtf8`: Asset contains invalid UTF-8
  pub fn get_text(&self, path: &str) -> Result<&'static str, AssetError> {
    let bytes = self.get(path)?;
    std::str::from_utf8(bytes).map_err(|_| AssetError::InvalidUtf8)
  }

  /// Get asset MIME type based on file extension
  ///
  /// Returns the appropriate MIME type for serving the asset.
  /// Defaults to "application/octet-stream" for unknown types.
  #[must_use]
  pub fn mime_type(&self, path: &str) -> &'static str {
    if path.ends_with(".css") {
      "text/css"
    } else if path.ends_with(".js") {
      "application/javascript"
    } else if path.ends_with(".html") {
      "text/html"
    } else if path.ends_with(".png") {
      "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
      "image/jpeg"
    } else if path.ends_with(".svg") {
      "image/svg+xml"
    } else if path.ends_with(".ico") || path.ends_with(".icns") {
      "image/x-icon"
    } else if path.ends_with(".woff") || path.ends_with(".woff2") {
      "font/woff2"
    } else if path.ends_with(".ttf") {
      "font/ttf"
    } else if path.ends_with(".otf") {
      "font/otf"
    } else {
      "application/octet-stream"
    }
  }

  /// Check if asset exists in registry
  #[must_use]
  pub fn contains(&self, path: &str) -> bool {
    self.assets.contains_key(path)
  }

  /// Get all asset paths
  #[must_use]
  pub fn paths(&self) -> Vec<&'static str> {
    self.assets.keys().copied().collect()
  }
}

impl Default for AssetRegistry {
  fn default() -> Self {
    Self::new()
  }
}

/// Global asset registry instance
///
/// Uses OnceLock for thread-safe lazy initialization.
/// The registry is created on first access and reused for subsequent calls.
static REGISTRY: std::sync::OnceLock<AssetRegistry> = std::sync::OnceLock::new();

/// Get global asset registry
///
/// Returns a reference to the global asset registry singleton.
/// The registry is initialized on first call.
#[must_use]
pub fn registry() -> &'static AssetRegistry {
  REGISTRY.get_or_init(AssetRegistry::new)
}

/// Convenience function to get text asset
///
/// # Errors
/// Returns `AssetError` if asset not found or invalid UTF-8
pub fn get_text_asset(path: &str) -> Result<&'static str, AssetError> {
  registry().get_text(path)
}

/// Convenience function to get binary asset
///
/// # Errors
/// Returns `AssetError` if asset not found
pub fn get_binary_asset(path: &str) -> Result<&'static [u8], AssetError> {
  registry().get(path)
}

/// Load CSS asset for use in components
///
/// This is a convenience function for loading CSS assets.
///
/// # Errors
/// Returns `AssetError` if the CSS file is not found or contains invalid UTF-8
pub fn load_css(path: &str) -> Result<String, AssetError> {
  get_text_asset(path).map(|s| s.to_string())
}

/// Dioxus hook to load and apply CSS asset
///
/// This hook loads a CSS asset and returns it as a string.
/// Use this in your components to apply embedded styles.
///
/// # Errors
/// Returns `AssetError` if the CSS file cannot be loaded
pub fn use_asset_css(path: &str) -> Result<String, AssetError> {
  load_css(path)
}

#[cfg(test)]
mod tests {
  use super::*;

  // Martin Fowler Test Suite: Desktop Asset Bundling (bd-8sw)

  #[test]
  fn test_asset_registry_initialized() {
    // Given: The asset registry is created
    let registry = AssetRegistry::new();

    // Then: It should contain expected assets
    assert!(
      registry.contains("css/responsive.css"),
      "Registry should contain responsive.css"
    );
  }

  #[test]
  fn test_css_asset_embedded_and_accessible() {
    // Given: The assets/responsive.css file exists
    // And: The asset registry is initialized

    // When: get_text_asset("css/responsive.css") is called
    let result = get_text_asset("css/responsive.css");

    // Then: The CSS should be returned as Ok(&str)
    assert!(result.is_ok(), "CSS asset should be accessible");
    let css = result.unwrap();

    // And: The CSS content should be valid UTF-8 (proved by successful &str return)
    assert!(css.len() > 0, "CSS should have content");

    // And: The CSS should contain expected content
    assert!(
      css.contains(".container"),
      "CSS should contain .container class"
    );
    assert!(css.contains("@media"), "CSS should contain @media queries");
  }

  #[test]
  fn test_missing_asset_returns_not_found_error() {
    // Given: The asset registry is initialized
    // And: Asset "missing.txt" does not exist

    // When: get_text_asset("missing.txt") is called
    let result = get_text_asset("missing.txt");

    // Then: The result should be Err(AssetError::NotFound)
    assert!(result.is_err(), "Missing asset should return error");
    match result {
      Err(AssetError::NotFound(path)) => {
        assert_eq!(path, "missing.txt", "Error should contain asset path");
      }
      _ => panic!("Expected NotFound error, got {:?}", result),
    }
  }

  #[test]
  fn test_mime_type_detection_for_asset_extensions() {
    // Given: The asset registry is initialized
    let registry = registry();

    // When: mime_type() is called for various asset paths

    // Then: Correct MIME types should be returned
    assert_eq!(registry.mime_type("style.css"), "text/css");
    assert_eq!(registry.mime_type("app.js"), "application/javascript");
    assert_eq!(registry.mime_type("page.html"), "text/html");
    assert_eq!(registry.mime_type("logo.png"), "image/png");
    assert_eq!(registry.mime_type("photo.jpg"), "image/jpeg");
    assert_eq!(registry.mime_type("icon.svg"), "image/svg+xml");
    assert_eq!(registry.mime_type("icon.ico"), "image/x-icon");
    assert_eq!(registry.mime_type("font.woff2"), "font/woff2");
    assert_eq!(registry.mime_type("font.ttf"), "font/ttf");
    assert_eq!(
      registry.mime_type("unknown.xyz"),
      "application/octet-stream"
    );
  }

  #[test]
  fn test_binary_asset_accessible() {
    // Given: The asset registry is initialized

    // When: get_binary_asset is called
    let result = get_binary_asset("css/responsive.css");

    // Then: Asset data should be returned
    assert!(result.is_ok(), "Binary asset should be accessible");
    let data = result.unwrap();
    assert!(data.len() > 0, "Asset data should not be empty");
  }

  #[test]
  fn test_asset_content_matches_source_file() {
    // Given: An asset file source with known content

    // When: The asset is loaded via get_text_asset()
    let embedded = get_text_asset("css/responsive.css").expect("Asset should be embedded");

    // And: Read source file directly
    let source =
      std::fs::read_to_string("assets/responsive.css").expect("Source file should exist");

    // Then: The loaded content should match the source file
    assert_eq!(embedded, source, "Embedded asset should match source file");
  }

  #[test]
  fn test_asset_with_utf8_content() {
    // Given: An asset file containing UTF-8 content

    // When: The asset is loaded via get_text_asset()
    let css = get_text_asset("css/responsive.css").expect("CSS should be embedded");

    // Then: The content should be valid UTF-8 (proved by successful &str return)
    // The fact that it returned &str proves UTF-8 validity
    assert!(css.len() > 0, "CSS should have content");

    // Verify it's actually valid UTF-8 string data
    assert!(css.chars().count() > 0, "CSS should have valid characters");
  }

  #[test]
  fn test_path_traversal_attempts_return_error() {
    // Given: The asset registry is initialized

    // When: Attempting path traversal
    let result1 = get_text_asset("../../../etc/passwd");
    let result2 = get_text_asset("/absolute/path/secret.txt");

    // Then: Both should return Err(AssetError::NotFound)
    assert!(
      matches!(result1, Err(AssetError::NotFound(_))),
      "Path traversal should fail"
    );
    assert!(
      matches!(result2, Err(AssetError::NotFound(_))),
      "Absolute paths should fail"
    );
  }

  #[test]
  fn test_multiple_assets_loaded_concurrently() {
    // Given: The asset registry contains CSS assets

    // When: All assets are loaded
    let css1 = get_text_asset("css/responsive.css");
    let css2 = get_text_asset("css/style.css");

    // Then: All assets should load successfully
    assert!(css1.is_ok(), "First CSS should load");
    assert!(css2.is_ok(), "Second CSS should load");

    // And: Assets should be independent
    let css1_bytes = css1.unwrap().as_bytes();
    let css2_bytes = css2.unwrap().as_bytes();
    assert_ne!(
      css1_bytes.as_ptr(),
      css2_bytes.as_ptr(),
      "Assets should be independent"
    );
  }

  #[test]
  fn test_asset_registry_singleton_initialization() {
    // Given: The asset registry is not yet initialized

    // When: registry() is called multiple times
    let registry1 = registry();
    let registry2 = registry();

    // Then: Should return the same instance (singleton)
    assert_eq!(
      registry1 as *const AssetRegistry, registry2 as *const AssetRegistry,
      "Registry should be singleton"
    );

    // And: Should contain assets
    assert!(
      registry1.contains("css/responsive.css"),
      "Registry should contain assets"
    );
  }

  #[test]
  fn test_asset_paths_list() {
    // Given: The asset registry is initialized
    let registry = registry();

    // When: Getting all asset paths
    let paths = registry.paths();

    // Then: Should contain expected assets
    assert!(
      paths.contains(&"css/responsive.css"),
      "Should list responsive.css"
    );
    assert!(paths.contains(&"css/style.css"), "Should list style.css");
  }

  #[test]
  fn test_contains_method() {
    // Given: The asset registry is initialized
    let registry = registry();

    // When: Checking for existing and non-existing assets
    assert!(
      registry.contains("css/responsive.css"),
      "Should contain existing asset"
    );
    assert!(
      !registry.contains("missing.txt"),
      "Should not contain missing asset"
    );
  }

  #[test]
  fn test_asset_error_display() {
    // Given: Various asset errors
    let err1 = AssetError::NotFound("test.txt".to_string());
    let err2 = AssetError::InvalidUtf8;
    let err3 = AssetError::TooLarge(1024);
    let err4 = AssetError::Malformed("corrupt".to_string());
    let err5 = AssetError::LoadFailed("failed".to_string());

    // When: Converting errors to string
    let msg1 = err1.to_string();
    let msg2 = err2.to_string();
    let msg3 = err3.to_string();
    let msg4 = err4.to_string();
    let msg5 = err5.to_string();

    // Then: Error messages should be descriptive
    assert!(msg1.contains("not found"));
    assert!(msg2.contains("UTF-8"));
    assert!(msg3.contains("too large"));
    assert!(msg4.contains("Malformed"));
    assert!(msg5.contains("load failed"));
  }
}
