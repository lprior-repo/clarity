//! Error types for interview session operations.
//!
//! This module defines the error types used throughout the interview system:
//!
//! - [`ProfileParseError`]: Errors parsing profile strings
//! - [`InterviewError`]: General interview errors (gap operations)
//! - [`ConflictDetectionError`]: Conflict detection and resolution errors
//! - [`InterviewSessionError`]: Session operation errors (phases, rounds, answers)

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::InterviewStage;

/// Errors that can occur when parsing a profile string.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProfileParseError {
  /// The input string does not match any known profile type.
  #[error("unknown profile type: {input}")]
  UnknownProfile { input: String },
}

/// General interview errors for gap operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InterviewError {
  /// The gap ID provided is empty or contains only whitespace.
  #[error("gap ID is empty")]
  EmptyGapId,
  /// The resolution text is empty or contains only whitespace.
  #[error("resolution text is empty")]
  EmptyResolution,
  /// No gap was found with the specified ID.
  #[error("gap not found: {0}")]
  GapNotFound(String),
}

/// Errors for conflict detection and resolution operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ConflictDetectionError {
  /// The session ID is empty.
  #[error("session ID is empty")]
  EmptySessionId,
  /// The conflict ID provided is empty or contains only whitespace.
  #[error("conflict ID is empty")]
  EmptyConflictId,
  /// No conflict was found with the specified ID.
  #[error("conflict not found: {0}")]
  ConflictNotFound(String),
  /// The conflict has already been resolved.
  #[error("conflict already resolved: {0}")]
  ConflictAlreadyResolved(String),
  /// The chosen option index is out of bounds for the conflict's options.
  #[error("invalid option index {index} for conflict {conflict_id} (has {option_count} options)")]
  InvalidOptionIndex {
    conflict_id: String,
    index: i32,
    option_count: usize,
  },
  /// The option index is negative.
  #[error("option index cannot be negative: {0}")]
  NegativeOptionIndex(i32),
  /// An answer has an empty question_id at the specified index.
  #[error("answer has empty question_id at index {0}")]
  EmptyQuestionId(usize),
  /// Cannot resolve a conflict that has no options.
  #[error("cannot resolve conflict with no options")]
  EmptyOptions,
}

/// Errors for interview session operations.
///
/// These errors are returned by methods on [`InterviewSession`](super::InterviewSession).
#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterviewSessionError {
  /// The answer's round does not match the session's current round.
  #[error("answer round {answer_round} does not match current round {current_round}")]
  RoundMismatch {
    answer_round: u32,
    current_round: u32,
  },
  /// The session is in a state that does not allow modifications.
  #[error("cannot modify session in {stage:?} state")]
  SessionNotModifiable { stage: InterviewStage },
  /// The session is paused; resume before modifying.
  #[error("session is paused; call resume before modifying")]
  SessionPaused,
  /// The phase number is invalid (must be >= 1).
  #[error("invalid phase number: {phase_number}; phase must be >= 1")]
  InvalidPhaseNumber { phase_number: u32 },
  /// The answer has an empty question_id.
  #[error("answer has empty question_id")]
  EmptyQuestionId,
  /// The timestamp cannot be empty.
  #[error("timestamp cannot be empty")]
  EmptyTimestamp,
  /// A duplicate answer was detected for the same question in the same round.
  #[error("duplicate answer for question '{question_id}' in round {round}")]
  DuplicateAnswer { question_id: String, round: u32 },
  /// Cannot proceed because blocking gaps are unresolved.
  #[error("cannot proceed: {count} blocking gap(s) unresolved")]
  BlockingGapsUnresolved { count: usize, gap_ids: Vec<String> },
  /// The session is already complete.
  #[error("session already complete")]
  AlreadyComplete,
  /// No answer was found for the specified question.
  #[error("answer not found for question: {0}")]
  AnswerNotFound(String),
}
