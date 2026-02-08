//! Desktop window state management
//!
//! This module provides window state persistence and management for Dioxus Desktop applications.
//! It handles saving and restoring window position, size, and maximization state across sessions.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::result::Result;

/// Window state errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowStateError {
  /// Failed to read state file
  ReadError(String),
  /// Failed to write state file
  WriteError(String),
  /// Failed to parse state data
  ParseError(String),
  /// Invalid state data
  InvalidState(String),
  /// File system error
  FsError(String),
}

impl std::fmt::Display for WindowStateError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::ReadError(msg) => write!(f, "Failed to read window state: {msg}"),
      Self::WriteError(msg) => write!(f, "Failed to write window state: {msg}"),
      Self::ParseError(msg) => write!(f, "Failed to parse window state: {msg}"),
      Self::InvalidState(msg) => write!(f, "Invalid window state: {msg}"),
      Self::FsError(msg) => write!(f, "File system error: {msg}"),
    }
  }
}

impl std::error::Error for WindowStateError {}

/// Window position and size
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowGeometry {
  /// X coordinate
  pub x: i32,
  /// Y coordinate
  pub y: i32,
  /// Window width
  pub width: u32,
  /// Window height
  pub height: u32,
}

impl WindowGeometry {
  /// Create a new window geometry
  #[must_use]
  pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
    Self {
      x,
      y,
      width,
      height,
    }
  }

  /// Validate geometry constraints
  ///
  /// # Errors
  /// Returns `WindowStateError::InvalidState` if geometry is invalid
  pub fn validate(&self) -> Result<(), WindowStateError> {
    if self.width < 100 {
      return Err(WindowStateError::InvalidState(format!(
        "Window width too small: {} (minimum: 100)",
        self.width
      )));
    }

    if self.height < 100 {
      return Err(WindowStateError::InvalidState(format!(
        "Window height too small: {} (minimum: 100)",
        self.height
      )));
    }

    if self.width > 10_000 {
      return Err(WindowStateError::InvalidState(format!(
        "Window width too large: {} (maximum: 10000)",
        self.width
      )));
    }

    if self.height > 10_000 {
      return Err(WindowStateError::InvalidState(format!(
        "Window height too large: {} (maximum: 10000)",
        self.height
      )));
    }

    Ok(())
  }
}

impl Default for WindowGeometry {
  fn default() -> Self {
    Self::new(100, 100, 1280, 720)
  }
}

/// Window state persistence
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowState {
  /// Window geometry
  pub geometry: WindowGeometry,
  /// Whether window is maximized
  pub maximized: bool,
  /// Whether window is fullscreen
  pub fullscreen: bool,
  /// Monitor index (for multi-monitor setups)
  pub monitor: Option<u32>,
}

impl WindowState {
  /// Create a new window state with default values
  #[must_use]
  pub const fn new() -> Self {
    Self {
      geometry: WindowGeometry::new(100, 100, 1280, 720),
      maximized: false,
      fullscreen: false,
      monitor: None,
    }
  }

  /// Create a new window state with custom geometry
  ///
  /// # Errors
  /// Returns `WindowStateError::InvalidState` if geometry is invalid
  pub fn with_geometry(geometry: WindowGeometry) -> Result<Self, WindowStateError> {
    geometry.validate()?;
    Ok(Self {
      geometry,
      maximized: false,
      fullscreen: false,
      monitor: None,
    })
  }

  /// Set maximized state
  #[must_use]
  pub const fn with_maximized(mut self, maximized: bool) -> Self {
    self.maximized = maximized;
    self
  }

  /// Set fullscreen state
  #[must_use]
  pub const fn with_fullscreen(mut self, fullscreen: bool) -> Self {
    self.fullscreen = fullscreen;
    self
  }

  /// Set monitor index
  #[must_use]
  pub const fn with_monitor(mut self, monitor: u32) -> Self {
    self.monitor = Some(monitor);
    self
  }

  /// Validate window state
  ///
  /// # Errors
  /// Returns `WindowStateError::InvalidState` if state is invalid
  pub fn validate(&self) -> Result<(), WindowStateError> {
    self.geometry.validate()?;

    if let Some(monitor) = self.monitor {
      if monitor > 10 {
        return Err(WindowStateError::InvalidState(format!(
          "Monitor index too large: {} (maximum: 10)",
          monitor
        )));
      }
    }

    Ok(())
  }
}

impl Default for WindowState {
  fn default() -> Self {
    Self::new()
  }
}

/// Window state manager
pub struct WindowStateManager {
  /// State file path
  state_path: PathBuf,
}

impl WindowStateManager {
  /// Create a new window state manager
  ///
  /// # Errors
  /// Returns `WindowStateError::FsError` if config directory cannot be determined
  pub fn new(app_name: &str) -> Result<Self, WindowStateError> {
    let state_path = Self::get_state_path(app_name)?;
    Ok(Self { state_path })
  }

  /// Get the state file path for the application
  ///
  /// # Errors
  /// Returns `WindowStateError::FsError` if config directory cannot be determined
  fn get_state_path(app_name: &str) -> Result<PathBuf, WindowStateError> {
    let config_dir = dirs::config_dir()
      .ok_or_else(|| WindowStateError::FsError("Cannot determine config directory".to_string()))
      .map(|d| d.join(app_name))?;

    // Create config directory if it doesn't exist
    std::fs::create_dir_all(&config_dir).map_err(|e| {
      WindowStateError::FsError(format!("Failed to create config directory: {}", e))
    })?;

    Ok(config_dir.join("window_state.json"))
  }

  /// Load window state from disk
  ///
  /// # Errors
  /// Returns `WindowStateError::ReadError` if state file cannot be read
  /// Returns `WindowStateError::ParseError` if state file cannot be parsed
  pub fn load(&self) -> Result<WindowState, WindowStateError> {
    // If state file doesn't exist, return default state
    if !self.state_path.exists() {
      return Ok(WindowState::default());
    }

    // Read state file
    let content = std::fs::read_to_string(&self.state_path).map_err(|e| {
      WindowStateError::ReadError(format!(
        "Failed to read state file from {:?}: {}",
        self.state_path, e
      ))
    })?;

    // Parse state
    let state: WindowState = serde_json::from_str(&content)
      .map_err(|e| WindowStateError::ParseError(format!("Failed to parse window state: {}", e)))?;

    // Validate state
    state.validate()?;

    Ok(state)
  }

  /// Save window state to disk
  ///
  /// # Errors
  /// Returns `WindowStateError::WriteError` if state file cannot be written
  pub fn save(&self, state: &WindowState) -> Result<(), WindowStateError> {
    // Validate state before saving
    state.validate()?;

    // Serialize state
    let content = serde_json::to_string_pretty(state).map_err(|e| {
      WindowStateError::WriteError(format!("Failed to serialize window state: {}", e))
    })?;

    // Write to temporary file first (atomic write)
    let temp_path = self.state_path.with_extension("json.tmp");

    std::fs::write(&temp_path, &content).map_err(|e| {
      WindowStateError::WriteError(format!(
        "Failed to write state file to {:?}: {}",
        temp_path, e
      ))
    })?;

    // Atomic rename
    std::fs::rename(&temp_path, &self.state_path).map_err(|e| {
      WindowStateError::WriteError(format!(
        "Failed to rename state file from {:?} to {:?}: {}",
        temp_path, self.state_path, e
      ))
    })?;

    Ok(())
  }

  /// Clear window state (reset to defaults)
  ///
  /// # Errors
  /// Returns `WindowStateError::FsError` if state file cannot be removed
  pub fn clear(&self) -> Result<(), WindowStateError> {
    if self.state_path.exists() {
      std::fs::remove_file(&self.state_path).map_err(|e| {
        WindowStateError::FsError(format!(
          "Failed to remove state file {:?}: {}",
          self.state_path, e
        ))
      })?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // Martin Fowler Test Suite: Window State Management

  #[test]
  fn test_window_geometry_new() {
    // GIVEN: valid coordinates and dimensions
    let x = 100;
    let y = 200;
    let width = 1280;
    let height = 720;

    // WHEN: creating a new window geometry
    let geometry = WindowGeometry::new(x, y, width, height);

    // THEN: geometry should be created with correct values
    assert_eq!(geometry.x, x);
    assert_eq!(geometry.y, y);
    assert_eq!(geometry.width, width);
    assert_eq!(geometry.height, height);
  }

  #[test]
  fn test_window_geometry_default() {
    // GIVEN: no parameters
    // WHEN: creating default geometry
    let geometry = WindowGeometry::default();

    // THEN: should have default HD resolution
    assert_eq!(geometry.x, 100);
    assert_eq!(geometry.y, 100);
    assert_eq!(geometry.width, 1280);
    assert_eq!(geometry.height, 720);
  }

  #[test]
  fn test_window_geometry_validate_valid() {
    // GIVEN: valid geometry
    let geometry = WindowGeometry::new(100, 100, 1280, 720);

    // WHEN: validating geometry
    let result = geometry.validate();

    // THEN: validation should succeed
    assert!(result.is_ok());
  }

  #[test]
  fn test_window_geometry_validate_too_small() {
    // GIVEN: geometry with width less than minimum
    let geometry = WindowGeometry::new(100, 100, 50, 720);

    // WHEN: validating geometry
    let result = geometry.validate();

    // THEN: validation should fail
    assert!(result.is_err());
    assert!(matches!(result, Err(WindowStateError::InvalidState(_))));
  }

  #[test]
  fn test_window_geometry_validate_height_too_small() {
    // GIVEN: geometry with height less than minimum
    let geometry = WindowGeometry::new(100, 100, 1280, 50);

    // WHEN: validating geometry
    let result = geometry.validate();

    // THEN: validation should fail
    assert!(result.is_err());
    assert!(matches!(result, Err(WindowStateError::InvalidState(_))));
  }

  #[test]
  fn test_window_geometry_validate_too_large() {
    // GIVEN: geometry with width exceeding maximum
    let geometry = WindowGeometry::new(100, 100, 20000, 720);

    // WHEN: validating geometry
    let result = geometry.validate();

    // THEN: validation should fail
    assert!(result.is_err());
    assert!(matches!(result, Err(WindowStateError::InvalidState(_))));
  }

  #[test]
  fn test_window_state_new() {
    // GIVEN: no parameters
    // WHEN: creating new window state
    let state = WindowState::new();

    // THEN: should have default values
    assert_eq!(state.geometry, WindowGeometry::default());
    assert_eq!(state.maximized, false);
    assert_eq!(state.fullscreen, false);
    assert_eq!(state.monitor, None);
  }

  #[test]
  fn test_window_state_default() {
    // GIVEN: no parameters
    // WHEN: creating default window state
    let state = WindowState::default();

    // THEN: should have default values
    assert_eq!(state.geometry, WindowGeometry::default());
    assert_eq!(state.maximized, false);
    assert_eq!(state.fullscreen, false);
    assert_eq!(state.monitor, None);
  }

  #[test]
  fn test_window_state_with_geometry_valid() {
    // GIVEN: valid geometry
    let geometry = WindowGeometry::new(200, 200, 1920, 1080);

    // WHEN: creating state with geometry
    let result = WindowState::with_geometry(geometry.clone());

    // THEN: state should be created successfully
    assert!(result.is_ok());
    let state = result.unwrap();
    assert_eq!(state.geometry, geometry);
    assert_eq!(state.maximized, false);
    assert_eq!(state.fullscreen, false);
  }

  #[test]
  fn test_window_state_with_geometry_invalid() {
    // GIVEN: invalid geometry
    let geometry = WindowGeometry::new(100, 100, 50, 720);

    // WHEN: creating state with invalid geometry
    let result = WindowState::with_geometry(geometry);

    // THEN: state creation should fail
    assert!(result.is_err());
    assert!(matches!(result, Err(WindowStateError::InvalidState(_))));
  }

  #[test]
  fn test_window_state_with_maximized() {
    // GIVEN: default state
    let state = WindowState::new();

    // WHEN: setting maximized to true
    let state = state.with_maximized(true);

    // THEN: maximized should be true
    assert!(state.maximized);
  }

  #[test]
  fn test_window_state_with_fullscreen() {
    // GIVEN: default state
    let state = WindowState::new();

    // WHEN: setting fullscreen to true
    let state = state.with_fullscreen(true);

    // THEN: fullscreen should be true
    assert!(state.fullscreen);
  }

  #[test]
  fn test_window_state_with_monitor() {
    // GIVEN: default state
    let state = WindowState::new();

    // WHEN: setting monitor to 1
    let state = state.with_monitor(1);

    // THEN: monitor should be Some(1)
    assert_eq!(state.monitor, Some(1));
  }

  #[test]
  fn test_window_state_validate_valid() {
    // GIVEN: valid state
    let state = WindowState::new().with_monitor(0);

    // WHEN: validating state
    let result = state.validate();

    // THEN: validation should succeed
    assert!(result.is_ok());
  }

  #[test]
  fn test_window_state_validate_monitor_too_large() {
    // GIVEN: state with monitor index exceeding maximum
    let state = WindowState::new().with_monitor(100);

    // WHEN: validating state
    let result = state.validate();

    // THEN: validation should fail
    assert!(result.is_err());
    assert!(matches!(result, Err(WindowStateError::InvalidState(_))));
  }

  #[test]
  fn test_window_state_error_display() {
    // GIVEN: various window state errors
    let err1 = WindowStateError::ReadError("Read failed".to_string());
    let err2 = WindowStateError::WriteError("Write failed".to_string());
    let err3 = WindowStateError::ParseError("Parse failed".to_string());
    let err4 = WindowStateError::InvalidState("Invalid".to_string());
    let err5 = WindowStateError::FsError("FS error".to_string());

    // WHEN: converting errors to string
    let msg1 = err1.to_string();
    let msg2 = err2.to_string();
    let msg3 = err3.to_string();
    let msg4 = err4.to_string();
    let msg5 = err5.to_string();

    // THEN: error messages should be descriptive
    assert!(msg1.contains("Failed to read window state"));
    assert!(msg2.contains("Failed to write window state"));
    assert!(msg3.contains("Failed to parse window state"));
    assert!(msg4.contains("Invalid window state"));
    assert!(msg5.contains("File system error"));
  }

  #[test]
  fn test_window_state_serialization() {
    // GIVEN: a window state
    let state = WindowState::new().with_maximized(true).with_monitor(1);

    // WHEN: serializing to JSON
    let json = serde_json::to_string(&state);

    // THEN: serialization should succeed
    assert!(json.is_ok());

    // WHEN: deserializing back
    let json = json.unwrap();
    let deserialized: Result<WindowState, _> = serde_json::from_str(&json);

    // THEN: deserialization should succeed and match original
    assert!(deserialized.is_ok());
    let deserialized = deserialized.unwrap();
    assert_eq!(deserialized.geometry, state.geometry);
    assert_eq!(deserialized.maximized, state.maximized);
    assert_eq!(deserialized.fullscreen, state.fullscreen);
    assert_eq!(deserialized.monitor, state.monitor);
  }
}
