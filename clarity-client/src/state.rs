#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Global application state management for Clarity desktop app
//!
//! This module provides reactive global state using Dioxus signals and context.
//! All state is immutable and accessed through type-safe getters.

use dioxus::prelude::*;
use rpds::Vector;
use std::rc::Rc;

// ===== Domain State Types =====

/// Immutable authentication state snapshot
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthState {
  /// Whether a user is currently authenticated
  pub is_authenticated: bool,
  /// Current user ID (if authenticated)
  pub current_user: Option<String>,
  /// Session token (if authenticated)
  pub session_token: Option<String>,
}

impl AuthState {
  /// Create a new unauthenticated state
  #[must_use]
  pub const fn unauthenticated() -> Self {
    Self {
      is_authenticated: false,
      current_user: None,
      session_token: None,
    }
  }

  /// Create an authenticated state
  #[must_use]
  pub const fn authenticated(user_id: String, token: String) -> Self {
    Self {
      is_authenticated: true,
      current_user: Some(user_id),
      session_token: Some(token),
    }
  }

  /// Clear authentication (logout)
  #[must_use]
  pub const fn clear(&self) -> Self {
    Self::unauthenticated()
  }
}

impl Default for AuthState {
  fn default() -> Self {
    Self::unauthenticated()
  }
}

/// Immutable bead list state snapshot
#[derive(Clone, Debug)]
pub struct BeadState {
  /// List of all beads (persistent vector for structural sharing)
  pub beads: Vector<Rc<clarity_core::db::models::Bead>>,
  /// Whether beads are currently being loaded
  pub loading: bool,
  /// Error message if loading failed
  pub error: Option<String>,
}

impl PartialEq for BeadState {
  fn eq(&self, other: &Self) -> bool {
    self.loading == other.loading &&
    self.error == other.error &&
    self.beads.len() == other.beads.len() &&
    self.beads.iter().zip(other.beads.iter()).all(|(a, b)| Rc::ptr_eq(a, b))
  }
}

impl BeadState {
  /// Create a new empty bead state
  #[must_use]
  pub fn empty() -> Self {
    Self {
      beads: Vector::new(),
      loading: false,
      error: None,
    }
  }

  /// Create a loading state
  #[must_use]
  pub fn loading() -> Self {
    Self {
      beads: Vector::new(),
      loading: true,
      error: None,
    }
  }

  /// Create an error state
  #[must_use]
  pub fn with_error(error: String) -> Self {
    Self {
      beads: Vector::new(),
      loading: false,
      error: Some(error),
    }
  }

  /// Update the beads list (pure transformation)
  #[must_use]
  pub fn with_beads(&self, beads: Vec<clarity_core::db::models::Bead>) -> Self {
    Self {
      beads: beads.into_iter().map(Rc::new).collect(),
      loading: false,
      error: None,
    }
  }

  /// Add a single bead to the state
  #[must_use]
  pub fn with_bead(&self, bead: clarity_core::db::models::Bead) -> Self {
    Self {
      beads: self.beads.push_back(Rc::new(bead)),
      loading: false,
      error: None,
    }
  }

  /// Clear the error state
  #[must_use]
  pub fn clear_error(&self) -> Self {
    Self {
      beads: self.beads.clone(),
      loading: self.loading,
      error: None,
    }
  }
}

impl Default for BeadState {
  fn default() -> Self {
    Self::empty()
  }
}

/// Immutable UI state snapshot
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UIState {
  /// Current route path
  pub current_route: String,
  /// Active theme (light/dark)
  pub theme: Theme,
}

/// Application theme
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
  Light,
  Dark,
}

impl UIState {
  /// Create a new UI state with defaults
  #[must_use]
  pub fn new() -> Self {
    Self {
      current_route: "/".to_string(),
      theme: Theme::Light,
    }
  }

  /// Update the current route
  #[must_use]
  pub const fn with_route(&self, route: String) -> Self {
    Self {
      current_route: route,
      theme: self.theme,
    }
  }

  /// Toggle the theme
  #[must_use]
  pub fn toggle_theme(&self) -> Self {
    Self {
      current_route: self.current_route.clone(),
      theme: match self.theme {
        Theme::Light => Theme::Dark,
        Theme::Dark => Theme::Light,
      },
    }
  }

  /// Set the theme
  #[must_use]
  pub fn with_theme(&self, theme: Theme) -> Self {
    Self {
      current_route: self.current_route.clone(),
      theme,
    }
  }
}

impl Default for UIState {
  fn default() -> Self {
    Self::new()
  }
}

// ===== Global App State =====

/// Global application state
///
/// This struct holds all reactive signals for the application.
/// It provides immutable getters and setters for state updates.
#[derive(Clone)]
pub struct AppState {
  /// Authentication state signal
  pub auth: Signal<AuthState>,
  /// Bead list state signal
  pub beads: Signal<BeadState>,
  /// UI state signal
  pub ui: Signal<UIState>,
}

impl AppState {
  /// Create a new app state with default values
  #[must_use]
  pub fn new() -> Self {
    Self {
      auth: Signal::new(AuthState::default()),
      beads: Signal::new(BeadState::default()),
      ui: Signal::new(UIState::default()),
    }
  }

  // ===== Auth State Accessors =====

  /// Get the current auth state (immutable snapshot)
  #[must_use]
  pub fn auth(&self) -> AuthState {
    self.auth.read().clone()
  }

  /// Check if user is authenticated
  #[must_use]
  pub fn is_authenticated(&self) -> bool {
    self.auth.read().is_authenticated
  }

  /// Get the current user ID
  #[must_use]
  pub fn current_user(&self) -> Option<String> {
    self.auth.read().current_user.clone()
  }

  /// Set authentication state (login)
  pub fn set_auth(&self, user_id: String, token: String) {
    let new_auth = AuthState::authenticated(user_id, token);
    let mut auth = self.auth;
    auth.set(new_auth);
  }

  /// Clear authentication state (logout)
  pub fn clear_auth(&mut self) {
    let new_auth = AuthState::unauthenticated();
    self.auth.set(new_auth);
  }

  // ===== Bead State Accessors =====

  /// Get the current bead state (immutable snapshot)
  #[must_use]
  pub fn beads(&self) -> BeadState {
    self.beads.read().clone()
  }

  /// Get the list of beads
  #[must_use]
  pub fn bead_list(&self) -> Vector<Rc<clarity_core::db::models::Bead>> {
    self.beads.read().beads.clone()
  }

  /// Check if beads are loading
  #[must_use]
  pub fn beads_loading(&self) -> bool {
    self.beads.read().loading
  }

  /// Get the current error state
  #[must_use]
  pub fn beads_error(&self) -> Option<String> {
    self.beads.read().error.clone()
  }

  /// Set beads to loading state
  pub fn set_beads_loading(&self) {
    let new_state = BeadState::loading();
    let mut beads = self.beads;
    beads.set(new_state);
  }

  /// Update the beads list
  pub fn set_beads(&self, beads: Vec<clarity_core::db::models::Bead>) {
    let current = self.beads.read().clone();
    let new_state = current.with_beads(beads);
    let mut beads_signal = self.beads;
    beads_signal.set(new_state);
  }

  /// Set beads error state
  pub fn set_beads_error(&self, error: String) {
    let new_state = BeadState::with_error(error);
    let mut beads = self.beads;
    beads.set(new_state);
  }

  /// Clear beads error state
  pub fn clear_beads_error(&self) {
    let current = self.beads.read().clone();
    let new_state = current.clear_error();
    let mut beads = self.beads;
    beads.set(new_state);
  }

  /// Add a single bead to the state
  pub fn add_bead(&self, bead: clarity_core::db::models::Bead) {
    let current = self.beads.read().clone();
    let new_state = current.with_bead(bead);
    let mut beads = self.beads;
    beads.set(new_state);
  }

  // ===== UI State Accessors =====

  /// Get the current UI state (immutable snapshot)
  #[must_use]
  pub fn ui(&self) -> UIState {
    self.ui.read().clone()
  }

  /// Get the current route
  #[must_use]
  pub fn current_route(&self) -> String {
    self.ui.read().current_route.clone()
  }

  /// Get the current theme
  #[must_use]
  pub fn theme(&self) -> Theme {
    self.ui.read().theme
  }

  /// Update the current route
  pub fn set_route(&self, route: String) {
    let current = self.ui.read().clone();
    let new_ui = current.with_route(route);
    let mut ui = self.ui;
    ui.set(new_ui);
  }

  /// Toggle the theme
  pub fn toggle_theme(&self) {
    let current = self.ui.read().clone();
    let new_ui = current.toggle_theme();
    let mut ui = self.ui;
    ui.set(new_ui);
  }

  /// Set the theme
  pub fn set_theme(&self, theme: Theme) {
    let current = self.ui.read().clone();
    let new_ui = current.with_theme(theme);
    let mut ui = self.ui;
    ui.set(new_ui);
  }
}

impl Default for AppState {
  fn default() -> Self {
    Self::new()
  }
}

// ===== State Persistence =====

/// State persistence manager for saving/loading state to disk
pub struct StatePersistence {
  db_path: std::path::PathBuf,
}

impl StatePersistence {
  /// Create a new state persistence manager
  ///
  /// # Errors
  /// Returns error if data directory cannot be determined
  pub fn new() -> Result<Self, anyhow::Error> {
    let data_dir = dirs::data_local_dir()
      .ok_or_else(|| anyhow::anyhow!("Failed to determine local data directory"))?;

    let app_dir = data_dir.join("clarity");
    std::fs::create_dir_all(&app_dir)?;

    Ok(Self {
      db_path: app_dir.join("state.json"),
    })
  }

  /// Save auth token to disk
  ///
  /// # Errors
  /// Returns error if file write fails
  pub fn save_auth_token(&self, token: &str) -> anyhow::Result<()> {
    let auth_data = serde_json::json!({
        "session_token": token,
        "saved_at": chrono::Utc::now().to_rfc3339(),
    });

    let json = serde_json::to_string_pretty(&auth_data)?;
    std::fs::write(&self.db_path, json)?;

    Ok(())
  }

  /// Load auth token from disk
  ///
  /// # Errors
  /// Returns error if file read or parse fails
  pub fn load_auth_token(&self) -> anyhow::Result<Option<String>> {
    if !self.db_path.exists() {
      return Ok(None);
    }

    let content = std::fs::read_to_string(&self.db_path)?;
    let data: serde_json::Value = serde_json::from_str(&content)?;

    Ok(
      data
        .get("session_token")
        .and_then(|v| v.as_str())
        .map(String::from),
    )
  }

  /// Clear saved auth token
  ///
  /// # Errors
  /// Returns error if file deletion fails
  pub fn clear_auth_token(&self) -> anyhow::Result<()> {
    if self.db_path.exists() {
      std::fs::remove_file(&self.db_path)?;
    }
    Ok(())
  }
}

impl Default for StatePersistence {
  fn default() -> Self {
    // Fallback to a simpler path that should always work
    let temp_dir = std::env::temp_dir();
    let app_dir = temp_dir.join("clarity");
    std::fs::create_dir_all(&app_dir).ok();
    Self {
      db_path: app_dir.join("state.json"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_auth_state_default() {
    let state = AuthState::default();
    assert!(!state.is_authenticated);
    assert!(state.current_user.is_none());
    assert!(state.session_token.is_none());
  }

  #[test]
  fn test_auth_state_authenticated() {
    let state = AuthState::authenticated("user123".to_string(), "token456".to_string());
    assert!(state.is_authenticated);
    assert_eq!(state.current_user, Some("user123".to_string()));
    assert_eq!(state.session_token, Some("token456".to_string()));
  }

  #[test]
  fn test_auth_state_clear() {
    let state = AuthState::authenticated("user123".to_string(), "token456".to_string());
    let cleared = state.clear();
    assert!(!cleared.is_authenticated);
  }

  #[test]
  fn test_bead_state_empty() {
    let state = BeadState::default();
    assert!(state.beads.is_empty());
    assert!(!state.loading);
    assert!(state.error.is_none());
  }

  #[test]
  fn test_bead_state_loading() {
    let state = BeadState::loading();
    assert!(state.loading);
    assert!(state.beads.is_empty());
  }

  #[test]
  fn test_bead_state_with_error() {
    let state = BeadState::with_error("Database error".to_string());
    assert!(!state.loading);
    assert_eq!(state.error, Some("Database error".to_string()));
  }

  #[test]
  fn test_ui_state_default() {
    let state = UIState::default();
    assert_eq!(state.current_route, "/");
    assert_eq!(state.theme, Theme::Light);
  }

  #[test]
  fn test_ui_state_with_route() {
    let state = UIState::new().with_route("/beads".to_string());
    assert_eq!(state.current_route, "/beads");
  }

  #[test]
  fn test_ui_state_toggle_theme() {
    let state = UIState::new();
    assert_eq!(state.theme, Theme::Light);

    let dark = state.toggle_theme();
    assert_eq!(dark.theme, Theme::Dark);

    let light = dark.toggle_theme();
    assert_eq!(light.theme, Theme::Light);
  }
}
