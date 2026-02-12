#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(warnings)]
#![allow(clippy::all)]

//! Hook for managing application settings
//!
//! Provides reactive state management for settings with persistence.

use crate::settings::{
  validate_beads_per_page, validate_data_location, BackupFrequency, Settings, Theme,
};
use clarity_core::db::models::{BeadPriority, BeadType};
use dioxus::prelude::*;
use std::path::PathBuf;

/// Settings state and actions
///
/// Immutable snapshot of settings state with update actions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsState {
  pub settings: Settings,
  pub loading: bool,
  pub error: Option<String>,
}

impl SettingsState {
  /// Create a new loaded state
  #[must_use]
  pub const fn loaded(settings: Settings) -> Self {
    Self {
      settings,
      loading: false,
      error: None,
    }
  }

  /// Create a loading state
  #[must_use]
  pub fn loading() -> Self {
    Self {
      settings: Settings::default(),
      loading: true,
      error: None,
    }
  }

  /// Create an error state
  #[must_use]
  pub fn with_error(error: String) -> Self {
    Self {
      settings: Settings::default(),
      loading: false,
      error: Some(error),
    }
  }

  /// Pure transformation: update settings
  #[must_use]
  pub const fn with_settings(&self, settings: Settings) -> Self {
    Self {
      settings,
      loading: false,
      error: None,
    }
  }
}

/// Settings actions for updating state
pub struct SettingsActions {
  pub update_theme: Callback<Theme>,
  pub update_priority: Callback<BeadPriority>,
  pub update_type: Callback<BeadType>,
  pub update_beads_per_page: Callback<usize>,
  pub update_auto_backup: Callback<bool>,
  pub update_backup_frequency: Callback<BackupFrequency>,
  pub update_keyboard_shortcut: Callback<(String, String)>,
  pub update_data_location: Callback<PathBuf>,
  pub save_settings: Callback<()>,
  pub reset_settings: Callback<()>,
}

/// Hook for managing application settings
///
/// Provides reactive state and actions for managing user preferences.
///
/// # Returns
/// A tuple of (`settings_state`, actions, `save_result`)
/// - `settings_state`: Current settings state with loading and error info
/// - actions: Callbacks for updating settings
/// - `save_result`: Signal tracking the last save operation result
#[must_use]
pub fn use_settings() -> (Signal<SettingsState>, SettingsActions, Signal<Option<bool>>) {
  let settings_state = use_signal(|| SettingsState::loaded(Settings::default()));
  let save_result = use_signal(|| None::<bool>);

  // Load settings on mount
  {
    let settings_state = settings_state;
    use_effect(move || {
      let mut settings_state_clone = settings_state;
      // Load settings
      match Settings::load() {
        Ok(settings) => {
          *settings_state_clone.write() = SettingsState::loaded(settings);
        }
        Err(e) => {
          *settings_state_clone.write() =
            SettingsState::with_error(format!("Failed to load settings: {e}"));
        }
      }
    });
  }

  // Update theme action
  let update_theme = {
    let mut settings_state = settings_state;
    Callback::new(move |theme: Theme| {
      let current = settings_state.read().clone();
      let updated = current.settings.with_theme(theme);
      let new_state = current.with_settings(updated);
      *settings_state.write() = new_state;
    })
  };

  // Update priority action
  let update_priority = {
    let mut settings_state = settings_state;
    Callback::new(move |priority: BeadPriority| {
      let current = settings_state.read().clone();
      let updated = current.settings.with_default_priority(priority);
      let new_state = current.with_settings(updated);
      *settings_state.write() = new_state;
    })
  };

  // Update type action
  let update_type = {
    let mut settings_state = settings_state;
    Callback::new(move |bead_type: BeadType| {
      let current = settings_state.read().clone();
      let updated = current.settings.with_default_type(bead_type);
      let new_state = current.with_settings(updated);
      *settings_state.write() = new_state;
    })
  };

  // Update beads per page action
  let update_beads_per_page = {
    let mut settings_state = settings_state;
    Callback::new(move |count: usize| {
      let current = settings_state.read().clone();
      let updated = current.settings.with_beads_per_page(count);
      let new_state = current.with_settings(updated);
      *settings_state.write() = new_state;
    })
  };

  // Update auto backup action
  let update_auto_backup = {
    let mut settings_state = settings_state;
    Callback::new(move |enabled: bool| {
      let current = settings_state.read().clone();
      let updated = current.settings.with_auto_backup(enabled);
      let new_state = current.with_settings(updated);
      *settings_state.write() = new_state;
    })
  };

  // Update backup frequency action
  let update_backup_frequency = {
    let mut settings_state = settings_state;
    Callback::new(move |frequency: BackupFrequency| {
      let current = settings_state.read().clone();
      let updated = current.settings.with_backup_frequency(frequency);
      let new_state = current.with_settings(updated);
      *settings_state.write() = new_state;
    })
  };

  // Update keyboard shortcut action
  let update_keyboard_shortcut = {
    let mut settings_state = settings_state;
    Callback::new(move |(action, shortcut): (String, String)| {
      let current = settings_state.read().clone();
      let updated = current.settings.with_keyboard_shortcut(action, shortcut);
      let new_state = current.with_settings(updated);
      *settings_state.write() = new_state;
    })
  };

  // Update data location action
  let update_data_location = {
    let mut settings_state = settings_state;
    Callback::new(move |location: PathBuf| {
      let current = settings_state.read().clone();
      let updated = current.settings.with_data_location(location);
      let new_state = current.with_settings(updated);
      *settings_state.write() = new_state;
    })
  };

  // Save settings action
  let save_settings = {
    let settings_state = settings_state;
    let save_result = save_result;
    Callback::new(move |()| {
      let settings_state_clone = settings_state;
      let mut save_result_clone = save_result;
      async move {
        let settings = settings_state_clone.read().settings.clone();
        match settings.save() {
          Ok(()) => {
            *save_result_clone.write() = Some(true);
          }
          Err(e) => {
            eprintln!("Failed to save settings: {e}");
            *save_result_clone.write() = Some(false);
          }
        }
      }
    })
  };

  // Reset settings action
  let reset_settings = {
    let mut settings_state = settings_state;
    Callback::new(move |()| {
      let current = settings_state.read().clone();
      let reset = current.settings.reset_to_defaults();
      let new_state = current.with_settings(reset);
      *settings_state.write() = new_state;
    })
  };

  let actions = SettingsActions {
    update_theme,
    update_priority,
    update_type,
    update_beads_per_page,
    update_auto_backup,
    update_backup_frequency,
    update_keyboard_shortcut,
    update_data_location,
    save_settings,
    reset_settings,
  };

  (settings_state, actions, save_result)
}

/// Validation helper for `beads_per_page`
#[must_use]
pub fn use_beads_per_page_validator() -> Callback<usize, bool> {
  Callback::new(|value: usize| validate_beads_per_page(value))
}

/// Validation helper for data location
#[must_use]
pub fn use_data_location_validator() -> Callback<PathBuf, Option<String>> {
  Callback::new(|path: PathBuf| validate_data_location(&path).err().map(|e| e.to_string()))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_settings_state_loaded() {
    let settings = Settings::default();
    let state = SettingsState::loaded(settings);
    assert!(!state.loading);
    assert!(state.error.is_none());
  }

  #[test]
  fn test_settings_state_loading() {
    let state = SettingsState::loading();
    assert!(state.loading);
    assert!(state.error.is_none());
  }

  #[test]
  fn test_settings_state_with_error() {
    let state = SettingsState::with_error("Test error".to_string());
    assert!(!state.loading);
    assert_eq!(state.error, Some("Test error".to_string()));
  }

  #[test]
  fn test_settings_state_with_settings() {
    let state1 = SettingsState::loaded(Settings::default());
    let new_settings = state1.settings.with_theme(Theme::Dark);
    let state2 = state1.with_settings(new_settings);
    assert_eq!(state2.settings.theme, Theme::Dark);
  }
}
