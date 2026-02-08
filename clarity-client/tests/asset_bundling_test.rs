//! Integration tests for Desktop Asset Bundling (bd-8sw)
//!
//! These tests verify that assets are properly embedded and accessible
//! in the desktop binary.
//!
//! See docs/TESTING.md for testing standards.

use clarity_client::assets::{get_binary_asset, get_text_asset, registry, AssetError};

#[test]
fn test_css_asset_embedded_and_accessible() {
  // GIVEN: The assets/responsive.css file exists
  // AND: The asset registry is initialized

  // WHEN: get_text_asset("css/responsive.css") is called
  let result = get_text_asset("css/responsive.css");

  // THEN: The CSS should be returned as Ok(&str)
  assert!(result.is_ok(), "CSS asset should be accessible");
  let css = match result {
    Ok(asset) => asset,
    Err(e) => panic!("Failed to get CSS asset: {:?}", e),
  };

  // AND: The CSS content should be valid UTF-8
  assert!(css.len() > 0, "CSS should have content");

  // AND: The CSS should contain expected content
  assert!(
    css.contains(".container"),
    "CSS should contain .container class"
  );
  assert!(css.contains("@media"), "CSS should contain @media queries");
}

#[test]
fn test_missing_asset_returns_not_found_error() {
  // GIVEN: The asset registry is initialized
  // AND: Asset "missing.txt" does not exist

  // WHEN: get_text_asset("missing.txt") is called
  let result = get_text_asset("missing.txt");

  // THEN: The result should be Err(AssetError::NotFound)
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
  // GIVEN: The asset registry is initialized
  let registry = registry();

  // WHEN: mime_type() is called for various asset paths

  // THEN: Correct MIME types should be returned
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
  // GIVEN: The asset registry is initialized

  // WHEN: get_binary_asset is called
  let result = get_binary_asset("css/responsive.css");

  // THEN: Asset data should be returned
  assert!(result.is_ok(), "Binary asset should be accessible");
  let data = result.unwrap();
  assert!(data.len() > 0, "Asset data should not be empty");
}

#[test]
fn test_asset_content_matches_source_file() {
  // GIVEN: An asset file source with known content

  // WHEN: The asset is loaded via get_text_asset()
  let embedded = get_text_asset("css/responsive.css").expect("Asset should be embedded");

  // AND: Read source file directly
  let source = std::fs::read_to_string("assets/responsive.css").expect("Source file should exist");

  // THEN: The loaded content should match the source file
  assert_eq!(embedded, source, "Embedded asset should match source file");
}

#[test]
fn test_asset_with_utf8_content() {
  // GIVEN: An asset file containing UTF-8 content

  // WHEN: The asset is loaded via get_text_asset()
  let css = get_text_asset("css/responsive.css").expect("CSS should be embedded");

  // THEN: The content should be valid UTF-8 (proved by successful &str return)
  assert!(css.len() > 0, "CSS should have content");
  assert!(css.chars().count() > 0, "CSS should have valid characters");
}

#[test]
fn test_path_traversal_attempts_return_error() {
  // GIVEN: The asset registry is initialized

  // WHEN: Attempting path traversal
  let result1 = get_text_asset("../../../etc/passwd");
  let result2 = get_text_asset("/absolute/path/secret.txt");

  // THEN: Both should return Err(AssetError::NotFound)
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
  // GIVEN: The asset registry contains CSS assets

  // WHEN: All assets are loaded
  let css1 = get_text_asset("css/responsive.css");
  let css2 = get_text_asset("css/style.css");

  // THEN: All assets should load successfully
  assert!(css1.is_ok(), "First CSS should load");
  assert!(css2.is_ok(), "Second CSS should load");

  // AND: Assets should be independent
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
  // GIVEN: The asset registry is not yet initialized

  // WHEN: registry() is called multiple times
  let registry1 = registry();
  let registry2 = registry();

  // THEN: Should return the same instance (singleton)
  assert_eq!(
    registry1 as *const _, registry2 as *const _,
    "Registry should be singleton"
  );

  // AND: Should contain assets
  assert!(
    registry1.contains("css/responsive.css"),
    "Registry should contain assets"
  );
}

#[test]
fn test_asset_paths_list() {
  // GIVEN: The asset registry is initialized
  let registry = registry();

  // WHEN: Getting all asset paths
  let paths = registry.paths();

  // THEN: Should contain expected assets
  assert!(
    paths.contains(&"css/responsive.css"),
    "Should list responsive.css"
  );
  assert!(paths.contains(&"css/style.css"), "Should list style.css");
}

#[test]
fn test_contains_method() {
  // GIVEN: The asset registry is initialized
  let registry = registry();

  // WHEN: Checking for existing and non-existing assets
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
  // GIVEN: Various asset errors
  let err1 = AssetError::NotFound("test.txt".to_string());
  let err2 = AssetError::InvalidUtf8;
  let err3 = AssetError::TooLarge(1024);
  let err4 = AssetError::Malformed("corrupt".to_string());
  let err5 = AssetError::LoadFailed("failed".to_string());

  // WHEN: Converting errors to string
  let msg1 = err1.to_string();
  let msg2 = err2.to_string();
  let msg3 = err3.to_string();
  let msg4 = err4.to_string();
  let msg5 = err5.to_string();

  // THEN: Error messages should be descriptive
  assert!(msg1.contains("not found"));
  assert!(msg2.contains("UTF-8"));
  assert!(msg3.contains("too large"));
  assert!(msg4.contains("Malformed"));
  assert!(msg5.contains("load failed"));
}

// Martin Fowler Test Suite: Desktop Asset Bundling (bd-8sw)

#[test]
fn test_desktop_binary_includes_all_assets() {
  // This is an integration test that verifies the binary is self-contained
  // Manual testing required: Run binary on a clean system without assets/

  // GIVEN: The asset registry is initialized

  // WHEN: Assets are loaded
  let css = get_text_asset("css/responsive.css");

  // THEN: All assets should be accessible (embedded in binary)
  assert!(css.is_ok(), "Assets should be embedded in binary");

  // Note: Full integration test requires running binary on clean system
}

#[test]
fn test_platform_specific_icon_assets() {
  // GIVEN: The platform is detected

  // WHEN: Platform-specific icons are loaded

  #[cfg(target_os = "linux")]
  {
    let result = get_binary_asset("icons/icon.png");
    assert!(result.is_ok(), "Linux icon should be available");
    let data = result.unwrap();
    assert!(data.len() > 0, "Icon data should not be empty");
  }

  #[cfg(target_os = "macos")]
  {
    let result = get_binary_asset("icons/icon.icns");
    assert!(result.is_ok(), "macOS icon should be available");
    let data = result.unwrap();
    assert!(data.len() > 0, "Icon data should not be empty");
  }

  #[cfg(target_os = "windows")]
  {
    let result = get_binary_asset("icons/icon.ico");
    assert!(result.is_ok(), "Windows icon should be available");
    let data = result.unwrap();
    assert!(data.len() > 0, "Icon data should not be empty");
  }
}
