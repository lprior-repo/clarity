use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
