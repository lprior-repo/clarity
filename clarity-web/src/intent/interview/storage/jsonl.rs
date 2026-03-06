use super::error::StorageError;
use crate::intent::interview::types::InterviewSession;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

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
pub fn session_to_jsonl_line(session: &InterviewSession) -> Result<String, StorageError> {
  serde_json::to_string(session).map_err(|error| StorageError::JsonError(error.to_string()))
}

/// Append or update a session in a JSONL file.
pub fn append_session_to_jsonl(
  session: &InterviewSession,
  jsonl_path: &Path,
) -> Result<(), StorageError> {
  ensure_parent_dir(jsonl_path)?;

  let existing_sessions = if jsonl_path.exists() {
    list_sessions_from_jsonl(jsonl_path)?
      .into_iter()
      .filter(|existing| existing.id != session.id)
      .collect::<Vec<_>>()
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

  for existing in existing_sessions {
    let line = session_to_jsonl_line(&existing)?;
    writeln!(writer, "{line}").map_err(|error| StorageError::IoError(error.to_string()))?;
  }

  let line = session_to_jsonl_line(session)?;
  writeln!(writer, "{line}").map_err(|error| StorageError::IoError(error.to_string()))?;
  writer
    .flush()
    .map_err(|error| StorageError::IoError(error.to_string()))
}

/// List all sessions from a JSONL file.
pub fn list_sessions_from_jsonl(jsonl_path: &Path) -> Result<Vec<InterviewSession>, StorageError> {
  if !jsonl_path.exists() {
    return Ok(Vec::new());
  }

  let file = File::open(jsonl_path).map_err(|error| StorageError::IoError(error.to_string()))?;
  let reader = BufReader::new(file);

  let mut sessions = Vec::new();
  for (line_num, line_result) in reader.lines().enumerate() {
    let line = line_result.map_err(|error| StorageError::IoError(error.to_string()))?;
    let trimmed = line.trim();

    if trimmed.is_empty() {
      continue;
    }

    let parsed = serde_json::from_str::<InterviewSession>(trimmed).map_err(|error| {
      StorageError::InvalidJsonLine {
        line: line_num + 1,
        error: error.to_string(),
      }
    })?;
    sessions.push(parsed);
  }

  Ok(sessions)
}

/// Get a specific session from a JSONL file by ID.
pub fn get_session_from_jsonl(
  jsonl_path: &Path,
  session_id: &str,
) -> Result<InterviewSession, StorageError> {
  list_sessions_from_jsonl(jsonl_path)?
    .into_iter()
    .find(|session| session.id == session_id)
    .ok_or_else(|| StorageError::SessionNotFound(session_id.to_string()))
}
