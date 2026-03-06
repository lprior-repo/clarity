use super::error::StorageError;
use super::models::SessionSnapshot;
use crate::intent::interview::types::InterviewSession;
use std::collections::HashMap;
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

/// Create a snapshot of an interview session for history tracking.
#[must_use]
pub fn create_snapshot(session: &InterviewSession, description: &str) -> SessionSnapshot {
  let answers = session
    .answers
    .iter()
    .map(|answer| (answer.question_id.clone(), answer.response.clone()))
    .collect::<HashMap<_, _>>();

  let timestamp = session.updated_at.clone();
  let snapshot_id = format!("{}-{timestamp}", session.id);

  SessionSnapshot {
    session_id: session.id.clone(),
    snapshot_id,
    timestamp,
    description: description.to_string(),
    answers,
    gaps_count: session.gaps.len(),
    conflicts_count: session.conflicts.len(),
    stage: session.stage.as_str().to_string(),
  }
}

/// Append a snapshot to a history file.
///
/// # Errors
/// Returns `StorageError` if file operations fail
pub fn append_to_history(
  session: &InterviewSession,
  description: &str,
  history_path: &Path,
) -> Result<(), StorageError> {
  ensure_parent_dir(history_path)?;

  let snapshot = create_snapshot(session, description);
  let line =
    serde_json::to_string(&snapshot).map_err(|error| StorageError::JsonError(error.to_string()))?;

  let file = OpenOptions::new()
    .create(true)
    .append(true)
    .open(history_path)
    .map_err(|error| StorageError::IoError(error.to_string()))?;

  let mut writer = BufWriter::new(file);
  writeln!(writer, "{line}").map_err(|error| StorageError::IoError(error.to_string()))?;
  writer
    .flush()
    .map_err(|error| StorageError::IoError(error.to_string()))
}

/// List all snapshots for a specific session from a history file.
///
/// # Errors
/// Returns `StorageError` if file operations fail or JSON is invalid
pub fn list_session_history(
  history_path: &Path,
  session_id: &str,
) -> Result<Vec<SessionSnapshot>, StorageError> {
  if !history_path.exists() {
    return Ok(Vec::new());
  }

  let file = File::open(history_path).map_err(|error| StorageError::IoError(error.to_string()))?;
  let reader = BufReader::new(file);
  let mut snapshots = Vec::new();

  for (line_num, line_result) in reader.lines().enumerate() {
    let line = line_result.map_err(|error| StorageError::IoError(error.to_string()))?;
    let trimmed = line.trim();

    if trimmed.is_empty() {
      continue;
    }

    let snapshot = serde_json::from_str::<SessionSnapshot>(trimmed).map_err(|error| {
      StorageError::InvalidJsonLine {
        line: line_num + 1,
        error: error.to_string(),
      }
    })?;

    if snapshot.session_id == session_id {
      snapshots.push(snapshot);
    }
  }

  Ok(snapshots)
}
