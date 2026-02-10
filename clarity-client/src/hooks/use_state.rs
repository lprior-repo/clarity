#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! State management hooks for Clarity desktop app
//!
//! This module provides hooks for accessing global application state
//! and performing common operations.

use crate::app::Route;
use crate::state::{AppState, AuthState, BeadState, Theme, UIState};
use dioxus::prelude::*;
use std::rc::Rc;

// ===== State Access Hook =====

/// Hook to access global application state
///
/// This hook provides type-safe access to the global `AppState`
/// from any component within the provider tree.
///
/// # Panics
/// Panics if used outside of an `AppStateProvider`
#[must_use]
pub fn use_app_state() -> AppState {
  use_context::<AppState>()
}

// ===== Auth State Hooks =====

/// Hook to access authentication state
///
/// Returns an immutable snapshot of the current auth state.
#[must_use]
pub fn use_auth_state() -> AuthState {
  let state = use_app_state();
  state.auth()
}

/// Hook to check if user is authenticated
#[must_use]
pub fn use_is_authenticated() -> bool {
  let state = use_app_state();
  state.is_authenticated()
}

/// Hook to get current user ID
#[must_use]
pub fn use_current_user() -> Option<String> {
  let state = use_app_state();
  state.current_user()
}

// ===== Bead State Hooks =====

/// Hook to access bead state
///
/// Returns an immutable snapshot of the current bead state.
#[must_use]
pub fn use_bead_state() -> BeadState {
  let state = use_app_state();
  state.beads()
}

/// Hook to access the list of beads
///
/// Returns a persistent vector of beads.
#[must_use]
pub fn use_beads() -> rpds::Vector<std::rc::Rc<clarity_core::db::models::Bead>> {
  let state = use_app_state();
  state.bead_list()
}

/// Hook to check if beads are loading
#[must_use]
pub fn use_beads_loading() -> bool {
  let state = use_app_state();
  state.beads_loading()
}

/// Hook to get beads error state
#[must_use]
pub fn use_beads_error() -> Option<String> {
  let state = use_app_state();
  state.beads_error()
}

/// Hook to add a single bead to the state
///
/// Returns a callback that adds a bead to the global state.
#[must_use]
pub fn use_add_bead() -> Callback<clarity_core::db::models::Bead> {
  let state = use_app_state();
  Callback::new(move |bead: clarity_core::db::models::Bead| {
    state.add_bead(bead);
  })
}

// ===== UI State Hooks =====

/// Hook to access UI state
///
/// Returns an immutable snapshot of the current UI state.
#[must_use]
pub fn use_ui_state() -> UIState {
  let state = use_app_state();
  state.ui()
}

/// Hook to get current route
#[must_use]
pub fn use_current_route() -> String {
  let state = use_app_state();
  state.current_route()
}

/// Hook to get current theme
#[must_use]
pub fn use_theme() -> Theme {
  let state = use_app_state();
  state.theme()
}

/// Hook to navigate to a different route
///
/// Returns a function that can be called to navigate to a specific Route.
/// This hook should be used within the `RouteProvider` context.
#[must_use]
pub fn use_navigator() -> impl Fn(Route) + Clone {
  let actions = use_ui_actions();
  move |route: Route| {
    let route_str = format!("{route}");
    (actions.set_route)(route_str);
  }
}

/// Hook to get the current route as a Route enum
///
/// Returns the current route as a Route enum if it can be parsed,
/// or `Route::Home` as a fallback.
#[must_use]
pub fn use_route() -> Option<Route> {
  let state = use_app_state();
  let current_route_str = state.current_route();

  // Parse the current route string into a Route enum
  current_route_str.parse().ok()
}

// ===== Action Hooks =====

/// Hook to get authentication actions
///
/// Returns callbacks for login/logout operations.
#[must_use]
pub fn use_auth_actions() -> AuthActions {
  let state = use_app_state();
  let auth_signal = state.auth;

  AuthActions {
    login: {
      let mut auth_signal = auth_signal;
      Rc::new(move |user_id: String, token: String| {
        let new_auth = AuthState::authenticated(user_id, token);
        auth_signal.set(new_auth);
      })
    },
    logout: {
      let mut auth_signal = auth_signal;
      Rc::new(move || {
        let new_auth = AuthState::unauthenticated();
        auth_signal.set(new_auth);
      })
    },
  }
}

/// Authentication action callbacks
#[derive(Clone)]
pub struct AuthActions {
  /// Login action: sets authenticated state
  pub login: std::rc::Rc<dyn FnMut(String, String)>,
  /// Logout action: clears authenticated state
  pub logout: std::rc::Rc<dyn FnMut()>,
}

impl PartialEq for AuthActions {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.login, &other.login) && Rc::ptr_eq(&self.logout, &other.logout)
  }
}

/// Hook to get bead actions
///
/// Returns callbacks for bead operations.
#[must_use]
pub fn use_bead_actions() -> BeadActions {
  let state = use_app_state();

  BeadActions {
    set_loading: {
      let state = state.clone();
      Rc::new(move || {
        state.set_beads_loading();
      })
    },
    set_beads: {
      let state = state.clone();
      Rc::new(move |beads: Vec<clarity_core::db::models::Bead>| {
        state.set_beads(beads);
      })
    },
    set_error: {
      let state = state.clone();
      Rc::new(move |error: String| {
        state.set_beads_error(error);
      })
    },
    clear_error: {
      let state = state.clone();
      Rc::new(move || {
        state.clear_beads_error();
      })
    },
    add_bead: {
      let state = state;
      Rc::new(move |bead: clarity_core::db::models::Bead| {
        state.add_bead(bead);
      })
    },
  }
}

/// Bead action callbacks
#[derive(Clone)]
pub struct BeadActions {
  /// Set loading state
  pub set_loading: std::rc::Rc<dyn Fn()>,
  /// Update beads list
  pub set_beads: std::rc::Rc<dyn Fn(Vec<clarity_core::db::models::Bead>)>,
  /// Set error state
  pub set_error: std::rc::Rc<dyn Fn(String)>,
  /// Clear error state
  pub clear_error: std::rc::Rc<dyn Fn()>,
  /// Add a single bead
  pub add_bead: std::rc::Rc<dyn Fn(clarity_core::db::models::Bead)>,
}

impl PartialEq for BeadActions {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.set_loading, &other.set_loading) &&
    Rc::ptr_eq(&self.set_beads, &other.set_beads) &&
    Rc::ptr_eq(&self.set_error, &other.set_error) &&
    Rc::ptr_eq(&self.clear_error, &other.clear_error) &&
    Rc::ptr_eq(&self.add_bead, &other.add_bead)
  }
}

/// Hook to get UI actions
///
/// Returns callbacks for UI operations.
#[must_use]
pub fn use_ui_actions() -> UIActions {
  let state = use_app_state();

  UIActions {
    set_route: {
      let state = state.clone();
      Rc::new(move |route: String| {
        state.set_route(route);
      })
    },
    toggle_theme: {
      let state = state.clone();
      Rc::new(move || {
        state.toggle_theme();
      })
    },
    set_theme: {
      let state = state;
      Rc::new(move |theme: Theme| {
        state.set_theme(theme);
      })
    },
  }
}

/// UI action callbacks
#[derive(Clone)]
pub struct UIActions {
  /// Set current route
  pub set_route: std::rc::Rc<dyn Fn(String)>,
  /// Toggle theme
  pub toggle_theme: std::rc::Rc<dyn Fn()>,
  /// Set theme
  pub set_theme: std::rc::Rc<dyn Fn(Theme)>,
}

impl PartialEq for UIActions {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.set_route, &other.set_route) &&
    Rc::ptr_eq(&self.toggle_theme, &other.toggle_theme) &&
    Rc::ptr_eq(&self.set_theme, &other.set_theme)
  }
}

// ===== Effect Hooks =====

/// Hook to load initial application state
///
/// This effect runs once on mount to load persisted state
/// and initialize data from the database.
pub fn use_init_app_state() {
  let _state = use_app_state();

  use_effect(move || {
    // Only run once on mount
    // In the future, this will:
    // 1. Load persisted auth token
    // 2. Validate session
    // 3. Load initial bead list
    // 4. Load user preferences
    //
    // Note: Cannot use tokio::spawn here because AppState is not Send
    // Async initialization must be handled differently for desktop apps
  });
}

/// Hook to sync beads with database
///
/// This effect loads beads from the database and updates state.
pub fn use_sync_beads() {
  let state = use_app_state();
  let has_run = use_signal(|| false);

  use_effect(move || {
    if *has_run.read() {
      // Already run, do nothing
      return;
    }

    state.set_beads_loading();

    // Note: Cannot use tokio::spawn here because AppState is not Send
    // For desktop apps, database operations should be handled differently
    // This is a placeholder for future implementation
  });
}

#[cfg(test)]
mod tests {
  use super::*;

  // Note: Most hook tests require a Dioxus runtime
  // These are placeholder tests to demonstrate the structure

  #[test]
  fn test_auth_actions_clone() {
    // Verify action callbacks can be cloned
    let actions = AuthActions {
      login: std::rc::Rc::new(|_, _| {}),
      logout: std::rc::Rc::new(|| {}),
    };

    let _ = actions.clone();
  }

  #[test]
  fn test_bead_actions_clone() {
    // Verify action callbacks can be cloned
    let actions = BeadActions {
      set_loading: std::rc::Rc::new(|| {}),
      set_beads: std::rc::Rc::new(|_| {}),
      set_error: std::rc::Rc::new(|_| {}),
      clear_error: std::rc::Rc::new(|| {}),
      add_bead: std::rc::Rc::new(|_| {}),
    };

    let _ = actions.clone();
  }

  #[test]
  fn test_ui_actions_clone() {
    // Verify action callbacks can be cloned
    let actions = UIActions {
      set_route: std::rc::Rc::new(|_| {}),
      toggle_theme: std::rc::Rc::new(|| {}),
      set_theme: std::rc::Rc::new(|_| {}),
    };

    let _ = actions.clone();
  }
}
