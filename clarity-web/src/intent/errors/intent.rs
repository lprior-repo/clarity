#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Intent domain errors following Scott Wlaschin's DDD principles:
/// - Specific variants with structured data
/// - No opaque String variants that lose type information
/// - Helper constructors for common cases
#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentError {
  /// JSON parsing or serialization failed
  #[error("JSON parse error at {location}: {reason}")]
  JsonParse {
    /// Where the error occurred
    location: String,
    /// The specific parsing error
    reason: JsonParseReason,
  },

  /// Required field is missing from a structure
  #[error("missing required field '{field}' in {context}")]
  MissingField {
    /// The field that is missing
    field: String,
    /// The context (struct, object) where it was expected
    context: String,
  },

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

  /// Field has invalid value
  #[error("invalid value for field '{field}': {reason}")]
  InvalidValue {
    /// Field name
    field: String,
    /// Why the value is invalid
    reason: InvalidValueReason,
  },

  /// Unknown field encountered
  #[error("unknown field '{field}' in {context}")]
  UnknownField {
    /// The unknown field name
    field: String,
    /// The context where it was found
    context: String,
  },

  /// Validation failed for a structure
  #[error("validation failed for {context}: {reason}")]
  ValidationFailed {
    /// What was being validated
    context: String,
    /// The validation failure details
    reason: ValidationFailureReason,
  },

  /// I/O operation failed
  #[error("I/O error during {operation}: {reason}")]
  Io {
    /// The I/O operation that failed
    operation: IoOperationType,
    /// The underlying error
    reason: String,
  },

  /// File not found at specified path
  #[error("file not found: {path}")]
  FileNotFound {
    /// The path that was searched
    path: String,
  },

  /// Path is invalid
  #[error("invalid path '{path}': {reason}")]
  InvalidPath {
    /// The invalid path
    path: String,
    /// Why the path is invalid
    reason: PathInvalidReason,
  },

  /// Circular dependency detected
  #[error("circular dependency detected: {chain}")]
  CircularDependency {
    /// The dependency chain that forms a cycle
    chain: DependencyChain,
  },

  /// Constraint violated
  #[error("constraint violation in {context}: {constraint}")]
  ConstraintViolation {
    /// Where the constraint was violated
    context: String,
    /// The constraint that was violated
    constraint: ConstraintType,
  },

  /// Configuration error
  #[error("configuration error: {reason}")]
  Configuration {
    /// The configuration error details
    reason: ConfigurationErrorReason,
  },

  /// Internal error (should not happen in normal operation)
  #[error("internal error: {details}")]
  Internal {
    /// Internal error details
    details: InternalErrorDetails,
  },
}

/// Specific reasons for JSON parsing failures
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JsonParseReason {
  /// Syntax error in JSON
  SyntaxError { message: String },
  /// Type mismatch during parsing
  TypeMismatch { expected: String, actual: String },
  /// Unexpected end of input
  UnexpectedEof,
  /// Invalid unicode sequence
  InvalidUnicode { sequence: String },
  /// Trailing characters after valid JSON
  TrailingCharacters { count: usize },
}

impl std::fmt::Display for JsonParseReason {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::SyntaxError { message } => write!(f, "syntax error: {message}"),
      Self::TypeMismatch { expected, actual } => {
        write!(f, "type mismatch: expected {expected}, got {actual}")
      }
      Self::UnexpectedEof => write!(f, "unexpected end of input"),
      Self::InvalidUnicode { sequence } => write!(f, "invalid unicode: {sequence}"),
      Self::TrailingCharacters { count } => write!(f, "{count} trailing characters"),
    }
  }
}

/// Reasons why a value is invalid
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvalidValueReason {
  /// Value is empty when it should not be
  Empty,
  /// Value is out of allowed range
  OutOfRange { min: String, max: String, actual: String },
  /// Value format is incorrect
  InvalidFormat { expected: String },
  /// Value does not match required pattern
  PatternMismatch { pattern: String },
  /// Custom validation failure
  Custom { message: String },
}

impl std::fmt::Display for InvalidValueReason {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Empty => write!(f, "value is empty"),
      Self::OutOfRange { min, max, actual } => {
        write!(f, "value {actual} out of range [{min}, {max}]")
      }
      Self::InvalidFormat { expected } => write!(f, "invalid format, expected {expected}"),
      Self::PatternMismatch { pattern } => write!(f, "does not match pattern: {pattern}"),
      Self::Custom { message } => write!(f, "{message}"),
    }
  }
}

/// Reasons why validation failed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationFailureReason {
  /// Required field is missing
  MissingRequired { fields: Vec<String> },
  /// Business rule violated
  BusinessRule { rule: String, message: String },
  /// Invariant violated
  InvariantViolation { invariant: String },
  /// Cross-field validation failed
  CrossFieldValidation { fields: Vec<String>, message: String },
}

impl std::fmt::Display for ValidationFailureReason {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::MissingRequired { fields } => {
        write!(f, "missing required fields: {}", fields.join(", "))
      }
      Self::BusinessRule { rule, message } => write!(f, "rule '{rule}': {message}"),
      Self::InvariantViolation { invariant } => write!(f, "invariant violated: {invariant}"),
      Self::CrossFieldValidation { fields, message } => {
        write!(f, "fields [{}] validation failed: {message}", fields.join(", "))
      }
    }
  }
}

/// Types of I/O operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IoOperationType {
  /// Reading from a file
  Read,
  /// Writing to a file
  Write,
  /// Deleting a file
  Delete,
  /// Creating a directory
  CreateDir,
  /// Listing directory contents
  ListDir,
}

impl std::fmt::Display for IoOperationType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Read => write!(f, "read"),
      Self::Write => write!(f, "write"),
      Self::Delete => write!(f, "delete"),
      Self::CreateDir => write!(f, "create directory"),
      Self::ListDir => write!(f, "list directory"),
    }
  }
}

/// Reasons why a path is invalid
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathInvalidReason {
  /// Path contains invalid characters
  InvalidCharacters { chars: Vec<char> },
  /// Path is not absolute when it should be
  NotAbsolute,
  /// Path is too long
  TooLong { length: usize, max: usize },
  /// Path contains forbidden components
  ForbiddenComponent { component: String },
  /// Path traversal attempt detected
  TraversalAttempt,
}

impl std::fmt::Display for PathInvalidReason {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::InvalidCharacters { chars } => {
        write!(f, "invalid characters: {}", chars.iter().collect::<String>())
      }
      Self::NotAbsolute => write!(f, "path must be absolute"),
      Self::TooLong { length, max } => write!(f, "path too long ({length} > {max})"),
      Self::ForbiddenComponent { component } => write!(f, "forbidden component: {component}"),
      Self::TraversalAttempt => write!(f, "path traversal attempt detected"),
    }
  }
}

/// A chain of dependencies forming a cycle
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyChain {
  /// The items in the dependency chain
  pub items: Vec<String>,
}

impl std::fmt::Display for DependencyChain {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} -> {}", self.items.join(" -> "), self.items.first().map_or("", |s| s))
  }
}

impl DependencyChain {
  /// Create a new dependency chain
  #[must_use]
  pub fn new(items: Vec<String>) -> Self {
    Self { items }
  }

  /// Create a dependency chain from two items forming a cycle
  #[must_use]
  pub fn cycle(from: String, to: String) -> Self {
    Self {
      items: vec![from, to],
    }
  }
}

/// Types of constraints that can be violated
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintType {
  /// Uniqueness constraint
  Uniqueness { field: String, value: String },
  /// Referential integrity constraint
  ReferentialIntegrity { from: String, to: String },
  /// Cardinality constraint
  Cardinality { min: usize, max: usize, actual: usize },
  /// Custom constraint
  Custom { name: String, description: String },
}

impl std::fmt::Display for ConstraintType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Uniqueness { field, value } => write!(f, "duplicate {field}={value}"),
      Self::ReferentialIntegrity { from, to } => write!(f, "broken reference: {from} -> {to}"),
      Self::Cardinality { min, max, actual } => {
        write!(f, "cardinality [{min}, {max}] violated with {actual} items")
      }
      Self::Custom { name, description } => write!(f, "{name}: {description}"),
    }
  }
}

/// Configuration error reasons
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigurationErrorReason {
  /// Required configuration is missing
  MissingConfig { key: String },
  /// Configuration value is invalid
  InvalidValue { key: String, value: String, expected: String },
  /// Configuration file could not be loaded
  LoadFailed { path: String, error: String },
  /// Environment variable error
  EnvVar { var: String, error: String },
}

impl std::fmt::Display for ConfigurationErrorReason {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::MissingConfig { key } => write!(f, "missing configuration: {key}"),
      Self::InvalidValue { key, value, expected } => {
        write!(f, "invalid {key}={value}, expected {expected}")
      }
      Self::LoadFailed { path, error } => write!(f, "failed to load {path}: {error}"),
      Self::EnvVar { var, error } => write!(f, "environment variable {var}: {error}"),
    }
  }
}

/// Internal error details
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InternalErrorDetails {
  /// An assertion failed
  AssertionFailed { what: String },
  /// An invariant was violated
  InvariantViolated { invariant: String },
  /// An unexpected state was encountered
  UnexpectedState { state: String },
  /// A generic internal error
  Generic { message: String },
}

impl std::fmt::Display for InternalErrorDetails {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::AssertionFailed { what } => write!(f, "assertion failed: {what}"),
      Self::InvariantViolated { invariant } => write!(f, "invariant violated: {invariant}"),
      Self::UnexpectedState { state } => write!(f, "unexpected state: {state}"),
      Self::Generic { message } => write!(f, "{message}"),
    }
  }
}

// ============================================================================
// Helper constructors for common error cases
// ============================================================================

impl IntentError {
  /// Create a JSON syntax error
  #[must_use]
  pub fn json_syntax(location: impl Into<String>, message: impl Into<String>) -> Self {
    Self::JsonParse {
      location: location.into(),
      reason: JsonParseReason::SyntaxError {
        message: message.into(),
      },
    }
  }

  /// Create a missing field error
  #[must_use]
  pub fn missing_field(field: impl Into<String>, context: impl Into<String>) -> Self {
    Self::MissingField {
      field: field.into(),
      context: context.into(),
    }
  }

  /// Create a file not found error
  #[must_use]
  pub fn file_not_found(path: impl Into<String>) -> Self {
    Self::FileNotFound { path: path.into() }
  }

  /// Create a circular dependency error with a simple cycle
  #[must_use]
  pub fn circular_dependency_simple(from: impl Into<String>, to: impl Into<String>) -> Self {
    Self::CircularDependency {
      chain: DependencyChain::cycle(from.into(), to.into()),
    }
  }

  /// Create a uniqueness constraint violation error
  #[must_use]
  pub fn uniqueness_violation(
    context: impl Into<String>,
    field: impl Into<String>,
    value: impl Into<String>,
  ) -> Self {
    Self::ConstraintViolation {
      context: context.into(),
      constraint: ConstraintType::Uniqueness {
        field: field.into(),
        value: value.into(),
      },
    }
  }

  /// Create a generic internal error
  #[must_use]
  pub fn internal(message: impl Into<String>) -> Self {
    Self::Internal {
      details: InternalErrorDetails::Generic {
        message: message.into(),
      },
    }
  }

  /// Create a validation failed error with missing fields
  #[must_use]
  pub fn validation_missing_fields(context: impl Into<String>, fields: Vec<String>) -> Self {
    Self::ValidationFailed {
      context: context.into(),
      reason: ValidationFailureReason::MissingRequired { fields },
    }
  }

  /// Create an I/O read error
  #[must_use]
  pub fn io_read(reason: impl Into<String>) -> Self {
    Self::Io {
      operation: IoOperationType::Read,
      reason: reason.into(),
    }
  }

  /// Create an I/O write error
  #[must_use]
  pub fn io_write(reason: impl Into<String>) -> Self {
    Self::Io {
      operation: IoOperationType::Write,
      reason: reason.into(),
    }
  }

  /// Create an invalid value error for empty value
  #[must_use]
  pub fn empty_value(field: impl Into<String>) -> Self {
    Self::InvalidValue {
      field: field.into(),
      reason: InvalidValueReason::Empty,
    }
  }

  /// Create an invalid value error for out of range
  #[must_use]
  pub fn out_of_range(
    field: impl Into<String>,
    min: impl Into<String>,
    max: impl Into<String>,
    actual: impl Into<String>,
  ) -> Self {
    Self::InvalidValue {
      field: field.into(),
      reason: InvalidValueReason::OutOfRange {
        min: min.into(),
        max: max.into(),
        actual: actual.into(),
      },
    }
  }
}
