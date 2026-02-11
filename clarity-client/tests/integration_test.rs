//! Integration tests for the Dioxus frontend application
//!
//! These tests verify error handling and other pure functionality
//! that doesn't require a Dioxus runtime.

use clarity_client::AppError;

#[test]
fn test_app_error_equality() {
  let err1 = AppError::network("test".to_string(), "internal".to_string(), true);
  let err2 = AppError::network("test".to_string(), "internal".to_string(), true);
  let err3 = AppError::network("other".to_string(), "internal".to_string(), true);

  assert_eq!(err1, err2);
  assert_ne!(err1, err3);
}

#[test]
fn test_app_error_recovery_action_labels() {
  // Test network error recovery
  let actions =
    AppError::network("test".to_string(), "internal".to_string(), true).recovery_actions();
  assert!(!actions.is_empty());
  // Check that "Try Again" is one of the actions
  assert!(actions.iter().any(|a| a.label() == "Try Again"));

  // Test validation error recovery
  let actions = AppError::validation_with_field(
    "test".to_string(),
    "field".to_string(),
    "internal".to_string(),
  )
  .recovery_actions();
  assert!(!actions.is_empty());
  // Check that we have some recovery action for validation
  assert!(actions.iter().any(|a| !a.label().is_empty()));
}
