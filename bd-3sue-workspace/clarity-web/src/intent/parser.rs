//! JSON Parser for Spec parsing (WP17)
//!
//! This module provides JSON parsing functionality for Spec structures,
//! with comprehensive error handling and validation.
//!
//! ## Design Principles
//!
//! - **Zero panics**: All fallible operations return `Result<T, E>`
//! - **Zero unwrap/expect**: No panics in production code
//! - **Graceful error handling**: Malformed JSON produces helpful error messages

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde_json::Value;
use thiserror::Error;

use super::types::Spec;

/// Error type for parsing operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ParseError {
  /// JSON syntax or structure error
  #[error("JSON error: {0}")]
  JsonError(String),

  /// Required field is missing from the JSON
  #[error("missing required field: {0}")]
  MissingField(String),

  /// Field has incorrect type
  #[error("invalid type for field '{field}': expected {expected}, got {actual}")]
  InvalidType {
    /// Field name
    field: String,
    /// Expected type
    expected: String,
    /// Actual type found
    actual: String,
  },

  /// Required field has empty value
  #[error("empty value for required field: {0}")]
  EmptyField(String),
}

/// Parse a JSON string into a Spec struct
///
/// This function parses the JSON string and validates that required fields
/// (name, version) are present. Optional fields will use their defaults.
///
/// # Errors
///
/// Returns `ParseError` if:
/// - JSON is malformed
/// - Required field `name` is missing or empty
///
/// # Example
///
/// ```rust,ignore
/// use clarity_web::intent::parser::parse_spec;
///
/// let json = r#"{"name": "my-spec", "description": "A spec"}"#;
/// let spec = parse_spec(json)?;
/// assert_eq!(spec.name, "my-spec");
/// ```
pub fn parse_spec(json: &str) -> Result<Spec, ParseError> {
  // Sanitize input string
  let sanitized = sanitize_string(json);

  // Parse JSON
  let value: Value = serde_json::from_str(&sanitized).map_err(|e| {
    ParseError::JsonError(format!(
      "Failed to parse JSON at line {}, column {}: {}",
      e.line(),
      e.column(),
      e
    ))
  })?;

  parse_spec_from_value(&value)
}

/// Parse from an already-parsed JSON value
///
/// This function validates the JSON structure and constructs a Spec.
///
/// # Errors
///
/// Returns `ParseError` if:
/// - Value is not an object
/// - Required field `name` is missing or empty
/// - Field types are incorrect
pub fn parse_spec_from_value(value: &Value) -> Result<Spec, ParseError> {
  // Ensure we have an object
  let obj = value.as_object().ok_or_else(|| ParseError::InvalidType {
    field: "root".to_string(),
    expected: "object".to_string(),
    actual: json_type_name(value),
  })?;

  // Extract and validate name (required)
  let name = extract_string_field(obj, "name")?;

  // Check for empty or whitespace-only name
  if name.trim().is_empty() {
    return Err(ParseError::EmptyField("name".to_string()));
  }

  // Parse the rest using serde (it handles defaults for optional fields)
  let spec: Spec = serde_json::from_value(value.clone())
    .map_err(|e| ParseError::JsonError(format!("Failed to deserialize Spec: {e}")))?;

  Ok(Spec { name, ..spec })
}

/// Sanitize a string by removing null bytes and trimming whitespace
///
/// Null bytes can cause issues with JSON parsing and should be removed.
/// Leading/trailing whitespace is also trimmed.
#[must_use]
pub fn sanitize_string(s: &str) -> String {
  s.chars()
    .filter(|&c| c != '\0')
    .collect::<String>()
    .trim()
    .to_string()
}

/// Validate a Spec for semantic correctness
///
/// Checks that:
/// - `name` is non-empty
/// - `features` list is non-empty (for a meaningful spec)
///
/// # Errors
///
/// Returns `ParseError` if validation fails.
pub fn validate_spec(spec: &Spec) -> Result<(), ParseError> {
  // Check name is non-empty
  if spec.name.trim().is_empty() {
    return Err(ParseError::EmptyField("name".to_string()));
  }

  // Check features is non-empty
  if spec.features.is_empty() {
    return Err(ParseError::EmptyField("features".to_string()));
  }

  Ok(())
}

/// Extract a string field from a JSON object
///
/// # Errors
///
/// Returns `ParseError` if the field is missing or not a string.
fn extract_string_field(
  obj: &serde_json::Map<String, Value>,
  field: &str,
) -> Result<String, ParseError> {
  obj.get(field).map_or_else(
    || Err(ParseError::MissingField(field.to_string())),
    |value| {
      value
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| ParseError::InvalidType {
          field: field.to_string(),
          expected: "string".to_string(),
          actual: json_type_name(value),
        })
    },
  )
}

/// Get a human-readable type name for a JSON value
fn json_type_name(value: &Value) -> String {
  match value {
    Value::Null => "null".to_string(),
    Value::Bool(_) => "boolean".to_string(),
    Value::Number(_) => "number".to_string(),
    Value::String(_) => "string".to_string(),
    Value::Array(_) => "array".to_string(),
    Value::Object(_) => "object".to_string(),
  }
}
