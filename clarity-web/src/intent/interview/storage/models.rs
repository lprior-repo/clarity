use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::intent::interview::types::Answer;

/// Snapshot for history tracking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSnapshot {
  /// ID of the session this snapshot belongs to.
  pub session_id: String,
  /// Unique identifier for this snapshot.
  pub snapshot_id: String,
  /// When this snapshot was taken (ISO 8601).
  pub timestamp: String,
  /// Human-readable description of the snapshot.
  pub description: String,
  /// Current answers at snapshot time (`question_id` -> response).
  pub answers: HashMap<String, String>,
  /// Number of gaps at snapshot time.
  pub gaps_count: usize,
  /// Number of conflicts at snapshot time.
  pub conflicts_count: usize,
  /// Session stage at snapshot time.
  pub stage: String,
}

/// Change type for answer differences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnswerChangeType {
  /// Answer was added (not present in "from" session).
  Added,
  /// Answer was modified (present in both but response differs).
  Modified,
  /// Answer was removed (not present in "to" session).
  Removed,
}

/// Difference for a single answer between two sessions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnswerDiff {
  /// Question identifier.
  pub question_id: String,
  /// The question text for context.
  pub question_text: String,
  /// Response in the "from" session (None if added).
  pub old_response: Option<String>,
  /// Response in the "to" session (None if removed).
  pub new_response: Option<String>,
  /// Type of change that occurred.
  pub change_type: AnswerChangeType,
}

/// Difference between two interview sessions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionDiff {
  /// ID of the "from" session.
  pub from_session_id: String,
  /// ID of the "to" session.
  pub to_session_id: String,
  /// Timestamp of the "from" session.
  pub from_timestamp: String,
  /// Timestamp of the "to" session.
  pub to_timestamp: String,
  /// Whether the stage changed between sessions.
  pub stage_changed: bool,
  /// Stage in the "from" session (None if not applicable).
  pub old_stage: Option<String>,
  /// Stage in the "to" session (None if not applicable).
  pub new_stage: Option<String>,
  /// Answers that were added.
  pub answers_added: Vec<AnswerDiff>,
  /// Answers that were modified.
  pub answers_modified: Vec<AnswerDiff>,
  /// Answers that were removed.
  pub answers_removed: Vec<AnswerDiff>,
  /// Change in gaps count (positive = new gaps, negative = resolved).
  pub gaps_added: i32,
  /// Change in conflicts count (positive = new conflicts, negative = resolved).
  pub conflicts_added: i32,
}

/// Versioned answer for tracking changes over time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnswerVersion {
  /// Version number, incrementing for each update.
  pub version: u32,
  /// The answer text response.
  pub response: String,
  /// Question identifier this answer belongs to.
  pub question_id: String,
  /// Reason for this version change (e.g., "initial", "correction", "update").
  pub change_reason: String,
  /// Timestamp when this version was created (ISO 8601).
  pub timestamp: String,
}

impl AnswerVersion {
  /// Create a new answer version.
  #[must_use]
  pub fn new(
    version: u32,
    response: String,
    question_id: String,
    change_reason: String,
    timestamp: String,
  ) -> Self {
    Self {
      version,
      response,
      question_id,
      change_reason,
      timestamp,
    }
  }
}

/// Collection of all versions of an answer, supporting version history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnswerWithHistory {
  /// Question identifier this answer belongs to.
  pub question_id: String,
  /// All versions of this answer, in chronological order.
  pub versions: Vec<AnswerVersion>,
}

impl AnswerWithHistory {
  /// Create a new answer with history.
  #[must_use]
  pub fn new(
    question_id: impl Into<String>,
    response: impl Into<String>,
    change_reason: impl Into<String>,
  ) -> Self {
    let question_id = question_id.into();
    let version = AnswerVersion::new(
      1,
      response.into(),
      question_id.clone(),
      change_reason.into(),
      chrono::Utc::now().to_rfc3339(),
    );
    Self {
      question_id,
      versions: vec![version],
    }
  }

  /// Add a new version to the history.
  pub fn add_version(&mut self, response: impl Into<String>, change_reason: impl Into<String>) {
    let version_num = self.versions.len() as u32 + 1;
    self.versions.push(AnswerVersion::new(
      version_num,
      response.into(),
      self.question_id.clone(),
      change_reason.into(),
      chrono::Utc::now().to_rfc3339(),
    ));
  }

  /// Get the current (latest) version, if any.
  #[must_use]
  pub fn current(&self) -> Option<&AnswerVersion> {
    self.versions.last()
  }

  /// Get a specific version by index (0-indexed).
  #[must_use]
  pub fn get_version(&self, index: usize) -> Option<&AnswerVersion> {
    self.versions.get(index)
  }

  /// Get the total number of versions.
  #[must_use]
  pub fn len(&self) -> usize {
    self.versions.len()
  }

  /// Check if there are no versions.
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.versions.is_empty()
  }
}

/// Convert an Answer to an AnswerVersion for tracking changes.
///
/// # Errors
/// Returns an error if the answer's question_id or response is empty.
pub fn answer_to_version(answer: &Answer, change_reason: &str) -> Result<AnswerVersion, String> {
  if answer.question_id.is_empty() {
    return Err("Answer must have a non-empty question_id".to_string());
  }
  if answer.response.is_empty() {
    return Err("Answer must have a non-empty response".to_string());
  }

  Ok(AnswerVersion::new(
    1,
    answer.response.clone(),
    answer.question_id.clone(),
    change_reason.to_string(),
    answer.timestamp.clone(),
  ))
}
