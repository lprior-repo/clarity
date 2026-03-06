use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::InterviewStage;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProfileParseError {
  #[error("unknown profile type: {input}")]
  UnknownProfile { input: String },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InterviewError {
  #[error("gap ID is empty")]
  EmptyGapId,
  #[error("resolution text is empty")]
  EmptyResolution,
  #[error("gap not found: {0}")]
  GapNotFound(String),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ConflictDetectionError {
  #[error("session ID is empty")]
  EmptySessionId,
  #[error("conflict ID is empty")]
  EmptyConflictId,
  #[error("conflict not found: {0}")]
  ConflictNotFound(String),
  #[error("conflict already resolved: {0}")]
  ConflictAlreadyResolved(String),
  #[error("invalid option index {index} for conflict {conflict_id} (has {option_count} options)")]
  InvalidOptionIndex {
    conflict_id: String,
    index: i32,
    option_count: usize,
  },
  #[error("option index cannot be negative: {0}")]
  NegativeOptionIndex(i32),
  #[error("answer has empty question_id at index {0}")]
  EmptyQuestionId(usize),
}

#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterviewSessionError {
  #[error("answer round {answer_round} does not match current round {current_round}")]
  RoundMismatch {
    answer_round: u32,
    current_round: u32,
  },
  #[error("cannot modify session in {stage:?} state")]
  SessionNotModifiable { stage: InterviewStage },
  #[error("session is paused; call resume before modifying")]
  SessionPaused,
  #[error("invalid phase number: {phase_number}; phase must be >= 1")]
  InvalidPhaseNumber { phase_number: u32 },
  #[error("answer has empty question_id")]
  EmptyQuestionId,
  #[error("timestamp cannot be empty")]
  EmptyTimestamp,
  #[error("duplicate answer for question '{question_id}' in round {round}")]
  DuplicateAnswer { question_id: String, round: u32 },
  #[error("cannot proceed: {count} blocking gap(s) unresolved")]
  BlockingGapsUnresolved { count: usize, gap_ids: Vec<String> },
  #[error("session already complete")]
  AlreadyComplete,
}
