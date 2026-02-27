use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ArrayIndexError {
  #[error("invalid path: {0}")]
  InvalidPath(String),

  #[error("index {index} out of bounds for array of length {length}")]
  IndexOutOfBounds { index: isize, length: usize },

  #[error("not an array: attempted to index '{field}' which is a {actual_type}")]
  NotAnArray { field: String, actual_type: String },

  #[error("field not found: '{0}'")]
  FieldNotFound(String),
}
