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
pub enum BeadStatus {
  Pending,
  Ready,
  InProgress,
  Blocked,
  Complete,
  Failed,
}

impl Default for BeadStatus {
  fn default() -> Self {
    Self::Pending
  }
}

impl BeadStatus {
  #[must_use]
  pub fn can_transition_to(&self, to: &Self) -> bool {
    match (self, to) {
      (Self::Pending, Self::Ready) | (Self::Pending, Self::Blocked) => true,
      (Self::Ready, Self::InProgress) | (Self::Ready, Self::Blocked) => true,
      (Self::InProgress, Self::Complete)
      | (Self::InProgress, Self::Failed)
      | (Self::InProgress, Self::Blocked) => true,
      (Self::Blocked, Self::Ready)
      | (Self::Blocked, Self::Pending)
      | (Self::Blocked, Self::InProgress) => true,
      (Self::Failed, Self::Ready) | (Self::Failed, Self::Pending) => true,
      (Self::Complete, _) => false,
      (from, to) if from == to => true,
      _ => false,
    }
  }

  #[must_use]
  pub fn is_terminal(&self) -> bool {
    matches!(self, Self::Complete)
  }

  #[must_use]
  pub fn is_active(&self) -> bool {
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
