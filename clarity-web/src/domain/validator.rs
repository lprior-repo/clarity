//! Domain Validation
//!
//! Traits and logic for ensuring domain data integrity and schema alignment.

use crate::domain::error::ClarityError;

/// Trait for types that can be validated against an external schema (e.g., CUE).
pub trait SchemaValidator {
  /// Validates the current instance against its canonical schema.
  /// # Errors
  ///
  /// Returns `ClarityError` if validation fails.
  fn validate_schema(&self) -> Result<(), ClarityError>;

  /// # Errors
  ///
  /// Returns `ClarityError` if JSON is invalid.
  fn vet_json(&self, json: &str, schema_path: &str) -> Result<(), ClarityError>;
}
