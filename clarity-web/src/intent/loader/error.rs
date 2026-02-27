use crate::intent::parser::ParseError;
use crate::intent::security::SecurityError;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum LoaderError {
  #[error("I/O error: {0}")]
  Io(String),
  #[error("JSON error: {0}")]
  Json(String),
  #[error("command execution failed: {0}")]
  CommandFailed(String),
  #[error("validation error: {0}")]
  Validation(String),
  #[error("session not found: {0}")]
  SessionNotFound(String),
  #[error("invalid spec for field '{field}': expected {expected}, got {actual}")]
  InvalidSpec {
    field: String,
    expected: String,
    actual: String,
  },
  #[error("empty required field: {0}")]
  EmptyField(String),
  #[error("invalid CUE output: {0}")]
  InvalidCueOutput(String),
  #[error("CUE binary not found: {0}")]
  CueBinaryNotFound(String),
  #[error("security error: {0}")]
  Security(String),
  #[error("file not found: {0}")]
  FileNotFound(String),
}

impl From<ParseError> for LoaderError {
  fn from(err: ParseError) -> Self {
    match err {
      ParseError::JsonError(msg) => Self::Json(msg),
      ParseError::MissingField(field) => {
        Self::Validation(format!("missing required field: {field}"))
      }
      ParseError::InvalidType {
        field,
        expected,
        actual,
      } => Self::InvalidSpec {
        field,
        expected,
        actual,
      },
      ParseError::EmptyField(field) => Self::EmptyField(field),
    }
  }
}

impl From<SecurityError> for LoaderError {
  fn from(err: SecurityError) -> Self {
    match err {
      SecurityError::PathTraversal { details } => {
        Self::Security(format!("path traversal: {details}"))
      }
      SecurityError::EncodedPathTraversal { encoding_type } => {
        Self::Security(format!("encoded path traversal: {encoding_type}"))
      }
      SecurityError::ShellMetacharacter { category, ch } => {
        Self::Security(format!("shell metacharacter '{ch}' ({category})"))
      }
      SecurityError::ReDoSVulnerability { vulnerability } => {
        Self::Security(format!("ReDoS vulnerability: {vulnerability}"))
      }
      SecurityError::SessionIdValidation { error } => {
        Self::Security(format!("session ID validation: {error}"))
      }
      SecurityError::NullByteDetected => Self::Security("null byte detected".into()),
      SecurityError::BackslashInPath => Self::Security("backslash in path".into()),
      SecurityError::EmptyInput => Self::Security("empty input".into()),
    }
  }
}

#[must_use]
pub fn format_loader_error(error: &LoaderError) -> String {
  match error {
    LoaderError::Io(msg) => format!("I/O Error: {msg}"),
    LoaderError::Json(msg) => format!("JSON Error: {msg}"),
    LoaderError::CommandFailed(msg) => format!("Command Failed: {msg}"),
    LoaderError::Validation(msg) => format!("Validation Error: {msg}"),
    LoaderError::SessionNotFound(id) => format!("Session Not Found: {id}"),
    LoaderError::InvalidSpec {
      field,
      expected,
      actual,
    } => {
      format!("Invalid Spec: field '{field}' expected {expected}, got {actual}")
    }
    LoaderError::EmptyField(field) => {
      format!("Empty Field: '{field}' is required and cannot be empty")
    }
    LoaderError::InvalidCueOutput(msg) => format!("Invalid CUE Output: {msg}"),
    LoaderError::CueBinaryNotFound(msg) => format!("CUE Binary Not Found: {msg}"),
    LoaderError::Security(msg) => format!("Security Error: {msg}"),
    LoaderError::FileNotFound(path) => format!("File Not Found: {path}"),
  }
}
