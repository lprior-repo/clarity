//! Interview Storage - JSONL persistence for interview sessions
//!
//! This module provides:
//! - JSONL file-based persistence for interview sessions
//! - Session history tracking via snapshots
//! - CRUD operations for sessions in JSONL format
//!
//! All operations return Result types and avoid unwrap/expect.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use super::types::InterviewSession;

#[cfg(test)]
use super::types::InterviewStage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use thiserror::Error;

/// Error type for storage operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StorageError {
    /// I/O error during file operations
    #[error("I/O error: {0}")]
    IoError(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(String),

    /// Session not found in storage
    #[error("session not found: {0}")]
    SessionNotFound(String),

    /// Invalid JSON on a specific line
    #[error("invalid JSON on line {line}: {error}")]
    InvalidJsonLine {
        /// Line number where the error occurred
        line: usize,
        /// The error message
        error: String,
    },

    /// Failed to create directory
    #[error("directory creation failed: {0}")]
    DirectoryCreationFailed(String),
}

/// Snapshot for history tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSnapshot {
    /// ID of the session this snapshot belongs to
    pub session_id: String,

    /// Unique identifier for this snapshot
    pub snapshot_id: String,

    /// When this snapshot was taken (ISO 8601)
    pub timestamp: String,

    /// Human-readable description of the snapshot
    pub description: String,

    /// Current answers at snapshot time (`question_id` -> response)
    pub answers: HashMap<String, String>,

    /// Number of gaps at snapshot time
    pub gaps_count: usize,

    /// Number of conflicts at snapshot time
    pub conflicts_count: usize,

    /// Session stage at snapshot time
    pub stage: String,
}

/// Change type for answer differences
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnswerChangeType {
    /// Answer was added (not present in "from" session)
    Added,
    /// Answer was modified (present in both but response differs)
    Modified,
    /// Answer was removed (not present in "to" session)
    Removed,
}

/// Difference for a single answer between two sessions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnswerDiff {
    /// Question identifier
    pub question_id: String,
    /// The question text for context
    pub question_text: String,
    /// Response in the "from" session (None if added)
    pub old_response: Option<String>,
    /// Response in the "to" session (None if removed)
    pub new_response: Option<String>,
    /// Type of change that occurred
    pub change_type: AnswerChangeType,
}

/// Difference between two interview sessions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionDiff {
    /// ID of the "from" session
    pub from_session_id: String,
    /// ID of the "to" session
    pub to_session_id: String,
    /// Timestamp of the "from" session
    pub from_timestamp: String,
    /// Timestamp of the "to" session
    pub to_timestamp: String,
    /// Whether the stage changed between sessions
    pub stage_changed: bool,
    /// Stage in the "from" session (None if not applicable)
    pub old_stage: Option<String>,
    /// Stage in the "to" session (None if not applicable)
    pub new_stage: Option<String>,
    /// Answers that were added
    pub answers_added: Vec<AnswerDiff>,
    /// Answers that were modified
    pub answers_modified: Vec<AnswerDiff>,
    /// Answers that were removed
    pub answers_removed: Vec<AnswerDiff>,
    /// Change in gaps count (positive = new gaps, negative = resolved)
    pub gaps_added: i32,
    /// Change in conflicts count (positive = new conflicts, negative = resolved)
    pub conflicts_added: i32,
}

/// Serialize an interview session to a JSONL line.
///
/// # Errors
/// Returns `StorageError::JsonError` if serialization fails.
///
/// # Example
/// ```ignore
/// let session = InterviewSession::new("id".to_string(), Profile::Api, "2026-01-01T00:00:00Z".to_string());
/// let line = session_to_jsonl_line(&session)?;
/// assert!(line.starts_with("{\"id\":\"id\""));
/// ```
pub fn session_to_jsonl_line(session: &InterviewSession) -> Result<String, StorageError> {
    serde_json::to_string(session).map_err(|e| StorageError::JsonError(e.to_string()))
}

/// Append or update a session in a JSONL file.
///
/// This function:
/// 1. Creates parent directories if they don't exist
/// 2. Reads existing sessions from the file
/// 3. Filters out any session with the same ID (for updates)
/// 4. Appends the new session
///
/// # Errors
/// - `StorageError::DirectoryCreationFailed` if parent directories can't be created
/// - `StorageError::IoError` for file read/write errors
/// - `StorageError::JsonError` for serialization errors
pub fn append_session_to_jsonl(
    session: &InterviewSession,
    jsonl_path: &Path,
) -> Result<(), StorageError> {
    // Create parent directories if needed
    if let Some(parent) = jsonl_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| StorageError::DirectoryCreationFailed(e.to_string()))?;
        }
    }

    // Read existing sessions, filtering out any with the same ID
    let existing_sessions = if jsonl_path.exists() {
        list_sessions_from_jsonl(jsonl_path)?
            .into_iter()
            .filter(|s| s.id != session.id)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    // Create or truncate the file and write all sessions
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(jsonl_path)
        .map_err(|e| StorageError::IoError(e.to_string()))?;

    let mut writer = BufWriter::new(file);

    // Write existing sessions
    for existing_session in existing_sessions {
        let line = session_to_jsonl_line(&existing_session)?;
        writeln!(writer, "{line}").map_err(|e| StorageError::IoError(e.to_string()))?;
    }

    // Write new session
    let line = session_to_jsonl_line(session)?;
    writeln!(writer, "{line}").map_err(|e| StorageError::IoError(e.to_string()))?;

    writer
        .flush()
        .map_err(|e| StorageError::IoError(e.to_string()))?;

    Ok(())
}

/// List all sessions from a JSONL file.
///
/// Reads each line and parses it as a JSON `InterviewSession`.
/// Lines that fail to parse are skipped (but see `list_sessions_from_jsonl_strict` for strict mode).
///
/// # Errors
/// - `StorageError::IoError` if the file can't be read
/// - `StorageError::InvalidJsonLine` if strict mode encounters invalid JSON
pub fn list_sessions_from_jsonl(jsonl_path: &Path) -> Result<Vec<InterviewSession>, StorageError> {
    if !jsonl_path.exists() {
        return Ok(Vec::new());
    }

    let file =
        File::open(jsonl_path).map_err(|e| StorageError::IoError(e.to_string()))?;

    let reader = BufReader::new(file);
    let mut sessions = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result.map_err(|e| StorageError::IoError(e.to_string()))?;

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match serde_json::from_str::<InterviewSession>(trimmed) {
            Ok(session) => sessions.push(session),
            Err(e) => {
                // Return error with line information for debugging
                return Err(StorageError::InvalidJsonLine {
                    line: line_num + 1,
                    error: e.to_string(),
                });
            }
        }
    }

    Ok(sessions)
}

/// Get a specific session from a JSONL file by ID.
///
/// # Errors
/// - `StorageError::IoError` if the file can't be read
/// - `StorageError::SessionNotFound` if no session matches the ID
pub fn get_session_from_jsonl(
    jsonl_path: &Path,
    session_id: &str,
) -> Result<InterviewSession, StorageError> {
    let sessions = list_sessions_from_jsonl(jsonl_path)?;

    sessions
        .into_iter()
        .find(|s| s.id == session_id)
        .ok_or_else(|| StorageError::SessionNotFound(session_id.to_string()))
}

/// Create a snapshot of an interview session for history tracking.
///
/// The snapshot captures the current state of the session including:
/// - All answers mapped as `question_id` -> response
/// - Gap and conflict counts
/// - Current stage
///
/// # Example
/// ```ignore
/// let session = InterviewSession::new("sess-1".into(), Profile::Api, "2026-01-01T00:00:00Z".into());
/// let snapshot = create_snapshot(&session, "Initial state");
/// assert!(snapshot.snapshot_id.starts_with("sess-1-"));
/// ```
#[must_use]
pub fn create_snapshot(session: &InterviewSession, description: &str) -> SessionSnapshot {
    let timestamp = session.updated_at.clone();

    // Build answers map from question_id to response
    let answers: HashMap<String, String> = session
        .answers
        .iter()
        .map(|answer| (answer.question_id.clone(), answer.response.clone()))
        .collect();

    let snapshot_id = format!("{}-{}", session.id, timestamp);

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
/// Creates parent directories if needed and appends the snapshot as JSON.
///
/// # Errors
/// - `StorageError::DirectoryCreationFailed` if parent directories can't be created
/// - `StorageError::IoError` for file write errors
/// - `StorageError::JsonError` for serialization errors
pub fn append_to_history(
    session: &InterviewSession,
    description: &str,
    history_path: &Path,
) -> Result<(), StorageError> {
    // Create parent directories if needed
    if let Some(parent) = history_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| StorageError::DirectoryCreationFailed(e.to_string()))?;
        }
    }

    let snapshot = create_snapshot(session, description);
    let line = serde_json::to_string(&snapshot).map_err(|e| StorageError::JsonError(e.to_string()))?;

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(history_path)
        .map_err(|e| StorageError::IoError(e.to_string()))?;

    let mut writer = BufWriter::new(file);
    writeln!(writer, "{line}").map_err(|e| StorageError::IoError(e.to_string()))?;

    writer
        .flush()
        .map_err(|e| StorageError::IoError(e.to_string()))?;

    Ok(())
}

/// List all snapshots for a specific session from a history file.
///
/// # Errors
/// - `StorageError::IoError` if the file can't be read
/// - `StorageError::InvalidJsonLine` if a line contains invalid JSON
pub fn list_session_history(
    history_path: &Path,
    session_id: &str,
) -> Result<Vec<SessionSnapshot>, StorageError> {
    if !history_path.exists() {
        return Ok(Vec::new());
    }

    let file =
        File::open(history_path).map_err(|e| StorageError::IoError(e.to_string()))?;

    let reader = BufReader::new(file);
    let mut snapshots = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result.map_err(|e| StorageError::IoError(e.to_string()))?;

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match serde_json::from_str::<SessionSnapshot>(trimmed) {
            Ok(snapshot) => {
                if snapshot.session_id == session_id {
                    snapshots.push(snapshot);
                }
            }
            Err(e) => {
                return Err(StorageError::InvalidJsonLine {
                    line: line_num + 1,
                    error: e.to_string(),
                });
            }
        }
    }

    Ok(snapshots)
}

// ==================== Session Diffing (WP16) ====================

/// Compute the difference between two interview sessions.
///
/// This function compares two sessions and returns a `SessionDiff` containing:
/// - Stage changes
/// - Added, modified, and removed answers
/// - Changes in gaps and conflicts counts
///
/// # Arguments
/// * `from` - The earlier session (baseline)
/// * `to` - The later session (current state)
///
/// # Returns
/// A `SessionDiff` struct describing all changes between the sessions.
///
/// # Example
/// ```ignore
/// let session_v1 = create_test_session("sess-1");
/// let session_v2 = create_test_session_with_answers("sess-1");
/// let diff = diff_sessions(&session_v1, &session_v2);
/// assert!(!diff.answers_added.is_empty());
/// ```
#[must_use]
pub fn diff_sessions(from: &InterviewSession, to: &InterviewSession) -> SessionDiff {
    // Build lookup maps for answers by question_id
    let from_answers: HashMap<&str, &str> = from
        .answers
        .iter()
        .map(|a| (a.question_id.as_str(), a.response.as_str()))
        .collect();

    let to_answers: HashMap<&str, &str> = to
        .answers
        .iter()
        .map(|a| (a.question_id.as_str(), a.response.as_str()))
        .collect();

    // Find Added: in "to" but not "from"
    let answers_added: Vec<AnswerDiff> = to
        .answers
        .iter()
        .filter(|a| !from_answers.contains_key(a.question_id.as_str()))
        .map(|a| AnswerDiff {
            question_id: a.question_id.clone(),
            question_text: a.question_text.clone(),
            old_response: None,
            new_response: Some(a.response.clone()),
            change_type: AnswerChangeType::Added,
        })
        .collect();

    // Find Modified: in both but response differs
    let answers_modified: Vec<AnswerDiff> = to
        .answers
        .iter()
        .filter(|a| {
            from_answers
                .get(a.question_id.as_str())
                .map_or(false, |&old| old != a.response)
        })
        .map(|a| AnswerDiff {
            question_id: a.question_id.clone(),
            question_text: a.question_text.clone(),
            old_response: from_answers.get(a.question_id.as_str()).map(|s| (*s).to_string()),
            new_response: Some(a.response.clone()),
            change_type: AnswerChangeType::Modified,
        })
        .collect();

    // Find Removed: in "from" but not "to"
    let answers_removed: Vec<AnswerDiff> = from
        .answers
        .iter()
        .filter(|a| !to_answers.contains_key(a.question_id.as_str()))
        .map(|a| AnswerDiff {
            question_id: a.question_id.clone(),
            question_text: a.question_text.clone(),
            old_response: Some(a.response.clone()),
            new_response: None,
            change_type: AnswerChangeType::Removed,
        })
        .collect();

    // Calculate stage change
    let stage_changed = from.stage != to.stage;
    let old_stage = Some(from.stage.as_str().to_string());
    let new_stage = Some(to.stage.as_str().to_string());

    // Calculate gaps and conflicts changes
    let gaps_added = i32::try_from(to.gaps.len())
        .map_or(i32::MAX, |to_gaps| {
            i32::try_from(from.gaps.len())
                .map_or(i32::MIN, |from_gaps| to_gaps - from_gaps)
        });

    let conflicts_added = i32::try_from(to.conflicts.len())
        .map_or(i32::MAX, |to_conflicts| {
            i32::try_from(from.conflicts.len())
                .map_or(i32::MIN, |from_conflicts| to_conflicts - from_conflicts)
        });

    SessionDiff {
        from_session_id: from.id.clone(),
        to_session_id: to.id.clone(),
        from_timestamp: from.updated_at.clone(),
        to_timestamp: to.updated_at.clone(),
        stage_changed,
        old_stage,
        new_stage,
        answers_added,
        answers_modified,
        answers_removed,
        gaps_added,
        conflicts_added,
    }
}

/// Format a session diff as human-readable text.
///
/// Output format:
/// - Header with session IDs and timestamps
/// - Stage change (if applicable)
/// - Answers added (+ prefix)
/// - Answers modified (~ prefix with old -> new)
/// - Answers removed (- prefix)
/// - Gaps summary
/// - Conflicts summary
///
/// Long responses are truncated to 50 characters.
///
/// # Example
/// ```ignore
/// let diff = diff_sessions(&session_v1, &session_v2);
/// let formatted = format_diff(&diff);
/// println!("{}", formatted);
/// ```
#[must_use]
pub fn format_diff(diff: &SessionDiff) -> String {
    const MAX_RESPONSE_LEN: usize = 50;

    /// Truncate a string to MAX_RESPONSE_LEN characters, respecting UTF-8 boundaries.
    fn truncate(s: &str) -> String {
        if s.chars().count() > MAX_RESPONSE_LEN {
            format!("{}...", s.chars().take(MAX_RESPONSE_LEN).collect::<String>())
        } else {
            s.to_string()
        }
    }

    let format_response = |opt: &Option<String>| -> String {
        opt.as_ref()
            .map(|s| truncate(s))
            .unwrap_or_else(|| "(none)".to_string())
    };

    let mut output = String::new();

    // Header
    output.push_str(&format!(
        "Session Diff: {} -> {}\n",
        diff.from_session_id, diff.to_session_id
    ));
    output.push_str(&format!(
        "Timestamps: {} -> {}\n\n",
        diff.from_timestamp, diff.to_timestamp
    ));

    // Stage change
    if diff.stage_changed {
        output.push_str(&format!(
            "Stage: {} -> {}\n\n",
            diff.old_stage.as_ref().map(|s| s.as_str()).unwrap_or("(none)"),
            diff.new_stage.as_ref().map(|s| s.as_str()).unwrap_or("(none)")
        ));
    }

    // Answers added
    if !diff.answers_added.is_empty() {
        output.push_str(&format!("Answers Added ({}):\n", diff.answers_added.len()));
        for answer in &diff.answers_added {
            output.push_str(&format!(
                "  + [{}] {}: {}\n",
                answer.question_id,
                truncate(&answer.question_text),
                format_response(&answer.new_response)
            ));
        }
        output.push('\n');
    }

    // Answers modified
    if !diff.answers_modified.is_empty() {
        output.push_str(&format!("Answers Modified ({}):\n", diff.answers_modified.len()));
        for answer in &diff.answers_modified {
            output.push_str(&format!(
                "  ~ [{}] {}:\n    {} -> {}\n",
                answer.question_id,
                truncate(&answer.question_text),
                format_response(&answer.old_response),
                format_response(&answer.new_response)
            ));
        }
        output.push('\n');
    }

    // Answers removed
    if !diff.answers_removed.is_empty() {
        output.push_str(&format!("Answers Removed ({}):\n", diff.answers_removed.len()));
        for answer in &diff.answers_removed {
            output.push_str(&format!(
                "  - [{}] {}: {}\n",
                answer.question_id,
                truncate(&answer.question_text),
                format_response(&answer.old_response)
            ));
        }
        output.push('\n');
    }

    // Gaps summary
    match diff.gaps_added.cmp(&0) {
        std::cmp::Ordering::Greater => {
            output.push_str(&format!("Gaps: +{} new gap(s)\n", diff.gaps_added));
        }
        std::cmp::Ordering::Less => {
            output.push_str(&format!("Gaps: {} gap(s) resolved\n", -diff.gaps_added));
        }
        std::cmp::Ordering::Equal => {
            output.push_str("Gaps: No change\n");
        }
    }

    // Conflicts summary
    match diff.conflicts_added.cmp(&0) {
        std::cmp::Ordering::Greater => {
            output.push_str(&format!("Conflicts: +{} new conflict(s)\n", diff.conflicts_added));
        }
        std::cmp::Ordering::Less => {
            output.push_str(&format!("Conflicts: {} conflict(s) resolved\n", -diff.conflicts_added));
        }
        std::cmp::Ordering::Equal => {
            output.push_str("Conflicts: No change\n");
        }
    }

    output
}

/// Compute the difference between two session snapshots.
///
/// This function is similar to `diff_sessions` but works with snapshots,
/// comparing the state captured at two different points in time.
///
/// # Arguments
/// * `from` - The earlier snapshot (baseline)
/// * `to` - The later snapshot (current state)
///
/// # Returns
/// A `SessionDiff` struct describing all changes between the snapshots.
///
/// # Example
/// ```ignore
/// let snapshot_v1 = create_snapshot(&session, "State 1");
/// // ... modify session ...
/// let snapshot_v2 = create_snapshot(&session, "State 2");
/// let diff = diff_snapshots(&snapshot_v1, &snapshot_v2);
/// ```
#[must_use]
pub fn diff_snapshots(from: &SessionSnapshot, to: &SessionSnapshot) -> SessionDiff {
    // Find Added: in "to" but not "from"
    let answers_added: Vec<AnswerDiff> = to
        .answers
        .iter()
        .filter(|(id, _)| !from.answers.contains_key(*id))
        .map(|(id, response)| AnswerDiff {
            question_id: id.clone(),
            question_text: id.clone(), // Snapshots don't have question_text, use id
            old_response: None,
            new_response: Some(response.clone()),
            change_type: AnswerChangeType::Added,
        })
        .collect();

    // Find Modified: in both but response differs
    let answers_modified: Vec<AnswerDiff> = to
        .answers
        .iter()
        .filter(|(id, response)| {
            from.answers
                .get(*id)
                .map_or(false, |old| old != *response)
        })
        .map(|(id, response)| AnswerDiff {
            question_id: id.clone(),
            question_text: id.clone(),
            old_response: from.answers.get(id).cloned(),
            new_response: Some(response.clone()),
            change_type: AnswerChangeType::Modified,
        })
        .collect();

    // Find Removed: in "from" but not "to"
    let answers_removed: Vec<AnswerDiff> = from
        .answers
        .iter()
        .filter(|(id, _)| !to.answers.contains_key(*id))
        .map(|(id, response)| AnswerDiff {
            question_id: id.clone(),
            question_text: id.clone(),
            old_response: Some(response.clone()),
            new_response: None,
            change_type: AnswerChangeType::Removed,
        })
        .collect();

    // Calculate stage change
    let stage_changed = from.stage != to.stage;

    // Calculate gaps and conflicts changes
    let gaps_added = i32::try_from(to.gaps_count)
        .map_or(i32::MAX, |to_gaps| {
            i32::try_from(from.gaps_count)
                .map_or(i32::MIN, |from_gaps| to_gaps - from_gaps)
        });

    let conflicts_added = i32::try_from(to.conflicts_count)
        .map_or(i32::MAX, |to_conflicts| {
            i32::try_from(from.conflicts_count)
                .map_or(i32::MIN, |from_conflicts| to_conflicts - from_conflicts)
        });

    SessionDiff {
        from_session_id: from.session_id.clone(),
        to_session_id: to.session_id.clone(),
        from_timestamp: from.timestamp.clone(),
        to_timestamp: to.timestamp.clone(),
        stage_changed,
        old_stage: Some(from.stage.clone()),
        new_stage: Some(to.stage.clone()),
        answers_added,
        answers_modified,
        answers_removed,
        gaps_added,
        conflicts_added,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent::interview::types::{Answer, Gap, Profile};
    use tempfile::NamedTempFile;

    fn create_test_session(id: &str) -> InterviewSession {
        InterviewSession::new(
            id.to_string(),
            Profile::Api,
            "2026-02-27T00:00:00Z".to_string(),
        )
    }

    fn create_test_session_with_answers(id: &str) -> InterviewSession {
        let mut session = create_test_session(id);
        session.answers.push(Answer {
            question_id: "q1".to_string(),
            question_text: "What is the API?".to_string(),
            response: "REST API for users".to_string(),
            extracted: HashMap::new(),
            confidence: 0.9,
            ..Answer::default()
        });
        session.answers.push(Answer {
            question_id: "q2".to_string(),
            question_text: "What is auth?".to_string(),
            response: "Bearer token".to_string(),
            extracted: HashMap::new(),
            confidence: 0.8,
            ..Answer::default()
        });
        session
    }

    // ==================== session_to_jsonl_line tests ====================

    #[test]
    fn test_session_to_jsonl_line_success() {
        let session = create_test_session("test-id");
        let result = session_to_jsonl_line(&session);

        assert!(result.is_ok());
        let line = result.expect("line should exist");
        assert!(line.contains("\"id\":\"test-id\""));
        assert!(line.contains("\"profile\":\"api\""));
    }

    #[test]
    fn test_session_to_jsonl_line_roundtrip() {
        let session = create_test_session_with_answers("roundtrip-test");
        let line = session_to_jsonl_line(&session).expect("should serialize");

        let parsed: InterviewSession =
            serde_json::from_str(&line).expect("should deserialize");

        assert_eq!(session.id, parsed.id);
        assert_eq!(session.profile, parsed.profile);
        assert_eq!(session.answers.len(), parsed.answers.len());
    }

    // ==================== append_session_to_jsonl tests ====================

    #[test]
    fn test_append_session_to_jsonl_creates_file() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let jsonl_path = temp_dir.path().join("sessions.jsonl");

        let session = create_test_session("new-session");
        let result = append_session_to_jsonl(&session, &jsonl_path);

        assert!(result.is_ok());
        assert!(jsonl_path.exists());

        let sessions = list_sessions_from_jsonl(&jsonl_path).expect("should list");
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "new-session");
    }

    #[test]
    fn test_append_session_to_jsonl_creates_parent_dirs() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let jsonl_path = temp_dir.path().join("nested/dir/sessions.jsonl");

        let session = create_test_session("nested-session");
        let result = append_session_to_jsonl(&session, &jsonl_path);

        assert!(result.is_ok());
        assert!(jsonl_path.exists());
    }

    #[test]
    fn test_append_session_to_jsonl_appends_multiple() {
        let temp_file = NamedTempFile::new().expect("temp file");
        let jsonl_path = temp_file.path();

        let session1 = create_test_session("session-1");
        let session2 = create_test_session("session-2");

        append_session_to_jsonl(&session1, jsonl_path).expect("should append 1");
        append_session_to_jsonl(&session2, jsonl_path).expect("should append 2");

        let sessions = list_sessions_from_jsonl(jsonl_path).expect("should list");
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_append_session_to_jsonl_updates_existing() {
        let temp_file = NamedTempFile::new().expect("temp file");
        let jsonl_path = temp_file.path();

        // Create initial session
        let mut session1 = create_test_session("session-1");
        session1.raw_notes = "Initial notes".to_string();
        append_session_to_jsonl(&session1, jsonl_path).expect("should append");

        // Update the session
        session1.raw_notes = "Updated notes".to_string();
        append_session_to_jsonl(&session1, jsonl_path).expect("should update");

        let sessions = list_sessions_from_jsonl(jsonl_path).expect("should list");
        assert_eq!(sessions.len(), 1); // Still only 1 session
        assert_eq!(sessions[0].raw_notes, "Updated notes");
    }

    #[test]
    fn test_append_session_to_jsonl_preserves_other_sessions() {
        let temp_file = NamedTempFile::new().expect("temp file");
        let jsonl_path = temp_file.path();

        let session1 = create_test_session("session-1");
        let mut session2 = create_test_session("session-2");
        let session3 = create_test_session("session-3");

        append_session_to_jsonl(&session1, jsonl_path).expect("should append 1");
        append_session_to_jsonl(&session2, jsonl_path).expect("should append 2");
        append_session_to_jsonl(&session3, jsonl_path).expect("should append 3");

        // Update session2
        session2.raw_notes = "Updated session 2".to_string();
        append_session_to_jsonl(&session2, jsonl_path).expect("should update 2");

        let sessions = list_sessions_from_jsonl(jsonl_path).expect("should list");
        assert_eq!(sessions.len(), 3);

        let session2_found = sessions
            .iter()
            .find(|s| s.id == "session-2")
            .expect("session 2 should exist");
        assert_eq!(session2_found.raw_notes, "Updated session 2");

        // Verify others still exist
        assert!(sessions.iter().any(|s| s.id == "session-1"));
        assert!(sessions.iter().any(|s| s.id == "session-3"));
    }

    // ==================== list_sessions_from_jsonl tests ====================

    #[test]
    fn test_list_sessions_from_jsonl_empty_file() {
        let temp_file = NamedTempFile::new().expect("temp file");
        let jsonl_path = temp_file.path();

        let sessions = list_sessions_from_jsonl(jsonl_path).expect("should list");

        assert!(sessions.is_empty());
    }

    #[test]
    fn test_list_sessions_from_jsonl_nonexistent_file() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let jsonl_path = temp_dir.path().join("nonexistent.jsonl");

        let sessions = list_sessions_from_jsonl(&jsonl_path).expect("should list");

        assert!(sessions.is_empty());
    }

    #[test]
    fn test_list_sessions_from_jsonl_skips_empty_lines() {
        let mut temp_file = NamedTempFile::new().expect("temp file");

        let session = create_test_session("test-session");
        let line = session_to_jsonl_line(&session).expect("should serialize");

        use std::io::Write;
        writeln!(temp_file).expect("write empty line");
        writeln!(temp_file, "{}", line).expect("write session");
        writeln!(temp_file, "   ").expect("write whitespace");
        temp_file.flush().expect("flush");

        // Get path after writing to avoid borrow issues
        let jsonl_path = temp_file.path();
        let sessions = list_sessions_from_jsonl(jsonl_path).expect("should list");
        assert_eq!(sessions.len(), 1);
    }

    #[test]
    fn test_list_sessions_from_jsonl_invalid_json() {
        let mut temp_file = NamedTempFile::new().expect("temp file");

        use std::io::Write;
        // Write invalid JSON (not valid JSON structure)
        writeln!(temp_file, "{{invalid json}}").expect("write invalid");
        temp_file.flush().expect("flush");

        // Get path after writing to avoid borrow issues
        let jsonl_path = temp_file.path();
        let result = list_sessions_from_jsonl(jsonl_path);
        assert!(matches!(result, Err(StorageError::InvalidJsonLine { line: 1, .. })));
    }

    // ==================== get_session_from_jsonl tests ====================

    #[test]
    fn test_get_session_from_jsonl_found() {
        let temp_file = NamedTempFile::new().expect("temp file");
        let jsonl_path = temp_file.path();

        let session1 = create_test_session("session-1");
        let session2 = create_test_session("session-2");

        append_session_to_jsonl(&session1, jsonl_path).expect("should append 1");
        append_session_to_jsonl(&session2, jsonl_path).expect("should append 2");

        let result = get_session_from_jsonl(jsonl_path, "session-2");

        assert!(result.is_ok());
        let found = result.expect("should find");
        assert_eq!(found.id, "session-2");
    }

    #[test]
    fn test_get_session_from_jsonl_not_found() {
        let temp_file = NamedTempFile::new().expect("temp file");
        let jsonl_path = temp_file.path();

        let session = create_test_session("session-1");
        append_session_to_jsonl(&session, jsonl_path).expect("should append");

        let result = get_session_from_jsonl(jsonl_path, "nonexistent");

        assert!(matches!(result, Err(StorageError::SessionNotFound(id)) if id == "nonexistent"));
    }

    #[test]
    fn test_get_session_from_jsonl_empty_file() {
        let temp_file = NamedTempFile::new().expect("temp file");
        let jsonl_path = temp_file.path();

        let result = get_session_from_jsonl(jsonl_path, "any-id");

        assert!(matches!(result, Err(StorageError::SessionNotFound(_))));
    }

    // ==================== create_snapshot tests ====================

    #[test]
    fn test_create_snapshot_basic() {
        let session = create_test_session("snap-test");
        let snapshot = create_snapshot(&session, "Test snapshot");

        assert_eq!(snapshot.session_id, "snap-test");
        assert!(snapshot.snapshot_id.starts_with("snap-test-"));
        assert_eq!(snapshot.description, "Test snapshot");
        assert_eq!(snapshot.stage, "discovery");
        assert_eq!(snapshot.gaps_count, 0);
        assert_eq!(snapshot.conflicts_count, 0);
    }

    #[test]
    fn test_create_snapshot_with_answers() {
        let session = create_test_session_with_answers("snap-answers");
        let snapshot = create_snapshot(&session, "With answers");

        assert_eq!(snapshot.answers.len(), 2);
        assert_eq!(
            snapshot.answers.get("q1"),
            Some(&"REST API for users".to_string())
        );
        assert_eq!(
            snapshot.answers.get("q2"),
            Some(&"Bearer token".to_string())
        );
    }

    #[test]
    fn test_create_snapshot_with_gaps_and_conflicts() {
        let mut session = create_test_session("snap-gaps");
        session.gaps.push(Gap {
            id: "gap-1".into(),
            field: "test".into(),
            ..Gap::default()
        });
        session.gaps.push(Gap {
            id: "gap-2".into(),
            field: "test2".into(),
            ..Gap::default()
        });

        let snapshot = create_snapshot(&session, "With gaps");
        assert_eq!(snapshot.gaps_count, 2);
    }

    #[test]
    fn test_create_snapshot_serializable() {
        let session = create_test_session_with_answers("snap-serialize");
        let snapshot = create_snapshot(&session, "Serializable");

        let json = serde_json::to_string(&snapshot).expect("should serialize");
        let parsed: SessionSnapshot = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(snapshot, parsed);
    }

    // ==================== append_to_history tests ====================

    #[test]
    fn test_append_to_history_creates_file() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let history_path = temp_dir.path().join("history.jsonl");

        let session = create_test_session("history-1");
        let result = append_to_history(&session, "Initial state", &history_path);

        assert!(result.is_ok());
        assert!(history_path.exists());
    }

    #[test]
    fn test_append_to_history_creates_parent_dirs() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let history_path = temp_dir.path().join("nested/history.jsonl");

        let session = create_test_session("history-nested");
        let result = append_to_history(&session, "Nested", &history_path);

        assert!(result.is_ok());
        assert!(history_path.exists());
    }

    #[test]
    fn test_append_to_history_multiple_snapshots() {
        let temp_file = NamedTempFile::new().expect("temp file");
        let history_path = temp_file.path();

        let mut session = create_test_session("multi-history");
        append_to_history(&session, "State 1", history_path).expect("append 1");

        session.raw_notes = "Updated".to_string();
        append_to_history(&session, "State 2", history_path).expect("append 2");

        let snapshots =
            list_session_history(history_path, "multi-history").expect("list history");
        assert_eq!(snapshots.len(), 2);
    }

    #[test]
    fn test_append_to_history_different_sessions() {
        let temp_file = NamedTempFile::new().expect("temp file");
        let history_path = temp_file.path();

        let session1 = create_test_session("session-a");
        let session2 = create_test_session("session-b");

        append_to_history(&session1, "A snapshot", history_path).expect("append a");
        append_to_history(&session2, "B snapshot", history_path).expect("append b");

        let snapshots_a = list_session_history(history_path, "session-a").expect("list a");
        let snapshots_b = list_session_history(history_path, "session-b").expect("list b");

        assert_eq!(snapshots_a.len(), 1);
        assert_eq!(snapshots_b.len(), 1);
        assert_eq!(snapshots_a[0].description, "A snapshot");
        assert_eq!(snapshots_b[0].description, "B snapshot");
    }

    // ==================== list_session_history tests ====================

    #[test]
    fn test_list_session_history_empty_file() {
        let temp_file = NamedTempFile::new().expect("temp file");
        let history_path = temp_file.path();

        let snapshots = list_session_history(history_path, "any-id").expect("should list");

        assert!(snapshots.is_empty());
    }

    #[test]
    fn test_list_session_history_nonexistent_file() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let history_path = temp_dir.path().join("nonexistent.jsonl");

        let snapshots = list_session_history(&history_path, "any-id").expect("should list");

        assert!(snapshots.is_empty());
    }

    #[test]
    fn test_list_session_history_filters_by_session_id() {
        let temp_file = NamedTempFile::new().expect("temp file");
        let history_path = temp_file.path();

        let session1 = create_test_session("filter-test-1");
        let session2 = create_test_session("filter-test-2");

        append_to_history(&session1, "Snapshot 1", history_path).expect("append 1");
        append_to_history(&session2, "Snapshot 2", history_path).expect("append 2");
        append_to_history(&session1, "Snapshot 3", history_path).expect("append 3");

        let snapshots =
            list_session_history(history_path, "filter-test-1").expect("should list");

        assert_eq!(snapshots.len(), 2);
        assert!(snapshots.iter().all(|s| s.session_id == "filter-test-1"));
    }

    #[test]
    fn test_list_session_history_invalid_json() {
        let mut temp_file = NamedTempFile::new().expect("temp file");

        use std::io::Write;
        // Write invalid JSON (not valid JSON structure)
        writeln!(temp_file, "{{invalid}}").expect("write invalid");
        temp_file.flush().expect("flush");

        // Get path after writing to avoid borrow issues
        let history_path = temp_file.path();
        let result = list_session_history(history_path, "any-id");
        assert!(matches!(result, Err(StorageError::InvalidJsonLine { line: 1, .. })));
    }

    // ==================== StorageError tests ====================

    #[test]
    fn test_storage_error_display() {
        assert_eq!(
            StorageError::IoError("file not found".into()).to_string(),
            "I/O error: file not found"
        );

        assert_eq!(
            StorageError::JsonError("parse error".into()).to_string(),
            "JSON error: parse error"
        );

        assert_eq!(
            StorageError::SessionNotFound("sess-123".into()).to_string(),
            "session not found: sess-123"
        );

        let invalid_line = StorageError::InvalidJsonLine {
            line: 5,
            error: "unexpected token".into(),
        };
        assert_eq!(
            invalid_line.to_string(),
            "invalid JSON on line 5: unexpected token"
        );

        assert_eq!(
            StorageError::DirectoryCreationFailed("permission denied".into()).to_string(),
            "directory creation failed: permission denied"
        );
    }

    #[test]
    fn test_storage_error_clone_and_eq() {
        let err1 = StorageError::SessionNotFound("test".into());
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    // ==================== diff_sessions tests (WP16) ====================

    #[test]
    fn test_diff_sessions_no_changes() {
        let session1 = create_test_session("sess-1");
        let session2 = session1.clone();

        let diff = diff_sessions(&session1, &session2);

        assert_eq!(diff.from_session_id, "sess-1");
        assert_eq!(diff.to_session_id, "sess-1");
        assert!(!diff.stage_changed);
        assert!(diff.answers_added.is_empty());
        assert!(diff.answers_modified.is_empty());
        assert!(diff.answers_removed.is_empty());
        assert_eq!(diff.gaps_added, 0);
        assert_eq!(diff.conflicts_added, 0);
    }

    #[test]
    fn test_diff_sessions_answers_added() {
        let session1 = create_test_session("sess-1");
        let session2 = create_test_session_with_answers("sess-1");

        let diff = diff_sessions(&session1, &session2);

        assert_eq!(diff.answers_added.len(), 2);
        assert!(diff.answers_modified.is_empty());
        assert!(diff.answers_removed.is_empty());

        // Verify first added answer
        let first_added = &diff.answers_added[0];
        assert_eq!(first_added.question_id, "q1");
        assert_eq!(first_added.change_type, AnswerChangeType::Added);
        assert!(first_added.old_response.is_none());
        assert_eq!(first_added.new_response, Some("REST API for users".to_string()));
    }

    #[test]
    fn test_diff_sessions_answers_modified() {
        let session1 = create_test_session_with_answers("sess-1");
        let mut session2 = session1.clone();

        // Modify an answer in session2
        session2.answers[0].response = "GraphQL API for users".to_string();
        session2.updated_at = "2026-02-27T01:00:00Z".to_string();

        let diff = diff_sessions(&session1, &session2);

        assert!(diff.answers_added.is_empty());
        assert_eq!(diff.answers_modified.len(), 1);
        assert!(diff.answers_removed.is_empty());

        let modified = &diff.answers_modified[0];
        assert_eq!(modified.question_id, "q1");
        assert_eq!(modified.change_type, AnswerChangeType::Modified);
        assert_eq!(modified.old_response, Some("REST API for users".to_string()));
        assert_eq!(modified.new_response, Some("GraphQL API for users".to_string()));
    }

    #[test]
    fn test_diff_sessions_answers_removed() {
        let session1 = create_test_session_with_answers("sess-1");
        let session2 = create_test_session("sess-1");

        let diff = diff_sessions(&session1, &session2);

        assert!(diff.answers_added.is_empty());
        assert!(diff.answers_modified.is_empty());
        assert_eq!(diff.answers_removed.len(), 2);

        // Verify removed answers
        let removed_ids: Vec<&str> = diff.answers_removed
            .iter()
            .map(|a| a.question_id.as_str())
            .collect();
        assert!(removed_ids.contains(&"q1"));
        assert!(removed_ids.contains(&"q2"));
    }

    #[test]
    fn test_diff_sessions_stage_changed() {
        let mut session1 = create_test_session("sess-1");
        let mut session2 = session1.clone();

        session1.stage = InterviewStage::Discovery;
        session2.stage = InterviewStage::Refinement;

        let diff = diff_sessions(&session1, &session2);

        assert!(diff.stage_changed);
        assert_eq!(diff.old_stage, Some("discovery".to_string()));
        assert_eq!(diff.new_stage, Some("refinement".to_string()));
    }

    #[test]
    fn test_diff_sessions_gaps_added() {
        let mut session1 = create_test_session("sess-1");
        let mut session2 = session1.clone();

        session1.gaps.clear();
        session2.gaps.push(Gap {
            id: "gap-1".into(),
            field: "test".into(),
            ..Gap::default()
        });
        session2.gaps.push(Gap {
            id: "gap-2".into(),
            field: "test2".into(),
            ..Gap::default()
        });

        let diff = diff_sessions(&session1, &session2);

        assert_eq!(diff.gaps_added, 2);
    }

    #[test]
    fn test_diff_sessions_gaps_resolved() {
        let mut session1 = create_test_session("sess-1");
        let mut session2 = session1.clone();

        session1.gaps.push(Gap {
            id: "gap-1".into(),
            field: "test".into(),
            ..Gap::default()
        });
        session1.gaps.push(Gap {
            id: "gap-2".into(),
            field: "test2".into(),
            ..Gap::default()
        });
        session2.gaps.clear();

        let diff = diff_sessions(&session1, &session2);

        assert_eq!(diff.gaps_added, -2);
    }

    #[test]
    fn test_diff_sessions_conflicts_added() {
        use crate::intent::interview::types::{Conflict, ConflictResolution};

        let mut session1 = create_test_session("sess-1");
        let mut session2 = session1.clone();

        session1.conflicts.clear();
        session2.conflicts.push(Conflict {
            id: "conflict-1".into(),
            between: ("a".into(), "b".into()),
            description: "test".into(),
            impact: "test".into(),
            options: vec![ConflictResolution::default()],
            chosen: None,
        });

        let diff = diff_sessions(&session1, &session2);

        assert_eq!(diff.conflicts_added, 1);
    }

    #[test]
    fn test_diff_sessions_conflicts_resolved() {
        use crate::intent::interview::types::{Conflict, ConflictResolution};

        let mut session1 = create_test_session("sess-1");
        let mut session2 = session1.clone();

        session1.conflicts.push(Conflict {
            id: "conflict-1".into(),
            between: ("a".into(), "b".into()),
            description: "test".into(),
            impact: "test".into(),
            options: vec![ConflictResolution::default()],
            chosen: None,
        });
        session2.conflicts.clear();

        let diff = diff_sessions(&session1, &session2);

        assert_eq!(diff.conflicts_added, -1);
    }

    #[test]
    fn test_diff_sessions_complex_changes() {
        let mut session1 = create_test_session_with_answers("sess-1");
        let mut session2 = session1.clone();

        // Add a new answer
        session2.answers.push(Answer {
            question_id: "q3".to_string(),
            question_text: "What is rate limiting?".to_string(),
            response: "100 req/min".to_string(),
            ..Answer::default()
        });

        // Modify an existing answer
        session2.answers.iter_mut().find(|a| a.question_id == "q1").map(|a| {
            a.response = "GraphQL API".to_string();
            a
        });

        // Remove an answer (by not including it in the clone, we need to do this differently)
        session1.answers.push(Answer {
            question_id: "q4".to_string(),
            question_text: "To be removed".to_string(),
            response: "old answer".to_string(),
            ..Answer::default()
        });

        // Add a gap
        session2.gaps.push(Gap {
            id: "gap-new".into(),
            field: "rate_limit".into(),
            ..Gap::default()
        });

        // Change stage
        session2.stage = InterviewStage::Refinement;

        let diff = diff_sessions(&session1, &session2);

        // 1 added (q3), 1 modified (q1), 1 removed (q4)
        assert_eq!(diff.answers_added.len(), 1);
        assert_eq!(diff.answers_modified.len(), 1);
        assert_eq!(diff.answers_removed.len(), 1);
        assert!(diff.stage_changed);
        assert_eq!(diff.gaps_added, 1);
    }

    // ==================== format_diff tests (WP16) ====================

    #[test]
    fn test_format_diff_empty() {
        let session1 = create_test_session("sess-1");
        let session2 = session1.clone();

        let diff = diff_sessions(&session1, &session2);
        let formatted = format_diff(&diff);

        assert!(formatted.contains("Session Diff: sess-1 -> sess-1"));
        assert!(formatted.contains("Gaps: No change"));
        assert!(formatted.contains("Conflicts: No change"));
    }

    #[test]
    fn test_format_diff_with_added_answers() {
        let session1 = create_test_session("sess-1");
        let session2 = create_test_session_with_answers("sess-1");

        let diff = diff_sessions(&session1, &session2);
        let formatted = format_diff(&diff);

        assert!(formatted.contains("Answers Added (2):"));
        assert!(formatted.contains("+ [q1]"));
        assert!(formatted.contains("REST API for users"));
    }

    #[test]
    fn test_format_diff_with_modified_answers() {
        let session1 = create_test_session_with_answers("sess-1");
        let mut session2 = session1.clone();
        session2.answers[0].response = "GraphQL API".to_string();

        let diff = diff_sessions(&session1, &session2);
        let formatted = format_diff(&diff);

        assert!(formatted.contains("Answers Modified (1):"));
        assert!(formatted.contains("~ [q1]"));
        assert!(formatted.contains("REST API for users -> GraphQL API"));
    }

    #[test]
    fn test_format_diff_with_removed_answers() {
        let session1 = create_test_session_with_answers("sess-1");
        let session2 = create_test_session("sess-1");

        let diff = diff_sessions(&session1, &session2);
        let formatted = format_diff(&diff);

        assert!(formatted.contains("Answers Removed (2):"));
        assert!(formatted.contains("- [q1]"));
        assert!(formatted.contains("- [q2]"));
    }

    #[test]
    fn test_format_diff_with_stage_change() {
        let mut session1 = create_test_session("sess-1");
        let mut session2 = session1.clone();
        session1.stage = InterviewStage::Discovery;
        session2.stage = InterviewStage::Complete;

        let diff = diff_sessions(&session1, &session2);
        let formatted = format_diff(&diff);

        assert!(formatted.contains("Stage: discovery -> complete"));
    }

    #[test]
    fn test_format_diff_with_gaps_positive() {
        let mut session1 = create_test_session("sess-1");
        let mut session2 = session1.clone();
        session1.gaps.clear();
        session2.gaps.push(Gap::default());
        session2.gaps.push(Gap::default());
        session2.gaps.push(Gap::default());

        let diff = diff_sessions(&session1, &session2);
        let formatted = format_diff(&diff);

        assert!(formatted.contains("Gaps: +3 new gap(s)"));
    }

    #[test]
    fn test_format_diff_with_gaps_negative() {
        let mut session1 = create_test_session("sess-1");
        let mut session2 = session1.clone();
        session1.gaps.push(Gap::default());
        session1.gaps.push(Gap::default());
        session2.gaps.clear();

        let diff = diff_sessions(&session1, &session2);
        let formatted = format_diff(&diff);

        assert!(formatted.contains("Gaps: 2 gap(s) resolved"));
    }

    #[test]
    fn test_format_diff_with_conflicts_positive() {
        use crate::intent::interview::types::{Conflict, ConflictResolution};

        let mut session1 = create_test_session("sess-1");
        let mut session2 = session1.clone();
        session1.conflicts.clear();
        session2.conflicts.push(Conflict {
            id: "c1".into(),
            between: ("a".into(), "b".into()),
            description: "test".into(),
            impact: "test".into(),
            options: vec![ConflictResolution::default()],
            chosen: None,
        });

        let diff = diff_sessions(&session1, &session2);
        let formatted = format_diff(&diff);

        assert!(formatted.contains("Conflicts: +1 new conflict(s)"));
    }

    #[test]
    fn test_format_diff_with_conflicts_negative() {
        use crate::intent::interview::types::{Conflict, ConflictResolution};

        let mut session1 = create_test_session("sess-1");
        let mut session2 = session1.clone();
        session1.conflicts.push(Conflict {
            id: "c1".into(),
            between: ("a".into(), "b".into()),
            description: "test".into(),
            impact: "test".into(),
            options: vec![ConflictResolution::default()],
            chosen: None,
        });
        session2.conflicts.clear();

        let diff = diff_sessions(&session1, &session2);
        let formatted = format_diff(&diff);

        assert!(formatted.contains("Conflicts: 1 conflict(s) resolved"));
    }

    #[test]
    fn test_format_diff_truncates_long_responses() {
        let session1 = create_test_session("sess-1");
        let mut session2 = session1.clone();

        // Add answer with very long response (> 50 chars)
        session2.answers.push(Answer {
            question_id: "q-long".to_string(),
            question_text: "Short".to_string(),
            response: "This is a very long response that should be truncated because it exceeds fifty characters".to_string(),
            ..Answer::default()
        });

        let diff = diff_sessions(&session1, &session2);
        let formatted = format_diff(&diff);

        // Response should be truncated to 50 chars + "..."
        assert!(formatted.contains("This is a very long response that should be trunca..."));
        // Full response should NOT appear
        assert!(!formatted.contains("exceeds fifty characters"));
    }

    #[test]
    fn test_format_diff_truncates_long_question_text() {
        let session1 = create_test_session("sess-1");
        let mut session2 = session1.clone();

        // Add answer with very long question text (> 50 chars)
        session2.answers.push(Answer {
            question_id: "q-long".to_string(),
            question_text: "This is a very long question text that should be truncated".to_string(),
            response: "Short response".to_string(),
            ..Answer::default()
        });

        let diff = diff_sessions(&session1, &session2);
        let formatted = format_diff(&diff);

        // Question text should be truncated to 50 chars + "..."
        assert!(formatted.contains("This is a very long question text that should be t..."));
    }

    // ==================== diff_snapshots tests (WP16) ====================

    #[test]
    fn test_diff_snapshots_no_changes() {
        let session = create_test_session("sess-1");
        let snapshot1 = create_snapshot(&session, "State 1");
        let snapshot2 = snapshot1.clone();

        let diff = diff_snapshots(&snapshot1, &snapshot2);

        assert_eq!(diff.from_session_id, "sess-1");
        assert_eq!(diff.to_session_id, "sess-1");
        assert!(!diff.stage_changed);
        assert!(diff.answers_added.is_empty());
        assert!(diff.answers_modified.is_empty());
        assert!(diff.answers_removed.is_empty());
        assert_eq!(diff.gaps_added, 0);
        assert_eq!(diff.conflicts_added, 0);
    }

    #[test]
    fn test_diff_snapshots_answers_added() {
        let session1 = create_test_session("sess-1");
        let session2 = create_test_session_with_answers("sess-1");

        let snapshot1 = create_snapshot(&session1, "Empty");
        let snapshot2 = create_snapshot(&session2, "With answers");

        let diff = diff_snapshots(&snapshot1, &snapshot2);

        assert_eq!(diff.answers_added.len(), 2);
        assert!(diff.answers_modified.is_empty());
        assert!(diff.answers_removed.is_empty());
    }

    #[test]
    fn test_diff_snapshots_answers_modified() {
        let session1 = create_test_session_with_answers("sess-1");
        let mut session2 = session1.clone();
        session2.answers[0].response = "GraphQL API".to_string();

        let snapshot1 = create_snapshot(&session1, "State 1");
        let snapshot2 = create_snapshot(&session2, "State 2");

        let diff = diff_snapshots(&snapshot1, &snapshot2);

        assert!(diff.answers_added.is_empty());
        assert_eq!(diff.answers_modified.len(), 1);
        assert!(diff.answers_removed.is_empty());

        let modified = &diff.answers_modified[0];
        assert_eq!(modified.question_id, "q1");
        assert_eq!(modified.old_response, Some("REST API for users".to_string()));
        assert_eq!(modified.new_response, Some("GraphQL API".to_string()));
    }

    #[test]
    fn test_diff_snapshots_answers_removed() {
        let session1 = create_test_session_with_answers("sess-1");
        let session2 = create_test_session("sess-1");

        let snapshot1 = create_snapshot(&session1, "With answers");
        let snapshot2 = create_snapshot(&session2, "Empty");

        let diff = diff_snapshots(&snapshot1, &snapshot2);

        assert!(diff.answers_added.is_empty());
        assert!(diff.answers_modified.is_empty());
        assert_eq!(diff.answers_removed.len(), 2);
    }

    #[test]
    fn test_diff_snapshots_stage_changed() {
        let mut session1 = create_test_session("sess-1");
        let mut session2 = session1.clone();
        session1.stage = InterviewStage::Discovery;
        session2.stage = InterviewStage::Complete;

        let snapshot1 = create_snapshot(&session1, "Discovery");
        let snapshot2 = create_snapshot(&session2, "Complete");

        let diff = diff_snapshots(&snapshot1, &snapshot2);

        assert!(diff.stage_changed);
        assert_eq!(diff.old_stage, Some("discovery".to_string()));
        assert_eq!(diff.new_stage, Some("complete".to_string()));
    }

    #[test]
    fn test_diff_snapshots_gaps_changed() {
        let mut session1 = create_test_session("sess-1");
        let mut session2 = session1.clone();

        session1.gaps.push(Gap::default());
        session2.gaps.push(Gap::default());
        session2.gaps.push(Gap::default());
        session2.gaps.push(Gap::default());

        let snapshot1 = create_snapshot(&session1, "1 gap");
        let snapshot2 = create_snapshot(&session2, "3 gaps");

        let diff = diff_snapshots(&snapshot1, &snapshot2);

        assert_eq!(diff.gaps_added, 2);
    }

    #[test]
    fn test_diff_snapshots_conflicts_changed() {
        use crate::intent::interview::types::{Conflict, ConflictResolution};

        let mut session1 = create_test_session("sess-1");
        let mut session2 = session1.clone();

        session1.conflicts.push(Conflict {
            id: "c1".into(),
            between: ("a".into(), "b".into()),
            description: "test".into(),
            impact: "test".into(),
            options: vec![ConflictResolution::default()],
            chosen: None,
        });
        session1.conflicts.push(Conflict {
            id: "c2".into(),
            between: ("c".into(), "d".into()),
            description: "test".into(),
            impact: "test".into(),
            options: vec![ConflictResolution::default()],
            chosen: None,
        });
        session2.conflicts.clear();

        let snapshot1 = create_snapshot(&session1, "2 conflicts");
        let snapshot2 = create_snapshot(&session2, "0 conflicts");

        let diff = diff_snapshots(&snapshot1, &snapshot2);

        assert_eq!(diff.conflicts_added, -2);
    }

    // ==================== AnswerChangeType and AnswerDiff serde tests ====================

    #[test]
    fn test_answer_change_type_serde_roundtrip() {
        let types = [AnswerChangeType::Added, AnswerChangeType::Modified, AnswerChangeType::Removed];

        for change_type in types {
            let json = serde_json::to_string(&change_type).expect("should serialize");
            let parsed: AnswerChangeType = serde_json::from_str(&json).expect("should deserialize");
            assert_eq!(change_type, parsed);
        }
    }

    #[test]
    fn test_answer_diff_serde_roundtrip() {
        let diff = AnswerDiff {
            question_id: "q1".to_string(),
            question_text: "What is the API?".to_string(),
            old_response: Some("REST".to_string()),
            new_response: Some("GraphQL".to_string()),
            change_type: AnswerChangeType::Modified,
        };

        let json = serde_json::to_string(&diff).expect("should serialize");
        let parsed: AnswerDiff = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(diff, parsed);
    }

    #[test]
    fn test_session_diff_serde_roundtrip() {
        let session1 = create_test_session("sess-1");
        let session2 = create_test_session_with_answers("sess-1");

        let diff = diff_sessions(&session1, &session2);

        let json = serde_json::to_string(&diff).expect("should serialize");
        let parsed: SessionDiff = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(diff, parsed);
    }
}
