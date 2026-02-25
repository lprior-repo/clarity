#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(warnings)]
#![allow(clippy::all)]

//! Form validation module
//!
//! Pure, functional validation system for form data.
//! Uses `ValidationResult`<T> for composing field-level validations.

use clarity_core::db::models::{BeadPriority, BeadStatus, BeadType};
use thiserror::Error;
use tracing::{debug, instrument, warn};

/// Validation result type
///
/// Represents either valid data or a collection of validation errors.
/// Multiple errors can be collected for a single field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationResult<T> {
  /// Data passed all validation rules
  Valid(T),
  /// Data failed validation with one or more errors
  Invalid(Vec<ValidationError>),
}

impl<T> ValidationResult<T> {
  /// Map over the valid value
  ///
  /// If the result is Valid, apply the function to the value.
  /// If the result is Invalid, preserve the errors.
  #[must_use]
  pub fn map<U, F>(self, f: F) -> ValidationResult<U>
  where
    F: FnOnce(T) -> U,
  {
    match self {
      Self::Valid(value) => ValidationResult::Valid(f(value)),
      Self::Invalid(errors) => ValidationResult::Invalid(errors),
    }
  }

  /// Check if the result is valid
  #[must_use]
  pub const fn is_valid(&self) -> bool {
    matches!(self, Self::Valid(_))
  }

  /// Check if the result is invalid
  #[must_use]
  pub const fn is_invalid(&self) -> bool {
    matches!(self, Self::Invalid(_))
  }

  /// Get the errors if invalid
  #[must_use]
  pub fn errors(&self) -> Option<&[ValidationError]> {
    match self {
      Self::Valid(_) => None,
      Self::Invalid(errors) => Some(errors),
    }
  }

  /// Combine two validation results
  ///
  /// If both are valid, return valid with a tuple of both values.
  /// If either is invalid, return invalid with combined errors.
  #[must_use]
  pub fn and<U>(self, other: ValidationResult<U>) -> ValidationResult<(T, U)> {
    match (self, other) {
      (Self::Valid(a), ValidationResult::Valid(b)) => ValidationResult::Valid((a, b)),
      (Self::Invalid(errs), ValidationResult::Valid(_)) => ValidationResult::Invalid(errs),
      (Self::Valid(_), ValidationResult::Invalid(errs)) => ValidationResult::Invalid(errs),
      (Self::Invalid(mut errs1), ValidationResult::Invalid(errs2)) => {
        errs1.extend(errs2);
        ValidationResult::Invalid(errs1)
      }
    }
  }
}

/// Field-specific validation error
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ValidationError {
  #[error("title: {0}")]
  Title(String),

  #[error("description: {0}")]
  Description(String),

  #[error("status: {0}")]
  Status(String),

  #[error("priority: {0}")]
  Priority(String),

  #[error("type: {0}")]
  Type(String),
}

impl ValidationError {
  /// Get the field name for this error
  #[must_use]
  pub const fn field_name(&self) -> &str {
    match self {
      Self::Title(_) => "title",
      Self::Description(_) => "description",
      Self::Status(_) => "status",
      Self::Priority(_) => "priority",
      Self::Type(_) => "type",
    }
  }

  /// Get the error message
  #[must_use]
  pub fn message(&self) -> &str {
    match self {
      Self::Title(msg)
      | Self::Description(msg)
      | Self::Status(msg)
      | Self::Priority(msg)
      | Self::Type(msg) => msg,
    }
  }
}

/// Field validator trait
///
/// Defines validation for a single field.
pub trait FieldValidator<T> {
  /// Validate a field value
  fn validate(&self, value: &T) -> ValidationResult<T>;
}

/// Title validation rules
///
/// - Required (non-empty)
/// - Minimum 3 characters
/// - Maximum 100 characters
#[must_use]
#[instrument]
pub fn validate_title(title: &str) -> ValidationResult<String> {
  let trimmed = title.trim();

  debug!(
    original_len = title.len(),
    trimmed_len = trimmed.len(),
    "Validating title"
  );

  let errors = [
    (trimmed.is_empty(), "Title is required"),
    (trimmed.len() < 3, "Title must be at least 3 characters"),
    (trimmed.len() > 100, "Title must be at most 100 characters"),
  ];

  let validation_errors = errors
    .iter()
    .filter(|(is_error, _)| *is_error)
    .map(|(_, msg)| ValidationError::Title(msg.to_string()))
    .collect::<Vec<_>>();

  if validation_errors.is_empty() {
    debug!(title = %trimmed, "Title validation passed");
    ValidationResult::Valid(trimmed.to_string())
  } else {
    warn!(
      error_count = validation_errors.len(),
      "Title validation failed"
    );
    ValidationResult::Invalid(validation_errors)
  }
}

/// Description validation rules
///
/// - Optional
/// - Maximum 1000 characters if present
#[must_use]
pub fn validate_description(description: &str) -> ValidationResult<Option<String>> {
  let trimmed = description.trim();

  if trimmed.is_empty() {
    ValidationResult::Valid(None)
  } else {
    let errors = [(
      trimmed.len() > 1000,
      "Description must be at most 1000 characters",
    )];

    let validation_errors = errors
      .iter()
      .filter(|(is_error, _)| *is_error)
      .map(|(_, msg)| ValidationError::Description(msg.to_string()))
      .collect::<Vec<_>>();

    if validation_errors.is_empty() {
      ValidationResult::Valid(Some(trimmed.to_string()))
    } else {
      ValidationResult::Invalid(validation_errors)
    }
  }
}

/// Status validation rules
///
/// - Must match one of the valid `BeadStatus` values
pub fn validate_status(status: &str) -> ValidationResult<BeadStatus> {
  status.parse::<BeadStatus>().map_or_else(
    |_| {
      ValidationResult::Invalid(vec![ValidationError::Status(
        "Must be a valid status".to_string(),
      )])
    },
    ValidationResult::Valid,
  )
}

/// Priority validation rules
///
/// - Must be between 1 and 3
#[must_use]
pub fn validate_priority(priority: i16) -> ValidationResult<BeadPriority> {
  match BeadPriority::from_value(priority) {
    Ok(p) => ValidationResult::Valid(p),
    Err(_) => {
      ValidationResult::Invalid(vec![ValidationError::Priority(
        "Priority must be between 1 and 3".to_string(),
      )])
    }
  }
}

/// Bead type validation rules
///
/// - Must match one of the valid `BeadType` values
pub fn validate_bead_type(bead_type: &str) -> ValidationResult<BeadType> {
  bead_type.parse::<BeadType>().map_or_else(
    |_| {
      ValidationResult::Invalid(vec![ValidationError::Type(
        "Must be a valid type".to_string(),
      )])
    },
    ValidationResult::Valid,
  )
}

/// Bead form data
///
/// Represents all fields in the bead form for validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BeadFormData {
  pub title: String,
  pub description: String,
  pub status: String,
  pub priority: i16,
  pub bead_type: String,
}

impl BeadFormData {
  /// Create new empty form data
  #[must_use]
  pub fn new() -> Self {
    Self {
      title: String::new(),
      description: String::new(),
      status: "open".to_string(),
      priority: 2,
      bead_type: "feature".to_string(),
    }
  }

  /// Validate the entire form
  ///
  /// Returns all validation errors across all fields.
  #[must_use]
  pub fn validate(&self) -> ValidationResult<Self> {
    // Validate each field and collect all errors
    let title_result = validate_title(&self.title);
    let description_result = validate_description(&self.description);
    let status_result = validate_status(&self.status);
    let priority_result = validate_priority(self.priority);
    let type_result = validate_bead_type(&self.bead_type);

    // Combine all validation results
    let all_valid = [
      title_result.is_valid(),
      description_result.is_valid(),
      status_result.is_valid(),
      priority_result.is_valid(),
      type_result.is_valid(),
    ]
    .iter()
    .all(|v| *v);

    if all_valid {
      ValidationResult::Valid(self.clone())
    } else {
      let all_errors = [
        title_result.errors(),
        description_result.errors(),
        status_result.errors(),
        priority_result.errors(),
        type_result.errors(),
      ]
      .into_iter()
      .flatten()
      .flatten()
      .cloned()
      .collect();

      ValidationResult::Invalid(all_errors)
    }
  }

  /// Get validation errors for a specific field
  #[must_use]
  pub fn field_errors(&self, field: &str) -> Vec<ValidationError> {
    let result = match field {
      "title" => validate_title(&self.title).map(|_| ()),
      "description" => validate_description(&self.description).map(|_| ()),
      "status" => validate_status(&self.status).map(|_| ()),
      "priority" => validate_priority(self.priority).map(|_| ()),
      "type" => validate_bead_type(&self.bead_type).map(|_| ()),
      _ => return Vec::new(),
    };

    match result {
      ValidationResult::Valid(()) => Vec::new(),
      ValidationResult::Invalid(errors) => errors,
    }
  }

  /// Check if a specific field is valid
  #[must_use]
  pub fn is_field_valid(&self, field: &str) -> bool {
    self.field_errors(field).is_empty()
  }
}

impl Default for BeadFormData {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::expect_used)]
  use super::*;

  // Title validation tests
  #[test]
  fn test_validate_title_empty() {
    let result = validate_title("");
    assert!(result.is_invalid());
    assert_eq!(result.errors().unwrap()[0].message(), "Title is required");
  }

  #[test]
  fn test_validate_title_whitespace_only() {
    let result = validate_title("   ");
    assert!(result.is_invalid());
    assert_eq!(result.errors().unwrap()[0].message(), "Title is required");
  }

  #[test]
  fn test_validate_title_too_short() {
    let result = validate_title("ab");
    assert!(result.is_invalid());
    assert_eq!(
      result.errors().unwrap()[0].message(),
      "Title must be at least 3 characters"
    );
  }

  #[test]
  fn test_validate_title_too_long() {
    let result = validate_title(&"a".repeat(101));
    assert!(result.is_invalid());
    assert_eq!(
      result.errors().unwrap()[0].message(),
      "Title must be at most 100 characters"
    );
  }

  #[test]
  fn test_validate_title_valid() {
    let result = validate_title("Valid title");
    assert!(result.is_valid());
    assert_eq!(result, ValidationResult::Valid("Valid title".to_string()));
  }

  #[test]
  fn test_validate_title_with_trimming() {
    let result = validate_title("  Valid title  ");
    assert!(result.is_valid());
    assert_eq!(result, ValidationResult::Valid("Valid title".to_string()));
  }

  // Description validation tests
  #[test]
  fn test_validate_description_empty() {
    let result = validate_description("");
    assert!(result.is_valid());
    assert_eq!(result, ValidationResult::Valid(None));
  }

  #[test]
  fn test_validate_description_whitespace_only() {
    let result = validate_description("   ");
    assert!(result.is_valid());
    assert_eq!(result, ValidationResult::Valid(None));
  }

  #[test]
  fn test_validate_description_too_long() {
    let result = validate_description(&"a".repeat(1001));
    assert!(result.is_invalid());
    assert_eq!(
      result.errors().unwrap()[0].message(),
      "Description must be at most 1000 characters"
    );
  }

  #[test]
  fn test_validate_description_valid() {
    let result = validate_description("Valid description");
    assert!(result.is_valid());
    assert_eq!(
      result,
      ValidationResult::Valid(Some("Valid description".to_string()))
    );
  }

  #[test]
  fn test_validate_description_exactly_max_length() {
    let result = validate_description(&"a".repeat(1000));
    assert!(result.is_valid());
  }

  // Status validation tests
  #[test]
  fn test_validate_status_valid_values() {
    let valid_statuses = ["open", "in_progress", "blocked", "deferred", "closed"];

    for status in valid_statuses {
      let result = validate_status(status);
      assert!(result.is_valid(), "Status '{status}' should be valid");
    }
  }

  #[test]
  fn test_validate_status_invalid() {
    let result = validate_status("invalid");
    assert!(result.is_invalid());
    assert_eq!(
      result.errors().unwrap()[0].message(),
      "Must be a valid status"
    );
  }

  #[test]
  fn test_validate_status_case_insensitive() {
    let result = validate_status("OPEN");
    assert!(result.is_valid());
  }

  // Priority validation tests
  #[test]
  fn test_validate_priority_valid_values() {
    for priority in [1, 2, 3] {
      let result = validate_priority(priority);
      assert!(result.is_valid(), "Priority {priority} should be valid");
    }
  }

  #[test]
  fn test_validate_priority_too_low() {
    let result = validate_priority(0);
    assert!(result.is_invalid());
    assert_eq!(
      result.errors().unwrap()[0].message(),
      "Priority must be between 1 and 3"
    );
  }

  #[test]
  fn test_validate_priority_too_high() {
    let result = validate_priority(4);
    assert!(result.is_invalid());
    assert_eq!(
      result.errors().unwrap()[0].message(),
      "Priority must be between 1 and 3"
    );
  }

  // Bead type validation tests
  #[test]
  fn test_validate_bead_type_valid_values() {
    let valid_types = ["feature", "bugfix", "refactor", "test", "docs"];

    for bead_type in valid_types {
      let result = validate_bead_type(bead_type);
      assert!(result.is_valid(), "Type '{bead_type}' should be valid");
    }
  }

  #[test]
  fn test_validate_bead_type_invalid() {
    let result = validate_bead_type("invalid");
    assert!(result.is_invalid());
    assert_eq!(
      result.errors().unwrap()[0].message(),
      "Must be a valid type"
    );
  }

  #[test]
  fn test_validate_bead_type_case_insensitive() {
    let result = validate_bead_type("FEATURE");
    assert!(result.is_valid());
  }

  // BeadFormData validation tests
  #[test]
  fn test_bead_form_data_validate_all_valid() {
    let form = BeadFormData {
      title: "Valid title".to_string(),
      description: "Valid description".to_string(),
      status: "open".to_string(),
      priority: 2,
      bead_type: "feature".to_string(),
    };

    let result = form.validate();
    assert!(result.is_valid());
  }

  #[test]
  fn test_bead_form_data_validate_multiple_errors() {
    let form = BeadFormData {
      title: "ab".to_string(),          // Too short
      description: "a".repeat(1001),    // Too long
      status: "invalid".to_string(),    // Invalid status
      priority: 5,                      // Invalid priority
      bead_type: "invalid".to_string(), // Invalid type
    };

    let result = form.validate();
    assert!(result.is_invalid());

    let errors = result.errors().unwrap();
    assert_eq!(errors.len(), 5);
  }

  #[test]
  fn test_bead_form_data_field_errors() {
    let form = BeadFormData {
      title: "ab".to_string(),
      description: "".to_string(),
      status: "open".to_string(),
      priority: 2,
      bead_type: "feature".to_string(),
    };

    let title_errors = form.field_errors("title");
    assert_eq!(title_errors.len(), 1);

    let description_errors = form.field_errors("description");
    assert_eq!(description_errors.len(), 0);
  }

  #[test]
  fn test_bead_form_data_is_field_valid() {
    let form = BeadFormData {
      title: "Valid title".to_string(),
      description: "".to_string(),
      status: "open".to_string(),
      priority: 2,
      bead_type: "feature".to_string(),
    };

    assert!(form.is_field_valid("title"));
    assert!(form.is_field_valid("description"));
    assert!(form.is_field_valid("status"));
    assert!(form.is_field_valid("priority"));
    assert!(form.is_field_valid("type"));
  }

  #[test]
  fn test_validation_result_map() {
    let result: ValidationResult<String> = ValidationResult::Valid("hello".to_string());
    let mapped = result.map(|s| s.to_uppercase());
    assert_eq!(mapped, ValidationResult::Valid("HELLO".to_string()));
  }

  #[test]
  fn test_validation_result_map_invalid() {
    let result: ValidationResult<String> =
      ValidationResult::Invalid(vec![ValidationError::Title("error".to_string())]);
    let mapped = result.map(|s| s.to_uppercase());
    assert!(mapped.is_invalid());
  }

  #[test]
  fn test_validation_result_and_both_valid() {
    let result1: ValidationResult<String> = ValidationResult::Valid("hello".to_string());
    let result2: ValidationResult<i32> = ValidationResult::Valid(42);

    let combined = result1.and(result2);
    assert_eq!(combined, ValidationResult::Valid(("hello".to_string(), 42)));
  }

  #[test]
  fn test_validation_result_and_first_invalid() {
    let result1: ValidationResult<String> =
      ValidationResult::Invalid(vec![ValidationError::Title("error".to_string())]);
    let result2: ValidationResult<i32> = ValidationResult::Valid(42);

    let combined = result1.and(result2);
    assert!(combined.is_invalid());
  }

  #[test]
  fn test_validation_result_and_both_invalid() {
    let result1: ValidationResult<String> =
      ValidationResult::Invalid(vec![ValidationError::Title("error1".to_string())]);
    let result2: ValidationResult<i32> =
      ValidationResult::Invalid(vec![ValidationError::Priority("error2".to_string())]);

    let combined = result1.and(result2);
    assert!(combined.is_invalid());
    assert_eq!(combined.errors().unwrap().len(), 2);
  }
}
