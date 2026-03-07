use super::error::StorageError;
use super::jsonl_core::{self, HasId};
use crate::intent::interview::types::InterviewSession;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

impl HasId for InterviewSession {
  fn id(&self) -> &str {
    &self.id
  }
}

fn ensure_parent_dir(path: &Path) -> Result<(), StorageError> {
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
/// # Errors
/// Returns `StorageError` if JSON serialization fails
pub fn session_to_jsonl_line(session: &InterviewSession) -> Result<String, StorageError> {
  jsonl_core::serialize_to_jsonl(session).map_err(|error| match error {
    jsonl_core::JsonlCoreError::Serialization { details } => StorageError::JsonError(details),
  })
}

/// Append or update a session in a JSONL file.
///
/// # Errors
/// Returns `StorageError` if file operations or serialization fails
pub fn append_session_to_jsonl(
  session: &InterviewSession,
  jsonl_path: &Path,
) -> Result<(), StorageError> {
  ensure_parent_dir(jsonl_path)?;

  let existing_sessions = if jsonl_path.exists() {
    let sessions = list_sessions_from_jsonl(jsonl_path)?;
    jsonl_core::filter_sessions_by_id(&sessions, &session.id)
  } else {
    Vec::new()
  };

  let file = OpenOptions::new()
    .create(true)
    .write(true)
    .truncate(true)
    .open(jsonl_path)
    .map_err(|error| StorageError::IoError(error.to_string()))?;

  let mut writer = BufWriter::new(file);

  let mut sessions_to_write = existing_sessions;
  sessions_to_write.push(session.clone());
  let content =
    jsonl_core::build_jsonl_content(&sessions_to_write).map_err(|error| match error {
      jsonl_core::JsonlCoreError::Serialization { details } => StorageError::JsonError(details),
    })?;

  if content.is_empty() {
    writer
      .flush()
      .map_err(|error| StorageError::IoError(error.to_string()))?;
    return Ok(());
  }

  writeln!(writer, "{content}").map_err(|error| StorageError::IoError(error.to_string()))?;
  writer
    .flush()
    .map_err(|error| StorageError::IoError(error.to_string()))
}

/// List all sessions from a JSONL file.
///
/// # Errors
/// Returns `StorageError` if file operations fail or JSON is invalid
pub fn list_sessions_from_jsonl(jsonl_path: &Path) -> Result<Vec<InterviewSession>, StorageError> {
  if !jsonl_path.exists() {
    return Ok(Vec::new());
  }

  let file = File::open(jsonl_path).map_err(|error| StorageError::IoError(error.to_string()))?;
  let reader = BufReader::new(file);

  reader
    .lines()
    .enumerate()
    .try_fold(Vec::new(), |mut sessions, (line_num, line_result)| {
      let line = line_result.map_err(|error| StorageError::IoError(error.to_string()))?;

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

      Ok(sessions)
    })
}

/// Get a specific session from a JSONL file by ID.
///
/// # Errors
/// Returns `StorageError` if session is not found or file operations fail
pub fn get_session_from_jsonl(
  jsonl_path: &Path,
  session_id: &str,
) -> Result<InterviewSession, StorageError> {
  let sessions = list_sessions_from_jsonl(jsonl_path)?;
  jsonl_core::find_session_by_id(&sessions, session_id)
    .ok_or_else(|| StorageError::SessionNotFound(session_id.to_string()))
}
