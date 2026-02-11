//! br Show Data Models
//!
//! Contains the data structures for representing `br show` command output
//! and associated error types.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Represents an issue from the `br` command output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrIssue {
  /// The issue ID
  pub id: String,
  /// The issue title
  pub title: String,
  /// The issue status
  pub status: String,
  /// The priority (1=high, 2=medium, 3=low)
  pub priority: u32,
  /// The issue type
  pub issue_type: String,
  /// When the issue was created
  pub created_at: chrono::DateTime<chrono::Utc>,
  /// Who created the issue
  pub created_by: String,
  /// When the issue was last updated
  pub updated_at: chrono::DateTime<chrono::Utc>,
  /// Source repository
  pub source_repo: String,
  /// Compaction level
  pub compaction_level: u32,
  /// Original size
  pub original_size: u64,
}

/// Domain errors for br show functionality
#[derive(Debug, Error)]
pub enum BrShowError {
  /// Error executing the br command
  #[error("br command execution failed: {0}")]
  CommandFailed(String),

  /// Error parsing br command output
  #[error("failed to parse br output: {0}")]
  ParseError(String),

  /// Issue not found
  #[error("issue not found: {0}")]
  IssueNotFound(String),

  /// JSON output parsing error
  #[error("JSON parsing error: {0}")]
  JsonError(#[from] serde_json::Error),

  /// IO error
  #[error("IO error: {0}")]
  IoError(#[from] std::io::Error),

  /// UTF8 conversion error
  #[error("UTF8 conversion error: {0}")]
  Utf8Error(#[from] std::string::FromUtf8Error),
}

impl BrShowError {
  /// Create a command failed error
  pub fn command_failed<S: Into<String>>(message: S) -> Self {
    Self::CommandFailed(message.into())
  }

  /// Create a parse error
  pub fn parse_error<S: Into<String>>(message: S) -> Self {
    Self::ParseError(message.into())
  }

  /// Create an issue not found error
  pub fn issue_not_found<S: Into<String>>(id: S) -> Self {
    Self::IssueNotFound(id.into())
  }
}
