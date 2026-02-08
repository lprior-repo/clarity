//! Integration tests for the Dioxus frontend application
//!
//! These tests verify the complete integration of components,
//! error handling, and application state management.
//!
//! See docs/TESTING.md for testing standards.

use clarity_client::app::{AppError, AppState};

#[test]
fn test_app_state_navigation_flow() {
  let mut state = AppState::new();

  // Test complete navigation flow
  assert!(state.navigate_to("/about".to_string()).is_ok());
  assert_eq!(state.current_route, "/about");

  assert!(state.navigate_to("/contact".to_string()).is_ok());
  assert_eq!(state.current_route, "/contact");

  assert!(state.navigate_to("/".to_string()).is_ok());
  assert_eq!(state.current_route, "/");
}

#[test]
fn test_app_state_error_handling_flow() {
  let mut state = AppState::new();

  // Simulate error scenario
  state.set_error(AppError::ComponentInit(
    "Failed to load component".to_string(),
  ));
  assert!(state.error.is_some());

  // Clear error and verify
  state.clear_error();
  assert!(state.error.is_none());

  // Verify state is still functional after error
  assert!(state.navigate_to("/test".to_string()).is_ok());
}

#[test]
fn test_app_state_error_preserves_on_navigation_failure() {
  let mut state = AppState::new();

  // Set initial valid state
  assert!(state.navigate_to("/valid".to_string()).is_ok());
  assert_eq!(state.current_route, "/valid");

  // Attempt invalid navigation
  let result = state.navigate_to("invalid-path".to_string());
  assert!(result.is_err());

  // Verify state is unchanged after failed navigation
  assert_eq!(state.current_route, "/valid");
}

#[test]
fn test_multiple_errors_in_sequence() {
  let mut state = AppState::new();

  state.set_error(AppError::InvalidRoute("first error".to_string()));
  assert_eq!(
    state.error,
    Some(AppError::InvalidRoute("first error".to_string()))
  );

  // Overwrite with new error
  state.set_error(AppError::StateUpdate("second error".to_string()));
  assert_eq!(
    state.error,
    Some(AppError::StateUpdate("second error".to_string()))
  );

  // Clear and verify
  state.clear_error();
  assert!(state.error.is_none());
}

#[test]
fn test_app_error_equality() {
  let err1 = AppError::InvalidRoute("test".to_string());
  let err2 = AppError::InvalidRoute("test".to_string());
  let err3 = AppError::InvalidRoute("other".to_string());

  assert_eq!(err1, err2);
  assert_ne!(err1, err3);
}

#[test]
fn test_app_state_clone() {
  let mut state = AppState::new();
  assert!(state.navigate_to("/test".to_string()).is_ok());

  let cloned = state.clone();
  assert_eq!(state.current_route, cloned.current_route);
  assert_eq!(state.error, cloned.error);

  // Modify clone doesn't affect original
  let mut cloned = cloned;
  assert!(cloned.navigate_to("/other".to_string()).is_ok());
  assert_eq!(state.current_route, "/test");
  assert_eq!(cloned.current_route, "/other");
}

#[test]
fn test_route_validation_various_cases() {
  let mut state = AppState::new();

  // Valid routes
  let valid_routes = vec![
    "/",
    "/about",
    "/contact",
    "/path/with/multiple/segments",
    "/path-with-dashes",
    "/path_with_underscores",
    "/path123",
    "/path?query=params",
  ];

  for route in valid_routes {
    let result = state.navigate_to(route.to_string());
    assert!(
      result.is_ok(),
      "Route '{}' should be valid, got error: {:?}",
      route,
      result
    );
  }

  // Invalid routes
  let invalid_routes = vec!["", "no-leading-slash", " ", "\t", "\n"];

  for route in invalid_routes {
    let result = state.navigate_to(route.to_string());
    assert!(
      result.is_err(),
      "Route '{}' should be invalid, but got success",
      route
    );
  }
}
