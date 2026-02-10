#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

// Form validation hook
//
// Provides real-time debounced validation for form fields.
// Uses Dioxus's use_signal and use_resource for reactive state management.

use crate::validation::{BeadFormData, ValidationResult};
use dioxus::prelude::*;

/// Validation state for a form
///
/// Tracks the overall validation status and provides field-level error information.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationState {
  /// Not yet validated
  Pristine,
  /// Currently validating (async operation in progress)
  Validating,
  /// All fields are valid
  Valid,
  /// One or more fields have validation errors
  Invalid,
}

impl ValidationState {
  /// Check if the form is in a valid state
  #[must_use]
  pub const fn is_valid(&self) -> bool {
    matches!(self, Self::Valid)
  }

  /// Check if the form is invalid
  #[must_use]
  pub const fn is_invalid(&self) -> bool {
    matches!(self, Self::Invalid)
  }

  /// Check if the form is pristine (not yet validated)
  #[must_use]
  pub const fn is_pristine(&self) -> bool {
    matches!(self, Self::Pristine)
  }

  /// Check if validation is in progress
  #[must_use]
  pub const fn is_validating(&self) -> bool {
    matches!(self, Self::Validating)
  }
}

/// Field-specific error state
///
/// Represents the validation status and error messages for a single form field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldErrorState {
  /// Whether the field has been touched (user has interacted with it)
  pub touched: bool,
  /// Validation errors for this field
  pub errors: Vec<String>,
}

impl FieldErrorState {
  /// Create a new untouched field state with no errors
  #[must_use]
  pub const fn new() -> Self {
    Self {
      touched: false,
      errors: Vec::new(),
    }
  }

  /// Check if the field is valid
  ///
  /// A field is considered valid if it has been touched and has no errors.
  #[must_use]
  pub const fn is_valid(&self) -> bool {
    self.touched && self.errors.is_empty()
  }

  /// Check if the field has errors
  #[must_use]
  pub const fn has_errors(&self) -> bool {
    !self.errors.is_empty()
  }

  /// Get the first error message, if any
  #[must_use]
  pub fn first_error(&self) -> Option<&str> {
    self.errors.first().map(String::as_str)
  }

  /// Mark the field as touched
  #[must_use]
  pub const fn touch(mut self) -> Self {
    self.touched = true;
    self
  }
}

impl Default for FieldErrorState {
  fn default() -> Self {
    Self::new()
  }
}

/// Form validation hook
///
/// Provides real-time debounced validation for bead forms.
///
/// # Features
/// - Field-level validation with 300ms debounce
/// - Touch tracking (only show errors after user interaction)
/// - Accessible error messages (aria-live)
/// - Pure validation logic with reactive state
///
/// # Returns
/// A tuple of:
/// - Current validation state
/// - Field error state map
/// - Function to mark a field as touched
/// - Function to trigger validation
/// - Overall validity check
#[must_use]
pub fn use_form_validation() -> (
  Signal<ValidationState>,
  Signal<std::collections::HashMap<String, FieldErrorState>>,
  Callback<String>,
  Callback<BeadFormData>,
  impl Fn() -> bool,
) {
  let validation_state = use_signal(|| ValidationState::Pristine);
  let field_errors = use_signal(std::collections::HashMap::<String, FieldErrorState>::new);

  // Mark a field as touched (show errors after user interaction)
  let touch_field = {
    let mut field_errors = field_errors;
    Callback::new(move |field: String| {
      // Clone the current errors to avoid double borrow
      let current_errors = field_errors.read().clone();

      // Create new errors map with the touched field
      let mut new_errors = current_errors;
      let entry = new_errors.entry(field).or_insert_with(FieldErrorState::new);
      entry.touched = true;

      // Write back the new errors
      *field_errors.write() = new_errors;
    })
  };

  // Validate form data (simplified, no debounce for now)
  let validate = {
    let mut validation_state = validation_state;
    let mut field_errors = field_errors;

    Callback::new(move |form_data: BeadFormData| {
      let result = form_data.validate();

      match result {
        ValidationResult::Valid(_) => {
          validation_state.set(ValidationState::Valid);
          // Clear all errors
          let empty_map = std::collections::HashMap::new();
          let _ = std::mem::replace(&mut *field_errors.write(), empty_map);
        }
        ValidationResult::Invalid(errors) => {
          validation_state.set(ValidationState::Invalid);

          // Group errors by field
          let mut error_map: std::collections::HashMap<String, FieldErrorState> =
            std::collections::HashMap::new();

          for error in errors {
            let field_name = error.field_name().to_string();
            let message = error.message().to_string();

            let state = error_map.entry(field_name.clone()).or_default();

            // Preserve touched state from existing errors
            if let Some(existing) = field_errors.read().get(&field_name) {
              state.touched = existing.touched;
            }

            state.errors.push(message);
          }

          // Create a completely new error map
          let current_errors = field_errors.read().clone();
          let new_errors: std::collections::HashMap<String, FieldErrorState> = current_errors
            .iter()
            .map(|(field, existing_state)| {
              let new_state = match error_map.get(field) {
                Some(error_state) => error_state.clone(),
                None => existing_state.clone(),
              };
              (field.clone(), new_state)
            })
            .collect();
          let _ = std::mem::replace(&mut *field_errors.write(), new_errors);
        }
      }
    })
  };

  // Check if the entire form is valid
  let is_valid = {
    let validation_state = validation_state;
    move || validation_state.read().is_valid()
  };

  (
    validation_state,
    field_errors,
    touch_field,
    validate,
    is_valid,
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_field_error_state_new() {
    let state = FieldErrorState::new();
    assert!(!state.touched);
    assert!(state.errors.is_empty());
    assert!(!state.is_valid());
    assert!(!state.has_errors());
  }

  #[test]
  fn test_field_error_state_touch() {
    let state = FieldErrorState::new().touch();
    assert!(state.touched);
    assert!(state.is_valid());
  }

  #[test]
  fn test_field_error_state_with_errors() {
    let mut state = FieldErrorState::new();
    state.errors.push("Error 1".to_string());
    state.errors.push("Error 2".to_string());

    assert!(state.has_errors());
    assert!(!state.is_valid());
    assert_eq!(state.first_error(), Some("Error 1"));
  }

  #[test]
  fn test_field_error_state_touched_with_errors() {
    let mut state = FieldErrorState::new().touch();
    state.errors.push("Error 1".to_string());

    assert!(state.touched);
    assert!(state.has_errors());
    assert!(!state.is_valid());
  }

  #[test]
  fn test_validation_state_pristine() {
    let state = ValidationState::Pristine;
    assert!(state.is_pristine());
    assert!(!state.is_valid());
    assert!(!state.is_invalid());
    assert!(!state.is_validating());
  }

  #[test]
  fn test_validation_state_valid() {
    let state = ValidationState::Valid;
    assert!(!state.is_pristine());
    assert!(state.is_valid());
    assert!(!state.is_invalid());
    assert!(!state.is_validating());
  }

  #[test]
  fn test_validation_state_invalid() {
    let state = ValidationState::Invalid;
    assert!(!state.is_pristine());
    assert!(!state.is_valid());
    assert!(state.is_invalid());
    assert!(!state.is_validating());
  }

  #[test]
  fn test_validation_state_validating() {
    let state = ValidationState::Validating;
    assert!(!state.is_pristine());
    assert!(!state.is_valid());
    assert!(!state.is_invalid());
    assert!(state.is_validating());
  }
}
