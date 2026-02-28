#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unreachable_patterns)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PlanError {
  #[error("dependency error: {0}")]
  DependencyError(String),
  #[error("invalid phase: {0}")]
  InvalidPhase(u32),
  #[error("phase not complete: {0}")]
  PhaseNotComplete(u32),
  #[error("no actionable beads")]
  NoActionableBeads,
  #[error("session ID is empty")]
  EmptySessionId,
  #[error("circular dependency detected: {0} -> {1}")]
  CircularDependency(String, String),
  #[error("invalid phase status transition from {from:?} to {to:?}")]
  InvalidPhaseTransition { from: PhaseStatus, to: PhaseStatus },
  #[error("invalid bead status transition from {from:?} to {to:?}")]
  InvalidBeadTransition { from: BeadStatus, to: BeadStatus },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PhaseStatus {
  #[default]
  Pending,
  InProgress,
  Complete,
  Blocked,
}

impl PhaseStatus {
  /// Check if transition to another status is valid.
  ///
  /// Valid transitions:
  /// - Pending -> `InProgress`, Blocked
  /// - `InProgress` -> Complete, Blocked
  /// - Blocked -> Pending, `InProgress`
  /// - Complete -> Complete (terminal)
  #[must_use]
  pub const fn can_transition_to(&self, next: Self) -> bool {
    matches!(
      (*self, next),
      (
        Self::Pending,
        Self::Pending | Self::InProgress | Self::Blocked
      ) | (
        Self::InProgress | Self::Blocked,
        Self::InProgress | Self::Blocked
      ) | (Self::Complete | Self::InProgress, Self::Complete)
    )
  }

  /// Transition to a new status with exhaustive pattern matching.
  ///
  /// # Errors
  /// Returns `PlanError::InvalidPhaseTransition` if the transition is not allowed.
  pub const fn transition_to(self, next: Self) -> Result<Self, PlanError> {
    if self.can_transition_to(next) {
      Ok(next)
    } else {
      Err(PlanError::InvalidPhaseTransition {
        from: self,
        to: next,
      })
    }
  }

  #[must_use]
  pub const fn is_terminal(&self) -> bool {
    matches!(self, Self::Complete)
  }

  #[must_use]
  pub const fn is_active(&self) -> bool {
    matches!(self, Self::InProgress)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BeadStatus {
  #[default]
  Pending,
  Ready,
  InProgress,
  Complete,
  Blocked,
}

impl BeadStatus {
  /// Check if transition to another status is valid.
  ///
  /// Valid transitions:
  /// - Pending -> Ready, Blocked
  /// - Ready -> `InProgress`, Blocked
  /// - `InProgress` -> Complete, Blocked
  /// - Blocked -> Pending, Ready, `InProgress`
  /// - Complete -> Complete (terminal)
  #[must_use]
  pub const fn can_transition_to(&self, next: Self) -> bool {
    matches!(
      (*self, next),
      (Self::Pending | Self::Blocked, Self::Pending)
        | (Self::Ready | Self::Pending | Self::Blocked, Self::Ready)
        | (
          Self::InProgress | Self::Ready | Self::Blocked,
          Self::InProgress
        )
        | (Self::Complete | Self::InProgress, Self::Complete)
        | (
          Self::Blocked | Self::Pending | Self::Ready | Self::InProgress,
          Self::Blocked
        )
    )
  }

  /// Transition to a new status with exhaustive pattern matching.
  ///
  /// # Errors
  /// Returns `PlanError::InvalidBeadTransition` if the transition is not allowed.
  pub const fn transition_to(self, next: Self) -> Result<Self, PlanError> {
    if self.can_transition_to(next) {
      Ok(next)
    } else {
      Err(PlanError::InvalidBeadTransition {
        from: self,
        to: next,
      })
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
pub struct PlanBead {
  pub id: String,
  pub title: String,
  pub description: String,
  pub priority: u8,
  pub status: BeadStatus,
  pub depends_on: Vec<String>,
  pub blocks: Vec<String>,
}

impl Default for PlanBead {
  fn default() -> Self {
    Self {
      id: String::new(),
      title: String::new(),
      description: String::new(),
      priority: 100,
      status: BeadStatus::default(),
      depends_on: Vec::new(),
      blocks: Vec::new(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phase {
  pub phase_number: u32,
  pub name: String,
  pub description: String,
  pub beads: Vec<PlanBead>,
  pub status: PhaseStatus,
  pub blockers: Vec<String>,
}

impl Default for Phase {
  fn default() -> Self {
    Self {
      phase_number: 1,
      name: String::new(),
      description: String::new(),
      beads: Vec::new(),
      status: PhaseStatus::default(),
      blockers: Vec::new(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExecutionPlan {
  pub session_id: String,
  pub phases: Vec<Phase>,
  pub blockers: Vec<String>,
  pub created_at: String,
}

#[cfg(test)]
mod tests {
  use super::*;

  // ============================================
  // PhaseStatus Exhaustive Transition Tests
  // ============================================

  #[test]
  fn phase_status_default_is_pending() {
    assert_eq!(PhaseStatus::default(), PhaseStatus::Pending);
  }

  #[test]
  fn phase_status_all_transitions_from_pending() {
    let pending = PhaseStatus::Pending;

    // Valid transitions from Pending
    assert!(pending.can_transition_to(PhaseStatus::InProgress));
    assert!(pending.can_transition_to(PhaseStatus::Blocked));
    assert!(pending.can_transition_to(PhaseStatus::Pending)); // no-op

    // Invalid transitions from Pending
    assert!(!pending.can_transition_to(PhaseStatus::Complete));
  }

  #[test]
  fn phase_status_all_transitions_from_in_progress() {
    let in_progress = PhaseStatus::InProgress;

    // Valid transitions from InProgress
    assert!(in_progress.can_transition_to(PhaseStatus::Complete));
    assert!(in_progress.can_transition_to(PhaseStatus::Blocked));
    assert!(in_progress.can_transition_to(PhaseStatus::InProgress)); // no-op

    // Invalid transitions from InProgress
    assert!(!in_progress.can_transition_to(PhaseStatus::Pending));
  }

  #[test]
  fn phase_status_all_transitions_from_blocked() {
    let blocked = PhaseStatus::Blocked;

    // Valid transitions from Blocked
    assert!(blocked.can_transition_to(PhaseStatus::Pending));
    assert!(blocked.can_transition_to(PhaseStatus::InProgress));
    assert!(blocked.can_transition_to(PhaseStatus::Blocked)); // no-op

    // Invalid transitions from Blocked
    assert!(!blocked.can_transition_to(PhaseStatus::Complete));
  }

  #[test]
  fn phase_status_all_transitions_from_complete() {
    let complete = PhaseStatus::Complete;

    // Complete is terminal - no transitions out except no-op
    assert!(complete.can_transition_to(PhaseStatus::Complete)); // no-op only

    // All other transitions invalid
    assert!(!complete.can_transition_to(PhaseStatus::Pending));
    assert!(!complete.can_transition_to(PhaseStatus::InProgress));
    assert!(!complete.can_transition_to(PhaseStatus::Blocked));
  }

  #[test]
  fn phase_status_transition_to_returns_correct_result() {
    // Valid transitions
    assert_eq!(
      PhaseStatus::Pending.transition_to(PhaseStatus::InProgress),
      Ok(PhaseStatus::InProgress)
    );
    assert_eq!(
      PhaseStatus::InProgress.transition_to(PhaseStatus::Complete),
      Ok(PhaseStatus::Complete)
    );
    assert_eq!(
      PhaseStatus::Blocked.transition_to(PhaseStatus::InProgress),
      Ok(PhaseStatus::InProgress)
    );

    // Invalid transitions
    assert!(PhaseStatus::Complete
      .transition_to(PhaseStatus::Pending)
      .is_err());
    assert!(PhaseStatus::Pending
      .transition_to(PhaseStatus::Complete)
      .is_err());
  }

  #[test]
  fn phase_status_no_op_transitions() {
    for status in [
      PhaseStatus::Pending,
      PhaseStatus::InProgress,
      PhaseStatus::Complete,
      PhaseStatus::Blocked,
    ] {
      assert!(status.can_transition_to(status));
      assert_eq!(status.transition_to(status), Ok(status));
    }
  }

  #[test]
  fn phase_status_is_terminal() {
    assert!(PhaseStatus::Complete.is_terminal());
    assert!(!PhaseStatus::Pending.is_terminal());
    assert!(!PhaseStatus::InProgress.is_terminal());
    assert!(!PhaseStatus::Blocked.is_terminal());
  }

  #[test]
  fn phase_status_is_active() {
    assert!(PhaseStatus::InProgress.is_active());
    assert!(!PhaseStatus::Pending.is_active());
    assert!(!PhaseStatus::Complete.is_active());
    assert!(!PhaseStatus::Blocked.is_active());
  }

  // ============================================
  // BeadStatus Exhaustive Transition Tests
  // ============================================

  #[test]
  fn bead_status_default_is_pending() {
    assert_eq!(BeadStatus::default(), BeadStatus::Pending);
  }

  #[test]
  fn bead_status_all_transitions_from_pending() {
    let pending = BeadStatus::Pending;

    // Valid transitions from Pending
    assert!(pending.can_transition_to(BeadStatus::Ready));
    assert!(pending.can_transition_to(BeadStatus::Blocked));
    assert!(pending.can_transition_to(BeadStatus::Pending)); // no-op

    // Invalid transitions from Pending
    assert!(!pending.can_transition_to(BeadStatus::InProgress));
    assert!(!pending.can_transition_to(BeadStatus::Complete));
  }

  #[test]
  fn bead_status_all_transitions_from_ready() {
    let ready = BeadStatus::Ready;

    // Valid transitions from Ready
    assert!(ready.can_transition_to(BeadStatus::InProgress));
    assert!(ready.can_transition_to(BeadStatus::Blocked));
    assert!(ready.can_transition_to(BeadStatus::Ready)); // no-op

    // Invalid transitions from Ready
    assert!(!ready.can_transition_to(BeadStatus::Pending));
    assert!(!ready.can_transition_to(BeadStatus::Complete));
  }

  #[test]
  fn bead_status_all_transitions_from_in_progress() {
    let in_progress = BeadStatus::InProgress;

    // Valid transitions from InProgress
    assert!(in_progress.can_transition_to(BeadStatus::Complete));
    assert!(in_progress.can_transition_to(BeadStatus::Blocked));
    assert!(in_progress.can_transition_to(BeadStatus::InProgress)); // no-op

    // Invalid transitions from InProgress
    assert!(!in_progress.can_transition_to(BeadStatus::Pending));
    assert!(!in_progress.can_transition_to(BeadStatus::Ready));
  }

  #[test]
  fn bead_status_all_transitions_from_blocked() {
    let blocked = BeadStatus::Blocked;

    // Valid transitions from Blocked
    assert!(blocked.can_transition_to(BeadStatus::Pending));
    assert!(blocked.can_transition_to(BeadStatus::Ready));
    assert!(blocked.can_transition_to(BeadStatus::InProgress));
    assert!(blocked.can_transition_to(BeadStatus::Blocked)); // no-op

    // Invalid transitions from Blocked
    assert!(!blocked.can_transition_to(BeadStatus::Complete));
  }

  #[test]
  fn bead_status_all_transitions_from_complete() {
    let complete = BeadStatus::Complete;

    // Complete is terminal - no transitions out except no-op
    assert!(complete.can_transition_to(BeadStatus::Complete)); // no-op only

    // All other transitions invalid
    assert!(!complete.can_transition_to(BeadStatus::Pending));
    assert!(!complete.can_transition_to(BeadStatus::Ready));
    assert!(!complete.can_transition_to(BeadStatus::InProgress));
    assert!(!complete.can_transition_to(BeadStatus::Blocked));
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
      BeadStatus::Blocked.transition_to(BeadStatus::Ready),
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
    ] {
      assert!(status.can_transition_to(status));
      assert_eq!(status.transition_to(status), Ok(status));
    }
  }

  #[test]
  fn bead_status_is_terminal() {
    assert!(BeadStatus::Complete.is_terminal());
    assert!(!BeadStatus::Pending.is_terminal());
    assert!(!BeadStatus::Ready.is_terminal());
    assert!(!BeadStatus::InProgress.is_terminal());
    assert!(!BeadStatus::Blocked.is_terminal());
  }

  #[test]
  fn bead_status_is_active() {
    assert!(BeadStatus::Ready.is_active());
    assert!(BeadStatus::InProgress.is_active());
    assert!(!BeadStatus::Pending.is_active());
    assert!(!BeadStatus::Complete.is_active());
    assert!(!BeadStatus::Blocked.is_active());
  }
}
