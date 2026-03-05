//! JSONL storage - Imperative shell for session persistence.
//!
//! This module is the "imperative shell" that handles all I/O operations,
//! delegating pure logic to `jsonl_core`. Following Scott Wlaschin's DDD
//! principle of "functional core, imperative shell".
//!
//! ## Architecture
//!
//! ```text
//! User Code
//!     |
//!     v
//! jsonl.rs (Shell - I/O only)
//!     |
//!     v
//! jsonl_core.rs (Core - Pure logic)
//! ```
//!
//! The shell handles:
//! - File system operations (read, write, create directories)
//! - Buffer management
//! - Error conversion to StorageError
//!
//! The core handles:
//! - JSON serialization/deserialization
//! - Line parsing
//! - Session filtering and lookup

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

#[path = "jsonl_core.rs"]
mod jsonl_core;

use super::error::StorageError;
use crate::intent::interview::types::InterviewSession;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub use jsonl_core::{HasId, JsonlLineParseResult};

// Implement HasId for InterviewSession
impl jsonl_core::HasId for InterviewSession {
  fn id(&self) -> &str {
    &self.id
  }
}

/// Ensure the parent directory exists for a path.
///
/// This is an I/O operation (shell function) that creates directories.
///
/// # Errors
///
/// Returns `StorageError::DirectoryCreationFailed` if the directory cannot be created.
fn ensure_parent_dir(path: &Path) -> Result<(), StorageError> {
  // Shell: I/O operation
  if let Some(parent) = path.parent() {
    if !parent.exists() {
      fs::create_dir_all(parent)
        .map_err(|error| StorageError::DirectoryCreationFailed(error.to_string()))?;
    }
  }
  Ok(())
}

/// Serialize an interview session to a JSONL line.
///
/// This is a pure function wrapper that delegates to the core.
///
/// # Errors
///
/// Returns `StorageError::JsonError` if serialization fails.
pub fn session_to_jsonl_line(session: &InterviewSession) -> Result<String, StorageError> {
  // Core: Pure serialization
  jsonl_core::serialize_to_jsonl(session).map_err(|error| StorageError::JsonError(error))
}

/// Append or update a session in a JSONL file.
///
/// This is an I/O operation (shell function) that writes to the filesystem.
///
/// # Errors
///
/// Returns `StorageError` if:
/// - Directory cannot be created
/// - File cannot be opened
/// - Write operation fails
/// - Serialization fails
pub fn append_session_to_jsonl(
  session: &InterviewSession,
  jsonl_path: &Path,
) -> Result<(), StorageError> {
  // Shell: Ensure directory exists (I/O)
  ensure_parent_dir(jsonl_path)?;

  // Shell: Read existing sessions (I/O)
  let existing_sessions = if jsonl_path.exists() {
    list_sessions_from_jsonl(jsonl_path)?
  } else {
    Vec::new()
  };

  // Core: Filter out session with same ID (pure)
  let mut sessions_to_write = jsonl_core::filter_sessions_by_id(&existing_sessions, &session.id);

  // Core: Add new/updated session (pure)
  sessions_to_write.push(session.clone());

  // Shell: Write all sessions (I/O)
  let file = OpenOptions::new()
    .create(true)
    .write(true)
    .truncate(true)
    .open(jsonl_path)
    .map_err(|error| StorageError::IoError(error.to_string()))?;

  let mut writer = BufWriter::new(file);

  for session_to_write in &sessions_to_write {
    // Core: Serialize session (pure)
    let line = session_to_jsonl_line(session_to_write)?;
    // Shell: Write line (I/O)
    writeln!(writer, "{line}").map_err(|error| StorageError::IoError(error.to_string()))?;
  }

  // Shell: Flush buffer (I/O)
  writer
    .flush()
    .map_err(|error| StorageError::IoError(error.to_string()))
}

/// List all sessions from a JSONL file.
///
/// This is an I/O operation (shell function) that reads from the filesystem.
///
/// # Errors
///
/// Returns `StorageError` if:
/// - File cannot be opened
/// - Read operation fails
/// - JSON parsing fails
pub fn list_sessions_from_jsonl(jsonl_path: &Path) -> Result<Vec<InterviewSession>, StorageError> {
  // Shell: Check file existence (I/O)
  if !jsonl_path.exists() {
    return Ok(Vec::new());
  }

  // Shell: Open file (I/O)
  let file = File::open(jsonl_path).map_err(|error| StorageError::IoError(error.to_string()))?;
  let reader = BufReader::new(file);

  // Shell: Read lines and parse (I/O + Core)
  let mut sessions = Vec::new();
  for (line_num, line_result) in reader.lines().enumerate() {
    // Shell: Read line (I/O)
    let line = line_result.map_err(|error| StorageError::IoError(error.to_string()))?;

    // Core: Parse line (pure)
    match jsonl_core::parse_jsonl_line::<InterviewSession>(&line, line_num + 1) {
      jsonl_core::JsonlLineParseResult::Success(session) => sessions.push(session),
      jsonl_core::JsonlLineParseResult::EmptyLine => {}
      jsonl_core::JsonlLineParseResult::ParseError { line_number, error } => {
        return Err(StorageError::InvalidJsonLine {
          line: line_number,
          error,
        });
      }
    }
  }

  Ok(sessions)
}

/// Get a specific session from a JSONL file by ID.
///
/// This is an I/O operation (shell function) that reads from the filesystem.
///
/// # Errors
///
/// Returns `StorageError::SessionNotFound` if the session doesn't exist.
pub fn get_session_from_jsonl(
  jsonl_path: &Path,
  session_id: &str,
) -> Result<InterviewSession, StorageError> {
  // Shell: Read all sessions (I/O)
  let sessions = list_sessions_from_jsonl(jsonl_path)?;

  // Core: Find session by ID (pure)
  jsonl_core::find_session_by_id(&sessions, session_id)
    .ok_or_else(|| StorageError::SessionNotFound(session_id.to_string()))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::intent::interview::types::Profile;

  fn create_test_session() -> InterviewSession {
    InterviewSession {
      id: "test-id".to_string(),
      profile: Profile::default(),
      stage: crate::intent::interview::types::InterviewStage::Discovery,
      answers: vec![],
      gaps: vec![],
      conflicts: vec![],
      created_at: "2024-01-01T00:00:00Z".to_string(),
      updated_at: "2024-01-01T00:00:00Z".to_string(),
      completed_at: None,
      rounds_completed: 0,
      raw_notes: String::new(),
      current_phase: 1,
      completed_phases: vec![],
    }
  }

  #[test]
  fn session_to_jsonl_line_works() {
    let session = create_test_session();
    let result = session_to_jsonl_line(&session);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("test-id"));
  }

  #[test]
  fn list_sessions_from_nonexistent_file_returns_empty() {
    let result = list_sessions_from_jsonl(Path::new("/nonexistent/path/to/file.jsonl"));
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
  }
}
