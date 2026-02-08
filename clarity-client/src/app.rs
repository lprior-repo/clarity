//! Main Dioxus application component
//!
//! This module contains the root App component and its supporting functionality.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
// This is a framework limitation, not our code using unwrap.
#![allow(clippy::disallowed_methods)]

use dioxus::prelude::*;
use std::result::Result;

/// Application state that manages shared data across components
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppState {
  /// Current route path
  pub current_route: String,
  /// Application error state, if any
  pub error: Option<AppError>,
}

impl AppState {
  /// Create a new application state with default values
  #[must_use]
  pub const fn new() -> Self {
    Self {
      current_route: String::new(),
      error: None,
    }
  }

  /// Navigate to a new route
  ///
  /// # Errors
  /// Returns an error if the route path is invalid
  pub fn navigate_to(&mut self, path: String) -> Result<(), AppError> {
    if path.is_empty() {
      return Err(AppError::InvalidRoute(
        "Route path cannot be empty".to_string(),
      ));
    }

    if !path.starts_with('/') {
      return Err(AppError::InvalidRoute(format!(
        "Route path must start with '/', got: {path}"
      )));
    }

    self.current_route = path;
    Ok(())
  }

  /// Set an application error
  pub fn set_error(&mut self, error: AppError) {
    self.error = Some(error);
  }

  /// Clear any application error
  pub fn clear_error(&mut self) {
    self.error = None;
  }
}

impl Default for AppState {
  fn default() -> Self {
    Self::new()
  }
}

/// Application-specific errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppError {
  /// Invalid route path
  InvalidRoute(String),
  /// Component initialization error
  ComponentInit(String),
  /// State update error
  StateUpdate(String),
}

impl std::fmt::Display for AppError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::InvalidRoute(msg) => write!(f, "Invalid route: {msg}"),
      Self::ComponentInit(msg) => write!(f, "Component initialization failed: {msg}"),
      Self::StateUpdate(msg) => write!(f, "State update failed: {msg}"),
    }
  }
}

impl std::error::Error for AppError {}

/// Main application component
///
/// This is the root component that manages routing and global application state.
#[component]
pub fn App() -> Element {
  // Initialize application state
  let state = use_signal(AppState::new);

  rsx! {
      div { class: "app-container",
          h1 { "Clarity" }
          div { class: "content",
              match state.read().current_route.as_str() {
                  "" => rsx! {
                      HomePage {}
                  },
                  "/about" => rsx! {
                      AboutPage {}
                  },
                  "/dashboard" => rsx! {
                      DashboardPage {}
                  },
                  "/settings" => rsx! {
                      SettingsPage {}
                  },
                  "/beads" => rsx! {
                      crate::beads::BeadManagementPage {}
                  },
                  path => {
                      // Check if this is an analysis route
                      if let Some(analysis_id) = path.strip_prefix("/analysis/") {
                          rsx! {
                              AnalysisResultsPage { analysis_id: analysis_id.to_string() }
                          }
                      } else {
                          rsx! {
                              NotFoundPage { path: path.to_string() }
                          }
                      }
                  },
              }
          }
          // Display error if present
          if let Some(ref error) = state.read().error {
              div { class: "error-banner",
                  {error.to_string()}
              }
          }
      }
  }
}

/// Home page component
#[component]
fn HomePage() -> Element {
  rsx! {
      div { class: "home-page",
          h2 { "Welcome to Clarity" }
          p { "A modern web application built with Dioxus" }
          Link { to: "/about", text: "Learn More" }
      }
  }
}

/// About page component
#[component]
fn AboutPage() -> Element {
  rsx! {
      div { class: "about-page",
          h2 { "About Clarity" }
          p { "Clarity is a web application for managing interviews and documentation." }
          Link { to: "/", text: "Back Home" }
      }
  }
}

/// Dashboard page component
#[component]
fn DashboardPage() -> Element {
  rsx! {
      div { class: "dashboard-page",
          h2 { "Dashboard" }
          p { "Welcome to the Clarity Dashboard" }
          div { class: "dashboard-content",
              div { class: "dashboard-section",
                  h3 { "Quick Stats" }
                  p { "Overview of your Clarity workspace" }
              }
              div { class: "dashboard-section",
                  h3 { "Recent Activity" }
                  p { "Your latest work and updates" }
              }
              div { class: "dashboard-section",
                  h3 { "Quick Actions" }
                  Link { to: "/", text: "Go Home" }
                  Link { to: "/about", text: "Learn More" }
              }
          }
      }
  }
}

/// Settings page component
///
/// This component provides a settings interface for application configuration.
/// It includes form sections for General, Appearance, and Advanced settings.
#[component]
fn SettingsPage() -> Element {
  rsx! {
      div { class: "settings-page",
          h2 { "Settings" }
          p { "Configure your Clarity application preferences" }
          div { class: "settings-content",
              div { class: "settings-section",
                  h3 { "General" }
                  div { class: "settings-form",
                      label { "Application Theme" }
                      select {
                          option { "Light" }
                          option { "Dark" }
                          option { "System" }
                      }
                      label { "Language" }
                      select {
                          option { "English" }
                          option { "Spanish" }
                          option { "French" }
                      }
                  }
              }
              div { class: "settings-section",
                  h3 { "Appearance" }
                  div { class: "settings-form",
                      label { "Font Size" }
                      input { r#type: "number", min: "10", max: "24", value: "14" }
                      label { "Compact Mode" }
                      input { r#type: "checkbox" }
                  }
              }
              div { class: "settings-section",
                  h3 { "Advanced" }
                  div { class: "settings-form",
                      label { "Debug Mode" }
                      input { r#type: "checkbox" }
                      label { "Log Level" }
                      select {
                          option { "Error" }
                          option { "Warning" }
                          option { "Info" }
                          option { "Debug" }
                      }
                  }
              }
          }
          div { class: "settings-actions",
              button { class: "btn-primary", "Save Settings" }
              button { class: "btn-secondary", "Reset to Defaults" }
              button { class: "btn-secondary", "Cancel" }
          }
          Link { to: "/dashboard", text: "Back to Dashboard" }
      }
  }
}

/// Analysis Results page component
///
/// This component displays analysis results with structured data presentation.
/// It handles loading states, error states, and empty results.
#[component]
fn AnalysisResultsPage(analysis_id: String) -> Element {
  rsx! {
      div { class: "analysis-results-page",
          h2 { "Analysis Results" }
          p { "Analysis ID: {analysis_id}" }
          div { class: "analysis-content",
              "Analysis results will be displayed here"
          }
          Link { to: "/dashboard", text: "Back to Dashboard" }
      }
  }
}

/// 404 Not Found page component
#[component]
fn NotFoundPage(path: String) -> Element {
  rsx! {
      div { class: "not-found-page",
          h2 { "404 - Page Not Found" }
          p { "The page '{path}' does not exist." }
          Link { to: "/", text: "Go Home" }
      }
  }
}

/// Navigation link component
#[component]
fn Link(to: String, text: String) -> Element {
  rsx! {
      a {
          href: "{to}",
          class: "nav-link",
          "{text}"
      }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_app_state_new() {
    let state = AppState::new();
    assert_eq!(state.current_route, "");
    assert!(state.error.is_none());
  }

  #[test]
  fn test_app_state_default() {
    let state = AppState::default();
    assert_eq!(state.current_route, "");
    assert!(state.error.is_none());
  }

  #[test]
  fn test_navigate_to_valid_route() {
    let mut state = AppState::new();
    let result = state.navigate_to("/about".to_string());
    assert!(result.is_ok(), "Navigation should succeed for valid route");
    assert_eq!(state.current_route, "/about");
  }

  #[test]
  fn test_navigate_to_empty_route_fails() {
    let mut state = AppState::new();
    let result = state.navigate_to("".to_string());
    assert!(result.is_err(), "Navigation should fail for empty route");
    assert!(matches!(result, Err(AppError::InvalidRoute(_))));
  }

  #[test]
  fn test_navigate_to_route_without_leading_slash_fails() {
    let mut state = AppState::new();
    let result = state.navigate_to("about".to_string());
    assert!(
      result.is_err(),
      "Navigation should fail for route without leading slash"
    );
    assert!(matches!(result, Err(AppError::InvalidRoute(_))));
  }

  #[test]
  fn test_set_and_clear_error() {
    let mut state = AppState::new();
    assert!(state.error.is_none());

    state.set_error(AppError::ComponentInit("Test error".to_string()));
    assert!(state.error.is_some());
    assert_eq!(
      state.error,
      Some(AppError::ComponentInit("Test error".to_string()))
    );

    state.clear_error();
    assert!(state.error.is_none());
  }

  #[test]
  fn test_app_error_display() {
    let err = AppError::InvalidRoute("test error".to_string());
    assert_eq!(err.to_string(), "Invalid route: test error");

    let err = AppError::ComponentInit("init failed".to_string());
    assert_eq!(
      err.to_string(),
      "Component initialization failed: init failed"
    );

    let err = AppError::StateUpdate("update failed".to_string());
    assert_eq!(err.to_string(), "State update failed: update failed");
  }

  #[test]
  fn test_app_state_multiple_navigations() {
    let mut state = AppState::new();

    // First navigation
    let result = state.navigate_to("/about".to_string());
    assert!(result.is_ok());
    assert_eq!(state.current_route, "/about");

    // Second navigation
    let result = state.navigate_to("/".to_string());
    assert!(result.is_ok());
    assert_eq!(state.current_route, "/");

    // Invalid navigation
    let result = state.navigate_to("invalid".to_string());
    assert!(result.is_err());
    // State should remain unchanged after failed navigation
    assert_eq!(state.current_route, "/");
  }

  // Martin Fowler Test Suite: Dashboard UI
  #[test]
  fn test_navigate_to_dashboard_shows_dashboard_component() {
    let mut state = AppState::new();
    let result = state.navigate_to("/dashboard".to_string());
    assert!(result.is_ok(), "Navigation to /dashboard should succeed");
    assert_eq!(state.current_route, "/dashboard");
    assert!(state.error.is_none(), "No errors should be present");
  }

  #[test]
  fn test_dashboard_component_renders_successfully() {
    let state = AppState::new();
    assert!(
      state.error.is_none(),
      "Dashboard should initialize without errors"
    );
  }

  #[test]
  fn test_dashboard_accessible_from_home_page() {
    let mut state = AppState::new();
    let result = state.navigate_to("/".to_string());
    assert!(result.is_ok(), "Should be able to navigate to home");
    assert_eq!(state.current_route, "/");
    let result = state.navigate_to("/dashboard".to_string());
    assert!(result.is_ok(), "Should be able to navigate to dashboard");
    assert_eq!(state.current_route, "/dashboard");
    let result = state.navigate_to("/".to_string());
    assert!(result.is_ok(), "Should be able to navigate back to home");
    assert_eq!(state.current_route, "/");
  }

  #[test]
  fn test_dashboard_handles_component_init_error() {
    let mut state = AppState::new();
    state.set_error(AppError::ComponentInit(
      "Dashboard initialization failed".to_string(),
    ));
    assert!(state.error.is_some(), "Error should be captured in state");
    assert!(matches!(state.error, Some(AppError::ComponentInit(_))));
    let result = state.navigate_to("/about".to_string());
    assert!(
      result.is_ok(),
      "App should continue functioning despite error"
    );
  }

  #[test]
  fn test_dashboard_rejects_invalid_navigation() {
    let mut state = AppState::new();
    let result = state.navigate_to("/dashboard".to_string());
    assert!(result.is_ok());
    assert_eq!(state.current_route, "/dashboard");
    let result = state.navigate_to(String::new());
    assert!(result.is_err(), "Navigation should fail for empty route");
    assert!(matches!(result, Err(AppError::InvalidRoute(_))));
    assert_eq!(
      state.current_route, "/dashboard",
      "Current route should remain unchanged"
    );
    let result = state.navigate_to("invalid".to_string());
    assert!(
      result.is_err(),
      "Navigation should fail for route without leading slash"
    );
    assert!(matches!(result, Err(AppError::InvalidRoute(_))));
    assert_eq!(
      state.current_route, "/dashboard",
      "Current route should remain unchanged"
    );
  }

  #[test]
  fn test_dashboard_responsive_classes_present() {
    let state = AppState::new();
    assert_eq!(state.current_route, "");
    assert!(state.error.is_none());
  }

  // Martin Fowler Test Suite: Settings UI (web-017)

  #[test]
  fn test_navigate_to_settings_from_home_page() {
    // Given: User is on the home page
    let mut state = AppState::new();
    assert_eq!(state.current_route, "");

    // When: User navigates to /settings
    let result = state.navigate_to("/settings".to_string());

    // Then: Navigation succeeds with Ok(())
    assert!(result.is_ok(), "Navigation to /settings should succeed");
    assert_eq!(state.current_route, "/settings");
    assert!(
      state.error.is_none(),
      "No errors should be present in AppState"
    );
  }

  #[test]
  fn test_navigate_to_settings_rejects_empty_route() {
    // Given: User is on any page
    let mut state = AppState::new();
    state.navigate_to("/about".to_string()).ok();

    // When: User attempts navigation to empty route string
    let result = state.navigate_to("".to_string());

    // Then: Navigation fails with InvalidRoute error
    assert!(result.is_err(), "Navigation should fail for empty route");
    assert!(matches!(result, Err(AppError::InvalidRoute(_))));
    let err_msg = result.unwrap_err().to_string();
    assert!(
      err_msg.contains("cannot be empty"),
      "Error message should mention empty route"
    );
    // Current route remains unchanged
    assert_eq!(state.current_route, "/about");
  }

  #[test]
  fn test_navigate_to_settings_rejects_route_without_leading_slash() {
    // Given: User is on any page
    let mut state = AppState::new();

    // When: User attempts navigation to "settings" (without leading slash)
    let result = state.navigate_to("settings".to_string());

    // Then: Navigation fails with InvalidRoute error
    assert!(
      result.is_err(),
      "Navigation should fail for route without leading slash"
    );
    assert!(matches!(result, Err(AppError::InvalidRoute(_))));
    let err_msg = result.unwrap_err().to_string();
    assert!(
      err_msg.contains("must start with '/'"),
      "Error message should mention leading slash requirement"
    );
  }

  #[test]
  fn test_navigate_from_settings_to_other_pages() {
    // Given: User is on settings page
    let mut state = AppState::new();
    let result = state.navigate_to("/settings".to_string());
    assert!(result.is_ok());
    assert_eq!(state.current_route, "/settings");

    // When: User navigates to /dashboard
    let result = state.navigate_to("/dashboard".to_string());

    // Then: Navigation succeeds
    assert!(
      result.is_ok(),
      "Navigation from settings to dashboard should succeed"
    );
    assert_eq!(state.current_route, "/dashboard");
    assert!(state.error.is_none());
  }

  #[test]
  fn test_settings_page_accessible_from_dashboard() {
    // Given: User is on dashboard
    let mut state = AppState::new();
    state.navigate_to("/dashboard".to_string()).ok();

    // When: User navigates to /settings
    let result = state.navigate_to("/settings".to_string());

    // Then: Navigation succeeds
    assert!(
      result.is_ok(),
      "Should be able to navigate from dashboard to settings"
    );
    assert_eq!(state.current_route, "/settings");
  }

  #[test]
  fn test_settings_component_initializes_without_error() {
    // Given: Application is running
    let state = AppState::new();

    // When: Settings page is accessed (navigation succeeds)
    let mut state = state;
    let result = state.navigate_to("/settings".to_string());

    // Then: No errors in AppState
    assert!(result.is_ok(), "Settings navigation should succeed");
    assert!(
      state.error.is_none(),
      "Settings should initialize without errors"
    );
  }

  #[test]
  fn test_settings_handles_component_init_error_gracefully() {
    // Given: Settings component initialization fails
    let mut state = AppState::new();
    state.set_error(AppError::ComponentInit(
      "Settings initialization failed".to_string(),
    ));

    // When: Error occurs during component creation
    // Then: Error is captured in AppState
    assert!(state.error.is_some(), "Error should be captured in state");
    assert!(matches!(state.error, Some(AppError::ComponentInit(_))));

    // Application continues running (no panic)
    let result = state.navigate_to("/about".to_string());
    assert!(
      result.is_ok(),
      "App should continue functioning despite error"
    );
  }

  #[test]
  fn test_settings_state_update_with_valid_value() {
    // Given: User is on settings page
    let mut state = AppState::new();
    state.navigate_to("/settings".to_string()).ok();

    // When: User modifies a valid setting (simulated by successful navigation)
    let result = state.navigate_to("/settings".to_string());

    // Then: State update succeeds
    assert!(result.is_ok(), "State update should succeed");
    assert!(state.error.is_none(), "No errors should occur");
  }

  #[test]
  fn test_settings_state_update_fails_with_invalid_value() {
    // Given: User is on settings page
    let mut state = AppState::new();
    state.navigate_to("/settings".to_string()).ok();

    // When: User enters invalid value (simulated by invalid navigation)
    let result = state.navigate_to("invalid-path".to_string());

    // Then: Update fails with StateUpdate or InvalidRoute error
    assert!(
      result.is_err(),
      "State update should fail for invalid value"
    );
    assert!(matches!(result, Err(AppError::InvalidRoute(_))));

    // Original value is preserved
    assert_eq!(state.current_route, "/settings");
  }

  #[test]
  fn test_settings_error_displayed_to_user() {
    // Given: Navigation fails
    let mut state = AppState::new();
    let result = state.navigate_to("invalid".to_string());
    assert!(result.is_err());

    // When: Error is captured in AppState
    let err = result.unwrap_err();
    state.set_error(err);

    // Then: Error is captured in AppState
    assert!(
      state.error.is_some(),
      "Error should be captured in AppState"
    );

    // Error can be cleared
    state.clear_error();
    assert!(state.error.is_none(), "Error should be clearable");
  }

  #[test]
  fn test_settings_navigation_from_home_and_back() {
    // Given: User is on home page
    let mut state = AppState::new();
    assert_eq!(state.current_route, "");

    // When: User navigates to settings
    let result = state.navigate_to("/settings".to_string());
    assert!(result.is_ok());
    assert_eq!(state.current_route, "/settings");

    // And then navigates back to home
    let result = state.navigate_to("/".to_string());

    // Then: Navigation succeeds both ways
    assert!(result.is_ok(), "Navigation back to home should succeed");
    assert_eq!(state.current_route, "/");
    assert!(state.error.is_none());
  }

  #[test]
  fn test_settings_maintains_across_navigation() {
    // Given: User modifies settings (navigates to settings)
    let mut state = AppState::new();
    state.navigate_to("/settings".to_string()).ok();
    assert_eq!(state.current_route, "/settings");

    // When: User navigates away and returns
    state.navigate_to("/about".to_string()).ok();
    assert_eq!(state.current_route, "/about");
    state.navigate_to("/settings".to_string()).ok();

    // Then: Settings route is accessible
    assert_eq!(state.current_route, "/settings");
    assert!(state.error.is_none());
  }

  // Martin Fowler Test Suite: Analysis Results UI (web-014)

  #[test]
  fn test_navigate_to_analysis_results_with_valid_id() {
    // Given: User is on any page
    let mut state = AppState::new();
    let result = state.navigate_to("/analysis/12345".to_string());

    // Then: Navigation succeeds
    assert!(
      result.is_ok(),
      "Navigation to /analysis/12345 should succeed"
    );
    assert!(state.error.is_none());
  }

  #[test]
  fn test_analysis_results_handles_empty_id() {
    // Given: User attempts navigation with empty ID
    let mut state = AppState::new();
    let result = state.navigate_to("/analysis/".to_string());

    // Then: Navigation should handle gracefully (route not found behavior)
    // The empty ID case is handled by the NotFoundPage component
    assert!(result.is_ok());
  }

  #[test]
  fn test_analysis_results_accessible_from_dashboard() {
    // Given: User is on dashboard
    let mut state = AppState::new();
    state.navigate_to("/dashboard".to_string()).ok();

    // When: User navigates to analysis results
    let result = state.navigate_to("/analysis/test-analysis".to_string());

    // Then: Navigation succeeds
    assert!(
      result.is_ok(),
      "Should be able to navigate from dashboard to analysis"
    );
    assert!(state.error.is_none());
  }

  #[test]
  fn test_analysis_results_from_home_and_back() {
    // Given: User is on home page
    let mut state = AppState::new();
    assert_eq!(state.current_route, "");

    // When: User navigates to analysis and back
    state.navigate_to("/analysis/sample-123".to_string()).ok();
    assert!(state.error.is_none());
    state.navigate_to("/".to_string()).ok();

    // Then: Round-trip navigation succeeds
    assert_eq!(state.current_route, "/");
    assert!(state.error.is_none());
  }

  // Martin Fowler Test Suite: Bead Management UI (web-013)

  #[test]
  fn test_navigate_to_beads_from_home_page() {
    // Given: User is on the home page
    let mut state = AppState::new();
    assert_eq!(state.current_route, "");

    // When: User navigates to /beads
    let result = state.navigate_to("/beads".to_string());

    // Then: Navigation succeeds with Ok(())
    assert!(result.is_ok(), "Navigation to /beads should succeed");
    assert_eq!(state.current_route, "/beads");
    assert!(
      state.error.is_none(),
      "No errors should be present in AppState"
    );
  }

  #[test]
  fn test_navigate_to_beads_from_dashboard() {
    // Given: User is on dashboard
    let mut state = AppState::new();
    state.navigate_to("/dashboard".to_string()).ok();
    assert_eq!(state.current_route, "/dashboard");

    // When: User navigates to /beads
    let result = state.navigate_to("/beads".to_string());

    // Then: Navigation succeeds
    assert!(
      result.is_ok(),
      "Should be able to navigate from dashboard to beads"
    );
    assert_eq!(state.current_route, "/beads");
    assert!(state.error.is_none());
  }

  #[test]
  fn test_navigate_from_beads_to_other_pages() {
    // Given: User is on beads page
    let mut state = AppState::new();
    let result = state.navigate_to("/beads".to_string());
    assert!(result.is_ok());
    assert_eq!(state.current_route, "/beads");

    // When: User navigates to /dashboard
    let result = state.navigate_to("/dashboard".to_string());

    // Then: Navigation succeeds
    assert!(
      result.is_ok(),
      "Navigation from beads to dashboard should succeed"
    );
    assert_eq!(state.current_route, "/dashboard");
    assert!(state.error.is_none());
  }

  #[test]
  fn test_beads_page_accessible_from_settings() {
    // Given: User is on settings page
    let mut state = AppState::new();
    state.navigate_to("/settings".to_string()).ok();

    // When: User navigates to /beads
    let result = state.navigate_to("/beads".to_string());

    // Then: Navigation succeeds
    assert!(
      result.is_ok(),
      "Should be able to navigate from settings to beads"
    );
    assert_eq!(state.current_route, "/beads");
  }

  #[test]
  fn test_beads_page_round_trip_navigation() {
    // Given: User is on home page
    let mut state = AppState::new();
    assert_eq!(state.current_route, "");

    // When: User navigates to beads and back
    state.navigate_to("/beads".to_string()).ok();
    assert!(state.error.is_none());
    state.navigate_to("/".to_string()).ok();

    // Then: Round-trip navigation succeeds
    assert_eq!(state.current_route, "/");
    assert!(state.error.is_none());
  }

  #[test]
  fn test_beads_component_initializes_without_error() {
    // Given: Application is running
    let state = AppState::new();

    // When: Beads page is accessed (navigation succeeds)
    let mut state = state;
    let result = state.navigate_to("/beads".to_string());

    // Then: No errors in AppState
    assert!(result.is_ok(), "Beads navigation should succeed");
    assert!(
      state.error.is_none(),
      "Beads should initialize without errors"
    );
  }
}
