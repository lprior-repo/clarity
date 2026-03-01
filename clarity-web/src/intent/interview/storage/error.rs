//! Comprehensive JSON decode error handling for storage operations.
//!
//! This module provides detailed error types for JSON parsing failures
//! with field context, actionable messages, and input sanitization.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use std::fmt;
use thiserror::Error;

/// Maximum length for sanitized error context.
const MAX_CONTEXT_LENGTH: usize = 200;

/// Maximum length for field names in error messages.
const MAX_FIELD_NAME_LENGTH: usize = 100;

/// Sanitize input for safe display in error messages.
///
/// Truncates long strings and replaces control characters with placeholders.
///
/// # Arguments
///
/// * `input` - The input string to sanitize
///
/// # Returns
///
/// A sanitized string safe for display in logs and error messages.
#[must_use]
pub fn sanitize_for_display(input: &str) -> String {
  let sanitized: String = input
    .chars()
    .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
    .collect();

  if sanitized.len() > MAX_CONTEXT_LENGTH {
    let truncated: String = sanitized.chars().take(MAX_CONTEXT_LENGTH).collect();
    format!("{truncated}... (truncated, {} bytes total)", input.len())
  } else {
    sanitized
  }
}

/// Sanitize a field name for safe display.
///
/// Ensures field names don't contain malicious content and are reasonable length.
///
/// # Arguments
///
/// * `field` - The field name to sanitize
///
/// # Returns
///
/// A sanitized field name.
#[must_use]
pub fn sanitize_field_name(field: &str) -> String {
  let sanitized: String = field
    .chars()
    .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
    .take(MAX_FIELD_NAME_LENGTH)
    .collect();

  if sanitized.is_empty() {
    "<invalid_field>".to_string()
  } else if field.len() > MAX_FIELD_NAME_LENGTH {
    format!("{sanitized}... (truncated)")
  } else {
    sanitized
  }
}

/// Extract the JSON path from a serde_json error.
///
/// Parses the error message to find the field path where the error occurred.
///
/// # Arguments
///
/// * `error` - The serde_json error
///
/// # Returns
///
/// An optional JSON path string.
fn extract_json_path(error: &serde_json::Error) -> Option<String> {
  let message = error.to_string();

  // serde_json errors often include paths like "answers[0].question_id"
  if let Some(start) = message.find(':') {
    let after_colon = &message[start + 1..].trim();
    // Look for quoted field paths
    if after_colon.starts_with('"') {
      if let Some(end) = after_colon[1..].find('"') {
        return Some(after_colon[1..end + 1].to_string());
      }
    }
  }

  None
}

/// Classify the type of JSON parsing error.
///
/// # Arguments
///
/// * `error` - The serde_json error
///
/// # Returns
///
/// A `JsonErrorKind` describing the error type.
#[must_use]
pub fn classify_json_error(error: &serde_json::Error) -> JsonErrorKind {
  let message = error.to_string().to_lowercase();
  let is_eof = message.contains("eof") || message.contains("end of file");
  let is_syntax = message.contains("expected")
    || message.contains("invalid")
    || message.contains("unexpected")
    || message.contains("trailing");
  let is_type = message.contains("type")
    || message.contains("number")
    || message.contains("string")
    || message.contains("boolean")
    || message.contains("array")
    || message.contains("object");
  let is_missing = message.contains("missing field");
  let is_duplicate = message.contains("duplicate");

  match (is_eof, is_syntax, is_type, is_missing, is_duplicate) {
    (true, _, _, _, _) => JsonErrorKind::UnexpectedEnd,
    (_, _, _, true, _) => JsonErrorKind::MissingField,
    (_, _, _, _, true) => JsonErrorKind::DuplicateField,
    (_, true, _, _, _) => JsonErrorKind::SyntaxError,
    (_, _, true, _, _) => JsonErrorKind::TypeError,
    _ => JsonErrorKind::Unknown,
  }
}

/// Classification of JSON error types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonErrorKind {
  /// Unexpected end of input.
  UnexpectedEnd,
  /// Syntax error in JSON structure.
  SyntaxError,
  /// Type mismatch (e.g., expected string, found number).
  TypeError,
  /// Required field is missing.
  MissingField,
  /// Duplicate field in object.
  DuplicateField,
  /// Unknown error type.
  Unknown,
}

impl fmt::Display for JsonErrorKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnexpectedEnd => write!(f, "unexpected end of input"),
      Self::SyntaxError => write!(f, "syntax error"),
      Self::TypeError => write!(f, "type mismatch"),
      Self::MissingField => write!(f, "missing field"),
      Self::DuplicateField => write!(f, "duplicate field"),
      Self::Unknown => write!(f, "unknown error"),
    }
  }
}

/// Detailed context for a JSON parsing error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonErrorContext {
  /// The line number where the error occurred (1-indexed).
  pub line_number: usize,
  /// The column number where the error occurred (1-indexed), if available.
  pub column: Option<usize>,
  /// The JSON path to the field where the error occurred.
  pub field_path: Option<String>,
  /// The kind of JSON error.
  pub kind: JsonErrorKind,
  /// The original error message.
  pub original_message: String,
  /// Sanitized snippet of the problematic content.
  pub context_snippet: Option<String>,
}

impl JsonErrorContext {
  /// Create a new error context from a serde_json error.
  ///
  /// # Arguments
  ///
  /// * `error` - The serde_json error
  /// * `line_number` - The line number in the file (1-indexed)
  /// * `raw_content` - The raw content that failed to parse
  ///
  /// # Returns
  ///
  /// A `JsonErrorContext` with detailed error information.
  #[must_use]
  pub fn from_serde_error(
    error: &serde_json::Error,
    line_number: usize,
    raw_content: &str,
  ) -> Self {
    Self {
      line_number,
      column: Some(error.line()),
      field_path: extract_json_path(error),
      kind: classify_json_error(error),
      original_message: error.to_string(),
      context_snippet: Some(sanitize_for_display(raw_content)),
    }
  }

  /// Create an error context for a missing field.
  ///
  /// # Arguments
  ///
  /// * `field_name` - The name of the missing field
  /// * `line_number` - The line number in the file (1-indexed)
  ///
  /// # Returns
  ///
  /// A `JsonErrorContext` for the missing field.
  #[must_use]
  pub fn missing_field(field_name: &str, line_number: usize) -> Self {
    Self {
      line_number,
      column: None,
      field_path: Some(sanitize_field_name(field_name)),
      kind: JsonErrorKind::MissingField,
      original_message: format!("missing required field: {field_name}"),
      context_snippet: None,
    }
  }

  /// Create an error context for a type mismatch.
  ///
  /// # Arguments
  ///
  /// * `field_name` - The name of the field with the wrong type
  /// * `expected` - The expected type
  /// * `actual` - The actual type found
  /// * `line_number` - The line number in the file (1-indexed)
  ///
  /// # Returns
  ///
  /// A `JsonErrorContext` for the type mismatch.
  #[must_use]
  pub fn type_mismatch(field_name: &str, expected: &str, actual: &str, line_number: usize) -> Self {
    Self {
      line_number,
      column: None,
      field_path: Some(sanitize_field_name(field_name)),
      kind: JsonErrorKind::TypeError,
      original_message: format!(
        "type mismatch at '{}': expected {}, found {}",
        field_name, expected, actual
      ),
      context_snippet: None,
    }
  }

  /// Generate an actionable error message.
  ///
  /// # Returns
  ///
  /// A human-readable error message with suggestions for fixing the error.
  #[must_use]
  pub fn to_actionable_message(&self) -> String {
    let location = match (self.column, &self.field_path) {
      (Some(col), Some(path)) => format!(
        "line {}, column {}, at field '{}'",
        self.line_number, col, path
      ),
      (Some(col), None) => format!("line {}, column {}", self.line_number, col),
      (None, Some(path)) => format!("line {}, at field '{}'", self.line_number, path),
      (None, None) => format!("line {}", self.line_number),
    };

    let suggestion = match self.kind {
      JsonErrorKind::UnexpectedEnd => {
        "Check for missing closing braces '}', brackets ']', or quotes '\"'. Ensure the JSON is complete."
      }
      JsonErrorKind::SyntaxError => {
        "Check for invalid characters, missing commas, or malformed JSON structure."
      }
      JsonErrorKind::TypeError => {
        "Check that the field value matches the expected type (string, number, boolean, array, or object)."
      }
      JsonErrorKind::MissingField => {
        "Add the missing required field with an appropriate value."
      }
      JsonErrorKind::DuplicateField => {
        "Remove the duplicate field or rename one of them."
      }
      JsonErrorKind::Unknown => {
        "Review the JSON structure and ensure it matches the expected schema."
      }
    };

    let mut message = format!(
      "JSON parsing error at {}:\n  Error: {}\n  Suggestion: {}",
      location, self.original_message, suggestion
    );

    if let Some(ref snippet) = self.context_snippet {
      message.push_str(&format!("\n  Content: {}", snippet));
    }

    message
  }
}

impl fmt::Display for JsonErrorContext {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "JSON error at line {}: {}",
      self.line_number, self.original_message
    )?;
    if let Some(ref path) = self.field_path {
      write!(f, " (field: {})", path)?;
    }
    Ok(())
  }
}

/// Error type for storage operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StorageError {
  /// I/O error during file operations.
  #[error("I/O error: {0}")]
  IoError(String),

  /// JSON serialization/deserialization error with detailed context.
  #[error("JSON error: {0}")]
  JsonError(String),

  /// Detailed JSON parsing error with field context.
  #[error("{}", context)]
  JsonParseError {
    /// Detailed error context.
    context: JsonErrorContext,
  },

  /// Session not found in storage.
  #[error("session not found: {0}")]
  SessionNotFound(String),

  /// Invalid JSON on a specific line with detailed context.
  #[error("{}", context)]
  InvalidJsonLineDetailed {
    /// Detailed error context.
    context: JsonErrorContext,
  },

  /// Invalid JSON on a specific line (simple variant for backward compatibility).
  #[error("invalid JSON on line {line}: {error}")]
  InvalidJsonLine {
    /// Line number where the error occurred.
    line: usize,
    /// The error message.
    error: String,
  },

  /// Failed to create directory.
  #[error("directory creation failed: {0}")]
  DirectoryCreationFailed(String),

  /// Validation error for session data.
  #[error("validation error: {0}")]
  ValidationError(String),

  /// Multiple JSON errors in a file.
  #[error("multiple JSON errors in file: {count} errors starting at line {first_line}")]
  MultipleJsonErrors {
    /// Total number of errors.
    count: usize,
    /// The line number of the first error.
    first_line: usize,
    /// All error contexts.
    errors: Vec<JsonErrorContext>,
  },
}

impl StorageError {
  /// Create a JSON parse error from a serde_json error.
  ///
  /// # Arguments
  ///
  /// * `error` - The serde_json error
  /// * `line_number` - The line number in the file (1-indexed)
  /// * `raw_content` - The raw content that failed to parse
  ///
  /// # Returns
  ///
  /// A `StorageError::JsonParseError` with detailed context.
  #[must_use]
  pub fn json_parse_error(
    error: &serde_json::Error,
    line_number: usize,
    raw_content: &str,
  ) -> Self {
    Self::JsonParseError {
      context: JsonErrorContext::from_serde_error(error, line_number, raw_content),
    }
  }

  /// Create an invalid JSON line error from a serde_json error.
  ///
  /// # Arguments
  ///
  /// * `error` - The serde_json error
  /// * `line_number` - The line number in the file (1-indexed)
  /// * `raw_content` - The raw content that failed to parse
  ///
  /// # Returns
  ///
  /// A `StorageError::InvalidJsonLineDetailed` with detailed context.
  #[must_use]
  pub fn invalid_json_line(
    error: &serde_json::Error,
    line_number: usize,
    raw_content: &str,
  ) -> Self {
    Self::InvalidJsonLineDetailed {
      context: JsonErrorContext::from_serde_error(error, line_number, raw_content),
    }
  }

  /// Create a validation error for a missing required field.
  ///
  /// # Arguments
  ///
  /// * `field_name` - The name of the missing field
  /// * `line_number` - The line number (1-indexed)
  ///
  /// # Returns
  ///
  /// A `StorageError::ValidationError`.
  #[must_use]
  pub fn missing_field(field_name: &str, line_number: usize) -> Self {
    Self::ValidationError(format!(
      "missing required field '{}' at line {}",
      sanitize_field_name(field_name),
      line_number
    ))
  }

  /// Create a validation error for a type mismatch.
  ///
  /// # Arguments
  ///
  /// * `field_name` - The name of the field with the wrong type
  /// * `expected` - The expected type
  /// * `actual` - The actual type found
  /// * `line_number` - The line number (1-indexed)
  ///
  /// # Returns
  ///
  /// A `StorageError::ValidationError`.
  #[must_use]
  pub fn type_mismatch(field_name: &str, expected: &str, actual: &str, line_number: usize) -> Self {
    Self::ValidationError(format!(
      "type mismatch at '{}' at line {}: expected {}, found {}",
      sanitize_field_name(field_name),
      line_number,
      expected,
      actual
    ))
  }

  /// Check if this error is related to JSON parsing.
  ///
  /// # Returns
  ///
  /// `true` if this is a JSON-related error.
  #[must_use]
  pub const fn is_json_error(&self) -> bool {
    matches!(
      self,
      Self::JsonError(_)
        | Self::JsonParseError { .. }
        | Self::InvalidJsonLine { .. }
        | Self::InvalidJsonLineDetailed { .. }
        | Self::MultipleJsonErrors { .. }
    )
  }

  /// Check if this error is recoverable.
  ///
  /// Recoverable errors might be fixed by modifying the input data.
  ///
  /// # Returns
  ///
  /// `true` if the error might be recoverable.
  #[must_use]
  pub const fn is_recoverable(&self) -> bool {
    matches!(
      self,
      Self::JsonError(_)
        | Self::JsonParseError { .. }
        | Self::InvalidJsonLine { .. }
        | Self::InvalidJsonLineDetailed { .. }
        | Self::ValidationError(_)
    )
  }

  /// Get the line number associated with this error, if any.
  ///
  /// # Returns
  ///
  /// The line number for JSON-related errors, or `None`.
  #[must_use]
  pub fn line_number(&self) -> Option<usize> {
    match self {
      Self::JsonParseError { context } => Some(context.line_number),
      Self::InvalidJsonLine { line, .. } => Some(*line),
      Self::InvalidJsonLineDetailed { context } => Some(context.line_number),
      Self::MultipleJsonErrors { first_line, .. } => Some(*first_line),
      _ => None,
    }
  }

  /// Get an actionable error message.
  ///
  /// # Returns
  ///
  /// A human-readable error message with suggestions.
  #[must_use]
  pub fn to_actionable_message(&self) -> String {
    match self {
      Self::JsonParseError { context } | Self::InvalidJsonLineDetailed { context } => {
        context.to_actionable_message()
      }
      Self::InvalidJsonLine { line, error } => {
        format!(
          "JSON parsing error at line {}:\n  Error: {}\n  Suggestion: Check for syntax errors, missing fields, or type mismatches.",
          line, error
        )
      }
      Self::MultipleJsonErrors {
        count,
        first_line,
        errors,
      } => {
        let first_suggestion = errors
          .first()
          .map_or("Review the JSON structure.", |ctx| &ctx.original_message);
        format!(
          "Multiple JSON errors ({count} total) starting at line {first_line}:\n  First error: {first_suggestion}\n  Suggestion: Fix the first error and re-validate."
        )
      }
      Self::ValidationError(msg) => {
        format!(
          "Validation error: {msg}\n  Suggestion: Ensure all required fields are present and have valid values."
        )
      }
      Self::SessionNotFound(id) => {
        format!(
          "Session not found: '{}'\n  Suggestion: Check that the session ID is correct and the session has been saved.",
          sanitize_field_name(id)
        )
      }
      Self::IoError(msg) => {
        format!("I/O error: {msg}\n  Suggestion: Check file permissions and disk space.")
      }
      Self::JsonError(msg) => {
        format!(
          "JSON error: {msg}\n  Suggestion: Ensure the JSON is well-formed and matches the expected schema."
        )
      }
      Self::DirectoryCreationFailed(msg) => {
        format!(
          "Directory creation failed: {msg}\n  Suggestion: Check parent directory permissions and path validity."
        )
      }
    }
  }
}

/// Validate JSON content before parsing.
///
/// Performs basic validation and returns early errors for common issues.
///
/// # Arguments
///
/// * `content` - The JSON content to validate
///
/// # Returns
///
/// `Ok(())` if basic validation passes, or an error describing the issue.
pub fn validate_json_basic(content: &str) -> Result<(), StorageError> {
  let trimmed = content.trim();

  // Check for empty content
  if trimmed.is_empty() {
    return Err(StorageError::JsonError("empty JSON content".to_string()));
  }

  // Check for unbalanced braces/brackets (basic check)
  let mut brace_count = 0i32;
  let mut bracket_count = 0i32;
  let mut in_string = false;
  let mut escape_next = false;

  for (idx, ch) in trimmed.chars().enumerate() {
    if escape_next {
      escape_next = false;
      continue;
    }

    match ch {
      '\\' if in_string => escape_next = true,
      '"' => in_string = !in_string,
      '{' if !in_string => brace_count += 1,
      '}' if !in_string => {
        brace_count -= 1;
        if brace_count < 0 {
          return Err(StorageError::JsonError(format!(
            "unbalanced '}}' at position {}",
            idx
          )));
        }
      }
      '[' if !in_string => bracket_count += 1,
      ']' if !in_string => {
        bracket_count -= 1;
        if bracket_count < 0 {
          return Err(StorageError::JsonError(format!(
            "unbalanced ']' at position {}",
            idx
          )));
        }
      }
      _ => {}
    }
  }

  if brace_count != 0 {
    return Err(StorageError::JsonError(format!(
      "unbalanced braces: {} unmatched '{{' or '}}'",
      brace_count.abs()
    )));
  }

  if bracket_count != 0 {
    return Err(StorageError::JsonError(format!(
      "unbalanced brackets: {} unmatched '[' or ']'",
      bracket_count.abs()
    )));
  }

  Ok(())
}

/// Parse JSON with enhanced error reporting.
///
/// # Type Parameters
///
/// * `T` - The type to deserialize into
///
/// # Arguments
///
/// * `content` - The JSON content to parse
/// * `line_number` - The line number for error reporting (1-indexed)
///
/// # Returns
///
/// The parsed value or a detailed error.
pub fn parse_json_with_context<T: serde::de::DeserializeOwned>(
  content: &str,
  line_number: usize,
) -> Result<T, StorageError> {
  // First, do basic validation
  validate_json_basic(content)?;

  // Then parse with serde
  serde_json::from_str(content)
    .map_err(|e| StorageError::json_parse_error(&e, line_number, content))
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::expect_used)]

  use super::*;

  #[test]
  fn sanitize_for_display_truncates_long_strings() {
    let long_input = "x".repeat(300);
    let sanitized = sanitize_for_display(&long_input);
    assert!(sanitized.len() < long_input.len());
    assert!(sanitized.contains("truncated"));
  }

  #[test]
  fn sanitize_for_display_removes_control_chars() {
    let input = "hello\x00world\x1F";
    let sanitized = sanitize_for_display(input);
    assert_eq!(sanitized, "helloworld");
  }

  #[test]
  fn sanitize_for_display_preserves_newlines() {
    let input = "hello\nworld";
    let sanitized = sanitize_for_display(input);
    assert_eq!(sanitized, "hello\nworld");
  }

  #[test]
  fn sanitize_field_name_removes_special_chars() {
    let input = "field<script>alert('xss')</script>";
    let sanitized = sanitize_field_name(input);
    assert!(!sanitized.contains('<'));
    assert!(!sanitized.contains('>'));
  }

  #[test]
  fn sanitize_field_name_truncates_long_names() {
    let long_name = "x".repeat(150);
    let sanitized = sanitize_field_name(&long_name);
    assert!(sanitized.len() < long_name.len());
  }

  #[test]
  fn classify_json_error_identifies_eof() {
    let result: Result<serde_json::Value, _> = serde_json::from_str("{");
    if let Err(e) = result {
      assert_eq!(classify_json_error(&e), JsonErrorKind::UnexpectedEnd);
    } else {
      panic!("Expected error");
    }
  }

  #[test]
  fn classify_json_error_identifies_syntax_error() {
    let result: Result<serde_json::Value, _> = serde_json::from_str("{}}");
    if let Err(e) = result {
      assert_eq!(classify_json_error(&e), JsonErrorKind::SyntaxError);
    } else {
      panic!("Expected error");
    }
  }

  #[test]
  fn classify_json_error_identifies_type_error() {
    #[derive(serde::Deserialize)]
    struct Test {
      value: i32,
    }
    let result: Result<Test, _> = serde_json::from_str(r#"{"value": "not a number"}"#);
    if let Err(e) = result {
      // Type errors contain "expected" in the message but classify as SyntaxError
      // since they're about invalid type structure
      let kind = classify_json_error(&e);
      assert!(matches!(
        kind,
        JsonErrorKind::SyntaxError | JsonErrorKind::TypeError
      ));
    } else {
      panic!("Expected error");
    }
  }

  #[test]
  fn json_error_context_to_actionable_message_includes_suggestion() {
    let ctx = JsonErrorContext {
      line_number: 5,
      column: Some(10),
      field_path: Some("answers[0].question_id".to_string()),
      kind: JsonErrorKind::MissingField,
      original_message: "missing field".to_string(),
      context_snippet: Some("test content".to_string()),
    };

    let msg = ctx.to_actionable_message();
    assert!(msg.contains("line 5"));
    assert!(msg.contains("column 10"));
    assert!(msg.contains("answers[0].question_id"));
    assert!(msg.contains("Suggestion:"));
  }

  #[test]
  fn storage_error_json_parse_error_creates_context() {
    let result: Result<serde_json::Value, _> = serde_json::from_str("{invalid}");
    if let Err(e) = result {
      let error = StorageError::json_parse_error(&e, 1, "{invalid}");
      assert!(matches!(error, StorageError::JsonParseError { .. }));
      assert!(error.is_json_error());
      assert!(error.line_number().is_some());
    } else {
      panic!("Expected error");
    }
  }

  #[test]
  fn storage_error_is_recoverable_for_json_errors() {
    let error = StorageError::JsonError("test".to_string());
    assert!(error.is_recoverable());

    let error = StorageError::SessionNotFound("test".to_string());
    assert!(!error.is_recoverable());
  }

  #[test]
  fn validate_json_basic_detects_unbalanced_braces() {
    let result = validate_json_basic("{}}");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("unbalanced"));
  }

  #[test]
  fn validate_json_basic_detects_unbalanced_brackets() {
    let result = validate_json_basic("[]]");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("unbalanced"));
  }

  #[test]
  fn validate_json_basic_accepts_valid_json() {
    let result = validate_json_basic(r#"{"key": "value"}"#);
    assert!(result.is_ok());
  }

  #[test]
  fn validate_json_basic_rejects_empty_content() {
    let result = validate_json_basic("");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("empty"));
  }

  #[test]
  fn parse_json_with_context_returns_parsed_value() {
    let result: Result<serde_json::Value, _> = parse_json_with_context(r#"{"key": "value"}"#, 1);
    assert!(result.is_ok());
  }

  #[test]
  fn parse_json_with_context_returns_detailed_error() {
    let result: Result<serde_json::Value, _> = parse_json_with_context("{invalid}", 1);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, StorageError::JsonParseError { .. }));
    assert!(error.line_number().is_some());
  }

  #[test]
  fn storage_error_to_actionable_message_includes_suggestion() {
    let error = StorageError::SessionNotFound("test_id".to_string());
    let msg = error.to_actionable_message();
    assert!(msg.contains("test_id"));
    assert!(msg.contains("Suggestion:"));
  }

  #[test]
  fn storage_error_missing_field_creates_validation_error() {
    let error = StorageError::missing_field("question_id", 5);
    assert!(matches!(error, StorageError::ValidationError(_)));
    let msg = error.to_string();
    assert!(msg.contains("question_id"));
    assert!(msg.contains("line 5"));
  }

  #[test]
  fn storage_error_type_mismatch_creates_validation_error() {
    let error = StorageError::type_mismatch("confidence", "number", "string", 10);
    assert!(matches!(error, StorageError::ValidationError(_)));
    let msg = error.to_string();
    assert!(msg.contains("confidence"));
    assert!(msg.contains("number"));
    assert!(msg.contains("string"));
  }

  #[test]
  fn multiple_json_errors_display() {
    let ctx1 = JsonErrorContext {
      line_number: 5,
      column: None,
      field_path: None,
      kind: JsonErrorKind::SyntaxError,
      original_message: "error 1".to_string(),
      context_snippet: None,
    };
    let ctx2 = JsonErrorContext {
      line_number: 10,
      column: None,
      field_path: None,
      kind: JsonErrorKind::TypeError,
      original_message: "error 2".to_string(),
      context_snippet: None,
    };
    let error = StorageError::MultipleJsonErrors {
      count: 2,
      first_line: 5,
      errors: vec![ctx1, ctx2],
    };

    let msg = error.to_string();
    assert!(msg.contains("2 errors"));
    assert!(msg.contains("line 5"));
  }

  #[test]
  fn json_error_kind_display() {
    assert_eq!(
      JsonErrorKind::UnexpectedEnd.to_string(),
      "unexpected end of input"
    );
    assert_eq!(JsonErrorKind::SyntaxError.to_string(), "syntax error");
    assert_eq!(JsonErrorKind::TypeError.to_string(), "type mismatch");
    assert_eq!(JsonErrorKind::MissingField.to_string(), "missing field");
    assert_eq!(JsonErrorKind::DuplicateField.to_string(), "duplicate field");
    assert_eq!(JsonErrorKind::Unknown.to_string(), "unknown error");
  }

  #[test]
  fn test_storage_error_display() {
    assert_eq!(
      StorageError::IoError("file missing".to_string()).to_string(),
      "I/O error: file missing"
    );

    assert_eq!(
      StorageError::InvalidJsonLine {
        line: 2,
        error: "bad json".to_string(),
      }
      .to_string(),
      "invalid JSON on line 2: bad json"
    );
  }
}
