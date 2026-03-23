use crate::intent::util::array_indexing::ArrayIndexError;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InterpolationError {
  #[error("variable not found: {0}")]
  VariableNotFound(String),
  #[error("invalid path: {0}")]
  InvalidPath(String),
  #[error("array index out of bounds: {index} in array of length {length}")]
  IndexOutOfBounds { index: usize, length: usize },
  #[error("not an array: {0}")]
  NotAnArray(String),
  #[error("JSON error: {0}")]
  JsonError(String),
  #[error("regex error: {0}")]
  RegexError(String),
  #[error("malformed interpolation: {0}")]
  MalformedInterpolation(String),
}

impl From<ArrayIndexError> for InterpolationError {
  fn from(err: ArrayIndexError) -> Self {
    match err {
      ArrayIndexError::InvalidPath(path) => Self::InvalidPath(path),
      ArrayIndexError::IndexOutOfBounds { index, length } => Self::IndexOutOfBounds {
        index: usize::try_from(index.max(0)).map_or(usize::MAX, |v| v),
        length,
      },
      ArrayIndexError::NotAnArray { field, .. } => Self::NotAnArray(field),
      ArrayIndexError::FieldNotFound(field) => Self::VariableNotFound(field),
    }
  }
}
