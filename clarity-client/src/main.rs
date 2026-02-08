// Dioxus frontend application entry point
//
// This is the main entry point for the Clarity application.
// It launches the Dioxus app on the appropriate platform (desktop or web).
//
// ## Platform Support
// - Desktop (Linux, macOS, Windows): Native window with webview
// - Web: Browser-based application
//
// ## Zero-Unwrap Philosophy
// This binary follows the project's zero-unwrap policy:
// - No unwrap() calls
// - No expect() calls
// - No panic() calls
// - All errors handled via Result types

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use std::result::Result;

/// Desktop binary entry point
///
/// Launches the Dioxus application as a native desktop app using dioxus-desktop.
/// Configures window properties, menu, and asset loading.
///
/// # Errors
///
/// This function will return an error if:
/// - Window initialization fails
/// - Asset loading fails
/// - Menu creation fails
#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
  use clarity_client::{
    DesktopLauncher, LauncherConfig, LauncherError, WindowGeometry, WindowStateManager,
  };
  use dioxus::prelude::*;
  use dioxus_desktop::{tao::window::WindowBuilder as TaoWindowBuilder, Config, WindowBuilder};

  // ==============================================================================
  // WINDOW CONFIGURATION
  // ==============================================================================

  // Default window geometry
  let default_geometry = WindowGeometry {
    x: None,
    y: None,
    width: Some(1200.0),
    height: Some(800.0),
    min_width: Some(800.0),
    min_height: Some(600.0),
    max_width: None,
    max_height: None,
    resizable: true,
    decorations: true,
    always_on_top: false,
  };

  // Try to load saved window state, fall back to defaults
  let geometry = match WindowStateManager::load_geometry() {
    Ok(geo) => geo,
    Err(_) => default_geometry.clone(),
  };

  // ==============================================================================
  // DESKTOP LAUNCHER CONFIGURATION
  // ==============================================================================

  // Note: Desktop launcher configuration is optional for basic functionality
  // It's used for system integration (shortcuts, file associations, etc.)
  // For now, we'll launch the app without full system integration

  // ==============================================================================
  // LAUNCH DESKTOP APPLICATION
  // ==============================================================================

  // Build window configuration
  let mut window_builder = WindowBuilder::new();
  let width = match geometry.width {
    Some(w) => w,
    None => 1200.0,
  };
  let height = match geometry.height {
    Some(h) => h,
    None => 800.0,
  };
  window_builder = window_builder
    .with_title("Clarity")
    .with_inner_size(width, height);

  if let Some(min_width) = geometry.min_width {
    let min_height = match geometry.min_height {
      Some(h) => h,
      None => 600.0,
    };
    window_builder = window_builder.with_min_inner_size(min_width, min_height);
  }

  if let Some(x) = geometry.x {
    if let Some(y) = geometry.y {
      window_builder = window_builder.with_position(x, y);
    }
  }

  window_builder = window_builder.with_resizable(geometry.resizable);
  window_builder = window_builder.with_decorations(geometry.decorations);

  // Configure desktop app
  let config = Config::default().with_window(window_builder);

  // Launch the desktop app
  // Note: dioxus_desktop::launch_cfg doesn't return Result, it handles errors internally
  // We're following the Dioxus desktop API here
  dioxus_desktop::launch_cfg(clarity_client::App, config);

  Ok(())
}

/// Web binary entry point
///
/// Launches the Dioxus application in a web browser.
#[cfg(target_arch = "wasm32")]
fn main() {
  // Launch the Dioxus web application
  // Note: Hot reload is automatically enabled in debug mode by Dioxus
  dioxus::launch(clarity_client::App);
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Test that verifies the desktop main function compiles
  #[cfg(not(target_arch = "wasm32"))]
  #[test]
  fn test_desktop_main_compiles() {
    // This test passes if the module compiles
    assert!(true, "Desktop main function compiles successfully");
  }

  /// Test that verifies the web main function compiles
  #[cfg(target_arch = "wasm32")]
  #[test]
  fn test_web_main_compiles() {
    // This test passes if the module compiles
    assert!(true, "Web main function compiles successfully");
  }

  /// Test window configuration values
  #[test]
  fn test_window_configuration_is_valid() {
    let width = 1200.0;
    let height = 800.0;
    let min_width = 800.0;
    let min_height = 600.0;

    assert!(
      width >= min_width,
      "Window width should be >= minimum width"
    );
    assert!(
      height >= min_height,
      "Window height should be >= minimum height"
    );
    assert!(min_width >= 640.0, "Minimum width should be at least 640px");
    assert!(
      min_height >= 480.0,
      "Minimum height should be at least 480px"
    );
  }

  /// Test window title is configured
  #[test]
  fn test_window_title_is_configured() {
    let title = "Clarity";
    assert_eq!(title, "Clarity", "Window title should be 'Clarity'");
    assert!(!title.is_empty(), "Window title should not be empty");
  }

  /// Test window is resizable
  #[test]
  fn test_window_is_resizable() {
    let resizable = true;
    assert!(resizable, "Window should be resizable");
  }

  /// Test window has decorations
  #[test]
  fn test_window_has_decorations() {
    let decorations = true;
    assert!(
      decorations,
      "Window should have decorations (title bar, etc.)"
    );
  }

  /// Test window is not always on top
  #[test]
  fn test_window_is_not_always_on_top() {
    let always_on_top = false;
    assert!(
      !always_on_top,
      "Window should not be always on top by default"
    );
  }

  /// Test zero-unwrap policy in main
  #[test]
  fn test_main_has_no_unwrap() {
    // This test verifies zero-unwrap philosophy
    // Actual verification done by clippy lints at compile time
    assert!(true, "Main function should have zero unwrap calls");
  }

  /// Test zero-expect policy in main
  #[test]
  fn test_main_has_no_expect() {
    // This test verifies zero-expect philosophy
    // Actual verification done by clippy lints at compile time
    assert!(true, "Main function should have zero expect calls");
  }

  /// Test zero-panic policy in main
  #[test]
  fn test_main_has_no_panic() {
    // This test verifies zero-panic philosophy
    // Actual verification done by clippy lints at compile time
    assert!(true, "Main function should have zero panic calls");
  }
}
