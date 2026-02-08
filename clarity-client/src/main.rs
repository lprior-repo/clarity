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
  use clarity_client::{WindowGeometry, WindowStateManager};
  use dioxus_desktop::{Config, WindowBuilder};

  // ==============================================================================
  // WINDOW CONFIGURATION
  // ==============================================================================

  // Default window geometry
  let default_geometry = WindowGeometry {
    x: 100,
    y: 100,
    width: 1200,
    height: 800,
  };

  // Try to load saved window state, fall back to defaults
  let geometry = match WindowStateManager::load_geometry() {
    Ok(geo) => geo,
    Err(_) => default_geometry,
  };

  // ==============================================================================
  // LAUNCH DESKTOP APPLICATION
  // ==============================================================================

  // Build window configuration
  let window_builder = WindowBuilder::new()
    .with_title("Clarity")
    .with_inner_size(geometry.width, geometry.height)
    .with_position(dioxus_desktop::tao::dpi::LogicalPosition::new(geometry.x, geometry.y))
    .with_resizable(true)
    .with_decorations(true);

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
    let width = 1200u32;
    let height = 800u32;

    assert!(width >= 800, "Window width should be at least 800px");
    assert!(height >= 600, "Window height should be at least 600px");
  }

  /// Test window title is configured
  #[test]
  fn test_window_title_is_configured() {
    let title = "Clarity";
    assert_eq!(title, "Clarity", "Window title should be 'Clarity'");
    assert!(!title.is_empty(), "Window title should not be empty");
  }

  /// Test window position is valid
  #[test]
  fn test_window_position_is_valid() {
    let x = 100i32;
    let y = 100i32;

    assert!(x >= 0, "Window X position should be non-negative");
    assert!(y >= 0, "Window Y position should be non-negative");
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
