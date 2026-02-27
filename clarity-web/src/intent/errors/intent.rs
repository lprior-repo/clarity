use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum IntentError {
  #[error("JSON parse error: {0}")]
  JsonParse(String),
  #[error("missing required field: {0}")]
  MissingField(String),
  #[error("invalid type for field '{field}': expected {expected}, got {actual}")]
  InvalidType {
    field: String,
    expected: String,
    actual: String,
  },
  #[error("invalid value for field '{field}': {reason}")]
  InvalidValue { field: String, reason: String },
  #[error("unknown field: {0}")]
  UnknownField(String),
  #[error("validation failed: {0}")]
  ValidationFailed(String),
  #[error("IO error: {0}")]
  Io(String),
  #[error("file not found: {0}")]
  FileNotFound(String),
  #[error("invalid path: {0}")]
  InvalidPath(String),
  #[error("circular dependency: {0}")]
  CircularDependency(String),
  #[error("constraint violation: {0}")]
  ConstraintViolation(String),
  #[error("configuration error: {0}")]
  Configuration(String),
  #[error("internal error: {0}")]
  Internal(String),
}
