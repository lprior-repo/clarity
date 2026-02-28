#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::intent::parser::ParseError;
use crate::intent::security::SecurityError;
use thiserror::Error;

/// Loader errors following Scott Wlaschin's DDD principles:
/// - Specific variants with structured data
/// - No opaque String variants that lose type information
/// - Helper constructors for common cases
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LoaderError {
  /// I/O operation failed with context about what was attempted
  #[error("I/O error: {operation} failed for '{path}': {reason}")]
  Io {
    /// The operation that failed (read, write, metadata, etc.)
    operation: IoOperation,
    /// The path involved in the operation
    path: String,
    /// The underlying reason for failure
    reason: String,
  },

  /// JSON parsing or serialization failed
  #[error("JSON error at {location}: {reason}")]
  Json {
    /// Where the error occurred (file path, field name, etc.)
    location: String,
    /// The specific JSON parsing error
    reason: JsonErrorReason,
  },

  /// External command execution failed
  #[error("command '{command}' failed: {reason}")]
  CommandFailed {
    /// The command that was executed
    command: String,
    /// Why the command failed
    reason: CommandFailureReason,
  },

  /// Validation of data failed
  #[error("validation failed for {context}: {reason}")]
  Validation {
    /// What was being validated (field, object, etc.)
    context: String,
    /// The specific validation failure
    reason: ValidationErrorReason,
  },

  /// Session could not be found
  #[error("session not found: {session_id}")]
  SessionNotFound {
    /// The session ID that was looked up
    session_id: String,
  },

  /// Spec field has incorrect type
  #[error("invalid spec for field '{field}': expected {expected}, got {actual}")]
  InvalidSpec {
    /// Field name
    field: String,
    /// Expected type
    expected: String,
    /// Actual type found
    actual: String,
  },

  /// Required field is empty
  #[error("empty required field: {field}")]
  EmptyField {
    /// The field that was empty
    field: String,
  },

  /// CUE tool output was invalid
  #[error("invalid CUE output: {reason}")]
  InvalidCueOutput {
    /// Why the output was invalid
    reason: CueOutputError,
  },

  /// CUE binary not found or not working
  #[error("CUE binary not found: {details}")]
  CueBinaryNotFound {
    /// Details about what was tried and what's needed
    details: CueBinaryError,
  },

  /// Security violation detected
  #[error("security error: {violation}")]
  Security {
    /// The security violation that was detected
    violation: SecurityViolation,
  },

  /// File not found at the specified path
  #[error("file not found: {path}")]
  FileNotFound {
    /// The path that was searched
    path: String,
  },
}

/// Types of I/O operations that can fail
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IoOperation {
  /// Reading file contents
  Read,
  /// Writing to a file
  Write,
  /// Getting file metadata
  Metadata,
  /// Checking file existence
  Access,
  /// Creating a directory
  CreateDir,
  /// Removing a file or directory
  Remove,
}

impl std::fmt::Display for IoOperation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Read => write!(f, "read"),
      Self::Write => write!(f, "write"),
      Self::Metadata => write!(f, "metadata"),
      Self::Access => write!(f, "access"),
      Self::CreateDir => write!(f, "create directory"),
      Self::Remove => write!(f, "remove"),
    }
  }
}

/// Specific reasons for JSON errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonErrorReason {
  /// Syntax error in JSON
  SyntaxError { message: String },
  /// Type mismatch during deserialization
  TypeMismatch { expected: String, actual: String },
  /// Missing required field
  MissingField { field: String },
  /// Invalid UTF-8 encoding
  InvalidUtf8,
  /// Generic parsing error
  ParseError { details: String },
}

impl std::fmt::Display for JsonErrorReason {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::SyntaxError { message } => write!(f, "syntax error: {message}"),
      Self::TypeMismatch { expected, actual } => {
        write!(f, "type mismatch: expected {expected}, got {actual}")
      }
      Self::MissingField { field } => write!(f, "missing field: {field}"),
      Self::InvalidUtf8 => write!(f, "invalid UTF-8 encoding"),
      Self::ParseError { details } => write!(f, "{details}"),
    }
  }
}

/// Reasons why a command failed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandFailureReason {
  /// Command could not be spawned
  SpawnFailed { error: String },
  /// Command exited with non-zero status
  ExitCode { code: i32, stderr: String },
  /// Command timed out
  Timeout { seconds: u64 },
  /// Output could not be captured
  OutputCaptureFailed { error: String },
}

impl std::fmt::Display for CommandFailureReason {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::SpawnFailed { error } => write!(f, "failed to spawn: {error}"),
      Self::ExitCode { code, stderr } => {
        if stderr.is_empty() {
          write!(f, "exit code {code}")
        } else {
          write!(f, "exit code {code}: {stderr}")
        }
      }
      Self::Timeout { seconds } => write!(f, "timed out after {seconds}s"),
      Self::OutputCaptureFailed { error } => write!(f, "output capture failed: {error}"),
    }
  }
}

/// Specific validation failure reasons
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationErrorReason {
  /// Required field is missing
  MissingRequired { field: String },
  /// Field value is out of allowed range
  OutOfRange {
    field: String,
    min: String,
    max: String,
    actual: String,
  },
  /// Field format is invalid
  InvalidFormat {
    field: String,
    expected_format: String,
  },
  /// Value does not match constraint
  ConstraintViolation { constraint: String },
  /// Custom validation rule failed
  CustomRule { rule: String, message: String },
}

impl std::fmt::Display for ValidationErrorReason {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::MissingRequired { field } => write!(f, "missing required field: {field}"),
      Self::OutOfRange {
        field,
        min,
        max,
        actual,
      } => {
        write!(f, "{field} value {actual} out of range [{min}, {max}]")
      }
      Self::InvalidFormat {
        field,
        expected_format,
      } => {
        write!(f, "{field} has invalid format, expected {expected_format}")
      }
      Self::ConstraintViolation { constraint } => write!(f, "constraint violated: {constraint}"),
      Self::CustomRule { rule, message } => write!(f, "rule '{rule}' failed: {message}"),
    }
  }
}

/// CUE output parsing errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CueOutputError {
  /// Output was not valid UTF-8
  InvalidUtf8 { error: String },
  /// Output was not valid JSON
  InvalidJson { error: String },
  /// Expected field missing from output
  MissingField { field: String },
  /// Output was empty
  EmptyOutput,
}

impl std::fmt::Display for CueOutputError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::InvalidUtf8 { error } => write!(f, "invalid UTF-8: {error}"),
      Self::InvalidJson { error } => write!(f, "invalid JSON: {error}"),
      Self::MissingField { field } => write!(f, "missing field: {field}"),
      Self::EmptyOutput => write!(f, "output was empty"),
    }
  }
}

/// CUE binary errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CueBinaryError {
  /// Binary not found in PATH
  NotInPath,
  /// Binary found but returned error
  ExecutionError { message: String },
  /// Binary version incompatible
  VersionMismatch { required: String, actual: String },
}

impl std::fmt::Display for CueBinaryError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::NotInPath => write!(
        f,
        "cue command not found in PATH. Install from https://cuelang.org/docs/install/"
      ),
      Self::ExecutionError { message } => write!(f, "{message}"),
      Self::VersionMismatch { required, actual } => {
        write!(f, "version mismatch: required {required}, got {actual}")
      }
    }
  }
}

/// Security violations that can be detected
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityViolation {
  /// Path traversal attempt detected
  PathTraversal { details: String },
  /// Encoded path traversal attempt
  EncodedPathTraversal { encoding_type: String },
  /// Shell metacharacter detected
  ShellMetacharacter { category: String, character: char },
  /// `ReDoS` vulnerability detected
  ReDoSVulnerability { pattern: String },
  /// Session ID validation failed
  SessionIdValidation { error: String },
  /// Null byte detected
  NullByte,
  /// Backslash in path
  BackslashInPath,
  /// Empty input where not allowed
  EmptyInput,
}

impl std::fmt::Display for SecurityViolation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::PathTraversal { details } => write!(f, "path traversal: {details}"),
      Self::EncodedPathTraversal { encoding_type } => {
        write!(f, "encoded path traversal: {encoding_type}")
      }
      Self::ShellMetacharacter {
        category,
        character,
      } => {
        write!(f, "shell metacharacter '{character}' ({category})")
      }
      Self::ReDoSVulnerability { pattern } => write!(f, "ReDoS vulnerability: {pattern}"),
      Self::SessionIdValidation { error } => write!(f, "session ID validation: {error}"),
      Self::NullByte => write!(f, "null byte detected"),
      Self::BackslashInPath => write!(f, "backslash in path"),
      Self::EmptyInput => write!(f, "empty input"),
    }
  }
}

// ============================================================================
// Helper constructors for common error cases
// ============================================================================

impl LoaderError {
  /// Create an I/O error for a file read failure
  #[must_use]
  pub fn io_read(path: impl Into<String>, reason: impl Into<String>) -> Self {
    Self::Io {
      operation: IoOperation::Read,
      path: path.into(),
      reason: reason.into(),
    }
  }

  /// Create an I/O error for a file write failure
  #[must_use]
  pub fn io_write(path: impl Into<String>, reason: impl Into<String>) -> Self {
    Self::Io {
      operation: IoOperation::Write,
      path: path.into(),
      reason: reason.into(),
    }
  }

  /// Create an I/O error for a metadata operation failure
  #[must_use]
  pub fn io_metadata(path: impl Into<String>, reason: impl Into<String>) -> Self {
    Self::Io {
      operation: IoOperation::Metadata,
      path: path.into(),
      reason: reason.into(),
    }
  }

  /// Create an I/O error for an access check failure
  #[must_use]
  pub fn io_access(path: impl Into<String>, reason: impl Into<String>) -> Self {
    Self::Io {
      operation: IoOperation::Access,
      path: path.into(),
      reason: reason.into(),
    }
  }

  /// Create a JSON syntax error
  #[must_use]
  pub fn json_syntax(location: impl Into<String>, message: impl Into<String>) -> Self {
    Self::Json {
      location: location.into(),
      reason: JsonErrorReason::SyntaxError {
        message: message.into(),
      },
    }
  }

  /// Create a JSON type mismatch error
  #[must_use]
  pub fn json_type_mismatch(
    location: impl Into<String>,
    expected: impl Into<String>,
    actual: impl Into<String>,
  ) -> Self {
    Self::Json {
      location: location.into(),
      reason: JsonErrorReason::TypeMismatch {
        expected: expected.into(),
        actual: actual.into(),
      },
    }
  }

  /// Create a JSON missing field error
  #[must_use]
  pub fn json_missing_field(location: impl Into<String>, field: impl Into<String>) -> Self {
    Self::Json {
      location: location.into(),
      reason: JsonErrorReason::MissingField {
        field: field.into(),
      },
    }
  }

  /// Create a command spawn failure error
  #[must_use]
  pub fn command_spawn_failed(command: impl Into<String>, error: impl Into<String>) -> Self {
    Self::CommandFailed {
      command: command.into(),
      reason: CommandFailureReason::SpawnFailed {
        error: error.into(),
      },
    }
  }

  /// Create a command exit code error
  #[must_use]
  pub fn command_exit_code(
    command: impl Into<String>,
    code: i32,
    stderr: impl Into<String>,
  ) -> Self {
    Self::CommandFailed {
      command: command.into(),
      reason: CommandFailureReason::ExitCode {
        code,
        stderr: stderr.into(),
      },
    }
  }

  /// Create a validation missing required field error
  #[must_use]
  pub fn validation_missing_field(context: impl Into<String>, field: impl Into<String>) -> Self {
    Self::Validation {
      context: context.into(),
      reason: ValidationErrorReason::MissingRequired {
        field: field.into(),
      },
    }
  }

  /// Create a validation constraint violation error
  #[must_use]
  pub fn validation_constraint(context: impl Into<String>, constraint: impl Into<String>) -> Self {
    Self::Validation {
      context: context.into(),
      reason: ValidationErrorReason::ConstraintViolation {
        constraint: constraint.into(),
      },
    }
  }

  /// Create a file not found error
  #[must_use]
  pub fn file_not_found(path: impl Into<String>) -> Self {
    Self::FileNotFound { path: path.into() }
  }

  /// Create a session not found error
  #[must_use]
  pub fn session_not_found(session_id: impl Into<String>) -> Self {
    Self::SessionNotFound {
      session_id: session_id.into(),
    }
  }

  /// Create an empty field error
  #[must_use]
  pub fn empty_field(field: impl Into<String>) -> Self {
    Self::EmptyField {
      field: field.into(),
    }
  }
}

impl From<ParseError> for LoaderError {
  fn from(err: ParseError) -> Self {
    match err {
      ParseError::JsonError(msg) => Self::Json {
        location: "parse".into(),
        reason: JsonErrorReason::ParseError { details: msg },
      },
      ParseError::MissingField(field) => Self::validation_missing_field("spec", field),
      ParseError::InvalidType {
        field,
        expected,
        actual,
      } => Self::InvalidSpec {
        field,
        expected,
        actual,
      },
      ParseError::EmptyField(field) => Self::empty_field(field),
    }
  }
}

impl From<SecurityError> for LoaderError {
  fn from(err: SecurityError) -> Self {
    use crate::intent::security::{
      MetacharCategory, PathEncodingType, RegexVulnerability, SessionIdError,
    };

    let violation = match err {
      SecurityError::PathTraversal { details } => SecurityViolation::PathTraversal { details },
      SecurityError::EncodedPathTraversal { encoding_type } => {
        let encoding_str = match encoding_type {
          PathEncodingType::SingleEncoded => "single URL encoding",
          PathEncodingType::DoubleEncoded => "double URL encoding",
          PathEncodingType::MixedEncoding => "mixed encoding",
        };
        SecurityViolation::EncodedPathTraversal {
          encoding_type: encoding_str.to_string(),
        }
      }
      SecurityError::ShellMetacharacter { category, ch } => {
        let category_str = match category {
          MetacharCategory::CommandSeparator => "command separator",
          MetacharCategory::VariableExpansion => "variable expansion",
          MetacharCategory::Grouping => "grouping",
          MetacharCategory::Redirection => "redirection",
          MetacharCategory::EscapeQuote => "escape or quote",
          MetacharCategory::ControlCharacter => "control character",
        };
        SecurityViolation::ShellMetacharacter {
          category: category_str.to_string(),
          character: ch,
        }
      }
      SecurityError::ReDoSVulnerability { vulnerability } => {
        let pattern_str = match vulnerability {
          RegexVulnerability::NestedQuantifiers => "nested quantifiers",
          RegexVulnerability::OverlappingWildcards => "overlapping wildcards",
          RegexVulnerability::AlternationOverlap => "alternation overlap",
          RegexVulnerability::ExponentialBacktracking => "exponential backtracking",
        };
        SecurityViolation::ReDoSVulnerability {
          pattern: pattern_str.to_string(),
        }
      }
      SecurityError::SessionIdValidation { error } => {
        let error_str = match error {
          SessionIdError::TooLong { max } => format!("exceeds max length of {max}"),
          SessionIdError::InvalidCharacter { ch } => format!("invalid character: '{ch}'"),
          SessionIdError::Empty => "session ID is empty".to_string(),
        };
        SecurityViolation::SessionIdValidation { error: error_str }
      }
      SecurityError::NullByteDetected => SecurityViolation::NullByte,
      SecurityError::BackslashInPath => SecurityViolation::BackslashInPath,
      SecurityError::EmptyInput => SecurityViolation::EmptyInput,
    };
    Self::Security { violation }
  }
}

/// Format a loader error for human-readable display
#[must_use]
pub fn format_loader_error(error: &LoaderError) -> String {
  match error {
    LoaderError::Io {
      operation,
      path,
      reason,
    } => {
      format!("I/O Error: {operation} failed for '{path}': {reason}")
    }
    LoaderError::Json { location, reason } => {
      format!("JSON Error at {location}: {reason}")
    }
    LoaderError::CommandFailed { command, reason } => {
      format!("Command Failed: '{command}' - {reason}")
    }
    LoaderError::Validation { context, reason } => {
      format!("Validation Error in {context}: {reason}")
    }
    LoaderError::SessionNotFound { session_id } => {
      format!("Session Not Found: {session_id}")
    }
    LoaderError::InvalidSpec {
      field,
      expected,
      actual,
    } => {
      format!("Invalid Spec: field '{field}' expected {expected}, got {actual}")
    }
    LoaderError::EmptyField { field } => {
      format!("Empty Field: '{field}' is required and cannot be empty")
    }
    LoaderError::InvalidCueOutput { reason } => {
      format!("Invalid CUE Output: {reason}")
    }
    LoaderError::CueBinaryNotFound { details } => {
      format!("CUE Binary Not Found: {details}")
    }
    LoaderError::Security { violation } => {
      format!("Security Error: {violation}")
    }
    LoaderError::FileNotFound { path } => {
      format!("File Not Found: {path}")
    }
  }
}
