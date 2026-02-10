#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Application settings and preferences management
//!
//! This module provides pure functions for loading, saving, and managing
//! application settings persisted to disk.

use anyhow::{Context, Result};
use clarity_core::db::models::{BeadPriority, BeadType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

// ===== Domain Types =====

/// Application theme preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
  Light,
  Dark,
  System,
}

impl Theme {
  /// Get the theme as a lowercase string
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Light => "light",
      Self::Dark => "dark",
      Self::System => "system",
    }
  }

  /// Parse a string into a Theme
  ///
  /// # Errors
  /// Returns an error if the string is not a valid theme
  pub fn from_str(s: &str) -> Result<Self> {
    match s.to_lowercase().as_str() {
      "light" => Ok(Self::Light),
      "dark" => Ok(Self::Dark),
      "system" => Ok(Self::System),
      _ => anyhow::bail!("Invalid theme: {s}"),
    }
  }
}

impl std::fmt::Display for Theme {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

/// Backup frequency settings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupFrequency {
  Hourly,
  Daily,
  Weekly,
  Never,
}

impl BackupFrequency {
  /// Get the duration for this frequency
  #[must_use]
  pub const fn as_duration(&self) -> Option<Duration> {
    match self {
      Self::Hourly => Some(Duration::from_secs(3600)),
      Self::Daily => Some(Duration::from_secs(86400)),
      Self::Weekly => Some(Duration::from_secs(604800)),
      Self::Never => None,
    }
  }

  /// Get the frequency as a lowercase string
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Hourly => "hourly",
      Self::Daily => "daily",
      Self::Weekly => "weekly",
      Self::Never => "never",
    }
  }

  /// Parse a string into a `BackupFrequency`
  ///
  /// # Errors
  /// Returns an error if the string is not a valid frequency
  pub fn from_str(s: &str) -> Result<Self> {
    match s.to_lowercase().as_str() {
      "hourly" => Ok(Self::Hourly),
      "daily" => Ok(Self::Daily),
      "weekly" => Ok(Self::Weekly),
      "never" => Ok(Self::Never),
      _ => anyhow::bail!("Invalid backup frequency: {s}"),
    }
  }
}

impl std::fmt::Display for BackupFrequency {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

// ===== Settings Model =====

/// Application settings
///
/// Immutable settings struct with pure transformation methods.
/// All settings updates create a new Settings instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
  pub theme: Theme,
  pub default_priority: BeadPriority,
  pub default_type: BeadType,
  pub beads_per_page: usize,
  pub auto_backup: bool,
  pub backup_frequency: BackupFrequency,
  pub keyboard_shortcuts: HashMap<String, String>,
  pub data_location: PathBuf,
}

impl Settings {
  /// Get the default settings
  #[must_use]
  pub fn defaults() -> Self {
    let data_location = get_default_data_dir();
    let keyboard_shortcuts = default_keyboard_shortcuts();

    Self {
      theme: Theme::System,
      default_priority: BeadPriority::MEDIUM,
      default_type: BeadType::Feature,
      beads_per_page: 20,
      auto_backup: true,
      backup_frequency: BackupFrequency::Daily,
      keyboard_shortcuts,
      data_location,
    }
  }

  /// Pure transformation: update theme
  #[must_use]
  pub fn with_theme(&self, theme: Theme) -> Self {
    let mut updated = self.clone();
    updated.theme = theme;
    updated
  }

  /// Pure transformation: update default priority
  #[must_use]
  pub fn with_default_priority(&self, priority: BeadPriority) -> Self {
    let mut updated = self.clone();
    updated.default_priority = priority;
    updated
  }

  /// Pure transformation: update default type
  #[must_use]
  pub fn with_default_type(&self, bead_type: BeadType) -> Self {
    let mut updated = self.clone();
    updated.default_type = bead_type;
    updated
  }

  /// Pure transformation: update beads per page
  #[must_use]
  pub fn with_beads_per_page(&self, count: usize) -> Self {
    let mut updated = self.clone();
    updated.beads_per_page = count;
    updated
  }

  /// Pure transformation: update auto backup
  #[must_use]
  pub fn with_auto_backup(&self, enabled: bool) -> Self {
    let mut updated = self.clone();
    updated.auto_backup = enabled;
    updated
  }

  /// Pure transformation: update backup frequency
  #[must_use]
  pub fn with_backup_frequency(&self, frequency: BackupFrequency) -> Self {
    let mut updated = self.clone();
    updated.backup_frequency = frequency;
    updated
  }

  /// Pure transformation: update keyboard shortcut
  #[must_use]
  pub fn with_keyboard_shortcut(&self, action: String, shortcut: String) -> Self {
    let mut updated = self.clone();
    updated.keyboard_shortcuts.insert(action, shortcut);
    updated
  }

  /// Pure transformation: update data location
  #[must_use]
  pub fn with_data_location(&self, location: PathBuf) -> Self {
    let mut updated = self.clone();
    updated.data_location = location;
    updated
  }

  /// Load settings from the config file
  ///
  /// # Errors
  /// Returns an error if the config file cannot be read or parsed
  pub fn load() -> Result<Self> {
    let config_path = get_config_path();

    if !config_path.exists() {
      // Return defaults if config doesn't exist
      return Ok(Self::defaults());
    }

    let content = std::fs::read_to_string(&config_path)
      .with_context(|| format!("Failed to read config from {}", config_path.display()))?;

    serde_json::from_str(&content)
      .with_context(|| format!("Failed to parse config from {}", config_path.display()))
  }

  /// Save settings to the config file
  ///
  /// # Errors
  /// Returns an error if the config file cannot be written
  pub fn save(&self) -> Result<()> {
    let config_path = get_config_path();

    // Ensure config directory exists
    if let Some(parent) = config_path.parent() {
      std::fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create config directory at {}", parent.display()))?;
    }

    let json = serde_json::to_string_pretty(self).context("Failed to serialize settings")?;

    std::fs::write(&config_path, json)
      .with_context(|| format!("Failed to write config to {}", config_path.display()))
  }

  /// Reset settings to defaults
  #[must_use]
  pub fn reset_to_defaults(&self) -> Self {
    Self::defaults()
  }
}

impl Default for Settings {
  fn default() -> Self {
    Self::defaults()
  }
}

// ===== Helper Functions =====

/// Get the path to the settings config file
#[must_use]
fn get_config_path() -> PathBuf {
  let config_dir = dirs::config_dir().unwrap_or_else(std::env::temp_dir);

  config_dir.join("clarity").join("settings.json")
}

/// Get the default data directory
#[must_use]
fn get_default_data_dir() -> PathBuf {
  dirs::data_local_dir()
    .unwrap_or_else(std::env::temp_dir)
    .join("clarity")
}

/// Get the default keyboard shortcuts
#[must_use]
fn default_keyboard_shortcuts() -> HashMap<String, String> {
  [
    ("new_bead".to_string(), "Ctrl+N".to_string()),
    ("save".to_string(), "Ctrl+S".to_string()),
    ("find".to_string(), "Ctrl+F".to_string()),
    ("settings".to_string(), "Ctrl+,".to_string()),
    ("toggle_theme".to_string(), "Ctrl+Shift+T".to_string()),
  ]
  .into()
}

// ===== Validation =====

/// Validate `beads_per_page` value
#[must_use]
pub fn validate_beads_per_page(value: usize) -> bool {
  (5..=100).contains(&value)
}

/// Validate data location path
///
/// # Errors
/// Returns an error if the path is invalid or not accessible
pub fn validate_data_location(path: &PathBuf) -> Result<()> {
  if path.as_os_str().is_empty() {
    anyhow::bail!("Data location cannot be empty");
  }

  // Check if parent directory exists or can be created
  if let Some(parent) = path.parent() {
    if !parent.as_os_str().is_empty() && !parent.exists() {
      std::fs::create_dir_all(parent)
        .with_context(|| format!("Cannot create data directory at {}", parent.display()))?;
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::expect_used)]
  use super::*;

  #[test]
  fn test_settings_default() {
    let settings = Settings::default();
    assert_eq!(settings.theme, Theme::System);
    assert_eq!(settings.default_priority, BeadPriority::MEDIUM);
    assert_eq!(settings.default_type, BeadType::Feature);
    assert_eq!(settings.beads_per_page, 20);
    assert!(settings.auto_backup);
    assert_eq!(settings.backup_frequency, BackupFrequency::Daily);
  }

  #[test]
  fn test_settings_with_theme() {
    let settings = Settings::default();
    let dark = settings.with_theme(Theme::Dark);
    assert_eq!(dark.theme, Theme::Dark);
  }

  #[test]
  fn test_settings_with_priority() {
    let settings = Settings::default();
    let high = settings.with_default_priority(BeadPriority::HIGH);
    assert_eq!(high.default_priority, BeadPriority::HIGH);
  }

  #[test]
  fn test_theme_from_str() {
    assert_eq!(Theme::from_str("light").unwrap(), Theme::Light);
    assert_eq!(Theme::from_str("dark").unwrap(), Theme::Dark);
    assert_eq!(Theme::from_str("system").unwrap(), Theme::System);
    assert!(Theme::from_str("invalid").is_err());
  }

  #[test]
  fn test_backup_frequency_duration() {
    assert_eq!(
      BackupFrequency::Hourly.as_duration(),
      Some(Duration::from_secs(3600))
    );
    assert_eq!(
      BackupFrequency::Daily.as_duration(),
      Some(Duration::from_secs(86400))
    );
    assert_eq!(
      BackupFrequency::Weekly.as_duration(),
      Some(Duration::from_secs(604800))
    );
    assert_eq!(BackupFrequency::Never.as_duration(), None);
  }

  #[test]
  fn test_validate_beads_per_page() {
    assert!(!validate_beads_per_page(0));
    assert!(!validate_beads_per_page(4));
    assert!(validate_beads_per_page(5));
    assert!(validate_beads_per_page(20));
    assert!(validate_beads_per_page(100));
    assert!(!validate_beads_per_page(101));
  }

  #[test]
  fn test_settings_serialization() {
    let settings = Settings::default();
    let json = serde_json::to_string(&settings).unwrap();
    let deserialized: Settings = serde_json::from_str(&json).unwrap();
    assert_eq!(settings, deserialized);
  }

  #[test]
  fn test_keyboard_shortcuts_default() {
    let shortcuts = default_keyboard_shortcuts();
    assert_eq!(shortcuts.get("new_bead"), Some(&"Ctrl+N".to_string()));
    assert_eq!(shortcuts.get("save"), Some(&"Ctrl+S".to_string()));
    assert_eq!(shortcuts.get("find"), Some(&"Ctrl+F".to_string()));
  }
}
