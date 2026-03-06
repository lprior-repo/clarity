use thiserror::Error;

/// Error type for storage operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StorageError {
  /// I/O error during file operations.
  #[error("I/O error: {0}")]
  IoError(String),

  /// JSON serialization/deserialization error.
  #[error("JSON error: {0}")]
  JsonError(String),

  /// Session not found in storage.
  #[error("session not found: {0}")]
  SessionNotFound(String),

  /// Invalid JSON on a specific line.
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
}
