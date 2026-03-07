#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![deny(unreachable_patterns)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use thiserror::Error;

use super::boundary::{parse_bead_id, parse_notes};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FeedbackError {
  #[error("bead not found: {0}")]
  BeadNotFound(String),
  #[error("invalid status transition from {from:?} to {to:?}")]
  InvalidTransition { from: BeadStatus, to: BeadStatus },
  #[error("empty feedback: notes cannot be empty")]
  EmptyFeedback,
  #[error("bead ID cannot be empty")]
  EmptyBeadId,
  #[error("bead is already complete")]
  AlreadyComplete,
  #[error("bead is blocked: {0}")]
  Blocked(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum BeadStatus {
  #[default]
  Pending,
  Ready,
  InProgress,
  Blocked,
  Complete,
  Failed,
}

impl BeadStatus {
  #[must_use]
  pub fn can_transition_to(&self, to: &Self) -> bool {
    self == to
      || matches!(
        (self, to),
        (Self::Pending | Self::Failed | Self::Blocked, Self::Ready)
          | (
            Self::Pending | Self::Ready | Self::InProgress,
            Self::Blocked
          )
          | (Self::Ready | Self::Blocked, Self::InProgress)
          | (Self::InProgress, Self::Complete | Self::Failed)
          | (Self::Blocked | Self::Failed, Self::Pending)
      )
  }

  /// Transition to a new state with exhaustive pattern matching.
  ///
  /// # Errors
  /// Returns `FeedbackError::InvalidTransition` if the transition is not allowed.
  /// Returns `FeedbackError::AlreadyComplete` if the bead is already complete.
  pub fn transition_to(self, next: Self) -> Result<Self, FeedbackError> {
    if self == next {
      return Ok(next);
    }
    match (self, next) {
      (Self::Pending, Self::Ready | Self::Blocked)
      | (Self::Ready, Self::InProgress | Self::Blocked)
      | (Self::InProgress, Self::Complete | Self::Failed | Self::Blocked)
      | (Self::Blocked, Self::Ready | Self::Pending | Self::InProgress)
      | (Self::Failed, Self::Ready | Self::Pending) => Ok(next),
      // Complete is terminal - no transitions out
      (Self::Complete, _) => Err(FeedbackError::AlreadyComplete),
      // All remaining invalid transitions
      _ => Err(FeedbackError::InvalidTransition {
        from: self,
        to: next,
      }),
    }
  }

  #[must_use]
  pub const fn is_terminal(&self) -> bool {
    matches!(self, Self::Complete)
  }

  #[must_use]
  pub const fn is_active(&self) -> bool {
    matches!(self, Self::Ready | Self::InProgress)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeadFeedback {
  pub bead_id: String,
  pub status: BeadStatus,
  pub notes: String,
  pub timestamp: String,
  pub reviewer: Option<String>,
  pub approved: bool,
}

impl BeadFeedback {
  /// Creates a new bead feedback record.
  ///
  /// # Errors
  ///
  /// Returns `FeedbackError` if the `bead_id` or `notes` validation fails.
  #[allow(clippy::needless_pass_by_value)]
  pub fn new(
    bead_id: String,
    status: BeadStatus,
    notes: String,
    reviewer: Option<String>,
    approved: bool,
  ) -> Result<Self, FeedbackError> {
    let parsed_bead_id = parse_bead_id(&bead_id)?;
    let parsed_notes = parse_notes(&notes)?;
    Ok(Self {
      bead_id: parsed_bead_id,
      status,
      notes: parsed_notes,
      timestamp: current_timestamp(),
      reviewer,
      approved,
    })
  }

  #[must_use]
  pub fn with_reviewer(self, reviewer: String) -> Self {
    Self {
      reviewer: Some(reviewer),
      ..self
    }
  }

  #[must_use]
  pub fn with_approved(self, approved: bool) -> Self {
    Self { approved, ..self }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeadRecord {
  pub id: String,
  pub title: String,
  pub status: BeadStatus,
  #[serde(skip)]
  pub feedback_history: VecDeque<BeadFeedback>,
  pub approved: bool,
}

impl BeadRecord {
  /// Creates a new bead record.
  ///
  /// # Errors
  ///
  /// Returns `FeedbackError` if the `id` validation fails.
  #[allow(clippy::needless_pass_by_value)]
  pub fn new(id: String, title: String) -> Result<Self, FeedbackError> {
    let parsed_id = parse_bead_id(&id)?;
    Ok(Self {
      id: parsed_id,
      title,
      status: BeadStatus::Pending,
      feedback_history: VecDeque::new(),
      approved: false,
    })
  }

  #[must_use]
  pub fn can_transition_to(&self, new_status: BeadStatus) -> bool {
    self.status.can_transition_to(&new_status)
  }

  #[must_use]
  pub fn get_feedback(&self) -> Vec<&BeadFeedback> {
    self.feedback_history.iter().collect()
  }
}

pub(super) fn transition_record(
  record: &BeadRecord,
  feedback: &BeadFeedback,
) -> Result<BeadRecord, FeedbackError> {
  if record.status.is_terminal() {
    return Err(FeedbackError::AlreadyComplete);
  }
  if !record.status.can_transition_to(&feedback.status) {
    return Err(FeedbackError::InvalidTransition {
      from: record.status,
      to: feedback.status,
    });
  }

  let feedback_history = record
    .feedback_history
    .iter()
    .cloned()
    .chain(std::iter::once(feedback.clone()))
    .collect();

  Ok(BeadRecord {
    status: feedback.status,
    approved: record.approved || feedback.approved,
    feedback_history,
    ..record.clone()
  })
}

pub(super) fn current_timestamp() -> String {
  Utc::now().to_rfc3339()
}

#[cfg(test)]
#[allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::unnecessary_debug_formatting,
  clippy::match_same_arms,
  clippy::option_if_let_else,
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro,
  clippy::needless_pass_by_value
)]
mod tests {
  use super::*;

  // ============================================
  // BeadStatus (feedback) Exhaustive Transition Tests
  // ============================================

  #[test]
  fn bead_status_default_is_pending() {
    assert_eq!(BeadStatus::default(), BeadStatus::Pending);
  }

  #[test]
  fn bead_status_all_transitions_from_pending() {
    let pending = BeadStatus::Pending;

    // Valid transitions from Pending
    assert!(pending.can_transition_to(&BeadStatus::Ready));
    assert!(pending.can_transition_to(&BeadStatus::Blocked));
    assert!(pending.can_transition_to(&BeadStatus::Pending)); // no-op

    // Invalid transitions from Pending
    assert!(!pending.can_transition_to(&BeadStatus::InProgress));
    assert!(!pending.can_transition_to(&BeadStatus::Complete));
    assert!(!pending.can_transition_to(&BeadStatus::Failed));
  }

  #[test]
  fn bead_status_all_transitions_from_ready() {
    let ready = BeadStatus::Ready;

    // Valid transitions from Ready
    assert!(ready.can_transition_to(&BeadStatus::InProgress));
    assert!(ready.can_transition_to(&BeadStatus::Blocked));
    assert!(ready.can_transition_to(&BeadStatus::Ready)); // no-op

    // Invalid transitions from Ready
    assert!(!ready.can_transition_to(&BeadStatus::Pending));
    assert!(!ready.can_transition_to(&BeadStatus::Complete));
    assert!(!ready.can_transition_to(&BeadStatus::Failed));
  }

  #[test]
  fn bead_status_all_transitions_from_in_progress() {
    let in_progress = BeadStatus::InProgress;

    // Valid transitions from InProgress
    assert!(in_progress.can_transition_to(&BeadStatus::Complete));
    assert!(in_progress.can_transition_to(&BeadStatus::Failed));
    assert!(in_progress.can_transition_to(&BeadStatus::Blocked));
    assert!(in_progress.can_transition_to(&BeadStatus::InProgress)); // no-op

    // Invalid transitions from InProgress
    assert!(!in_progress.can_transition_to(&BeadStatus::Pending));
    assert!(!in_progress.can_transition_to(&BeadStatus::Ready));
  }

  #[test]
  fn bead_status_all_transitions_from_blocked() {
    let blocked = BeadStatus::Blocked;

    // Valid transitions from Blocked
    assert!(blocked.can_transition_to(&BeadStatus::Ready));
    assert!(blocked.can_transition_to(&BeadStatus::Pending));
    assert!(blocked.can_transition_to(&BeadStatus::InProgress));
    assert!(blocked.can_transition_to(&BeadStatus::Blocked)); // no-op

    // Invalid transitions from Blocked
    assert!(!blocked.can_transition_to(&BeadStatus::Complete));
    assert!(!blocked.can_transition_to(&BeadStatus::Failed));
  }

  #[test]
  fn bead_status_all_transitions_from_failed() {
    let failed = BeadStatus::Failed;

    // Valid transitions from Failed
    assert!(failed.can_transition_to(&BeadStatus::Ready));
    assert!(failed.can_transition_to(&BeadStatus::Pending));
    assert!(failed.can_transition_to(&BeadStatus::Failed)); // no-op

    // Invalid transitions from Failed
    assert!(!failed.can_transition_to(&BeadStatus::InProgress));
    assert!(!failed.can_transition_to(&BeadStatus::Blocked));
    assert!(!failed.can_transition_to(&BeadStatus::Complete));
  }

  #[test]
  fn bead_status_all_transitions_from_complete() {
    let complete = BeadStatus::Complete;

    // Complete is terminal - no transitions out except no-op
    assert!(complete.can_transition_to(&BeadStatus::Complete)); // no-op only

    // All other transitions invalid
    assert!(!complete.can_transition_to(&BeadStatus::Pending));
    assert!(!complete.can_transition_to(&BeadStatus::Ready));
    assert!(!complete.can_transition_to(&BeadStatus::InProgress));
    assert!(!complete.can_transition_to(&BeadStatus::Blocked));
    assert!(!complete.can_transition_to(&BeadStatus::Failed));
  }

  #[test]
  fn bead_status_transition_to_returns_correct_result() {
    // Valid transitions
    assert_eq!(
      BeadStatus::Pending.transition_to(BeadStatus::Ready),
      Ok(BeadStatus::Ready)
    );
    assert_eq!(
      BeadStatus::Ready.transition_to(BeadStatus::InProgress),
      Ok(BeadStatus::InProgress)
    );
    assert_eq!(
      BeadStatus::InProgress.transition_to(BeadStatus::Complete),
      Ok(BeadStatus::Complete)
    );
    assert_eq!(
      BeadStatus::InProgress.transition_to(BeadStatus::Failed),
      Ok(BeadStatus::Failed)
    );
    assert_eq!(
      BeadStatus::Failed.transition_to(BeadStatus::Ready),
      Ok(BeadStatus::Ready)
    );

    // Invalid transitions
    assert!(BeadStatus::Complete
      .transition_to(BeadStatus::Pending)
      .is_err());
    assert!(BeadStatus::Pending
      .transition_to(BeadStatus::Complete)
      .is_err());
  }

  #[test]
  fn bead_status_no_op_transitions() {
    for status in [
      BeadStatus::Pending,
      BeadStatus::Ready,
      BeadStatus::InProgress,
      BeadStatus::Complete,
      BeadStatus::Blocked,
      BeadStatus::Failed,
    ] {
      assert!(status.can_transition_to(&status));
      assert_eq!(status.transition_to(status), Ok(status));
    }
  }

  #[test]
  fn bead_status_complete_transitions_return_already_complete() {
    let complete = BeadStatus::Complete;

    assert_eq!(
      complete.transition_to(BeadStatus::Pending),
      Err(FeedbackError::AlreadyComplete)
    );
    assert_eq!(
      complete.transition_to(BeadStatus::Ready),
      Err(FeedbackError::AlreadyComplete)
    );
    assert_eq!(
      complete.transition_to(BeadStatus::InProgress),
      Err(FeedbackError::AlreadyComplete)
    );
    assert_eq!(
      complete.transition_to(BeadStatus::Blocked),
      Err(FeedbackError::AlreadyComplete)
    );
    assert_eq!(
      complete.transition_to(BeadStatus::Failed),
      Err(FeedbackError::AlreadyComplete)
    );
  }

  #[test]
  fn bead_status_invalid_transitions_return_correct_error() {
    assert!(matches!(
      BeadStatus::Pending.transition_to(BeadStatus::InProgress),
      Err(FeedbackError::InvalidTransition {
        from: BeadStatus::Pending,
        to: BeadStatus::InProgress
      })
    ));
    assert!(matches!(
      BeadStatus::Ready.transition_to(BeadStatus::Pending),
      Err(FeedbackError::InvalidTransition {
        from: BeadStatus::Ready,
        to: BeadStatus::Pending
      })
    ));
  }

  #[test]
  fn bead_status_is_terminal() {
    assert!(BeadStatus::Complete.is_terminal());
    assert!(!BeadStatus::Pending.is_terminal());
    assert!(!BeadStatus::Ready.is_terminal());
    assert!(!BeadStatus::InProgress.is_terminal());
    assert!(!BeadStatus::Blocked.is_terminal());
    assert!(!BeadStatus::Failed.is_terminal());
  }

  #[test]
  fn bead_status_is_active() {
    assert!(BeadStatus::Ready.is_active());
    assert!(BeadStatus::InProgress.is_active());
    assert!(!BeadStatus::Pending.is_active());
    assert!(!BeadStatus::Complete.is_active());
    assert!(!BeadStatus::Blocked.is_active());
    assert!(!BeadStatus::Failed.is_active());
  }
}
