#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unreachable_patterns)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use super::error::PlanError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Execution state of a plan bead.
///
/// This enum replaces separate `ready: bool` and `completed: bool` fields,
/// following Scott Wlaschin's DDD principle of making states explicit.
/// Invalid state combinations (e.g., ready && completed) are impossible by design.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BeadState {
  /// Bead is waiting for dependencies or other conditions.
  #[default]
  Pending,
  /// Bead dependencies are satisfied, ready for execution.
  Ready,
  /// Bead execution completed successfully.
  Completed,
}

impl BeadState {
  /// Check if the bead is pending (waiting for dependencies).
  #[must_use]
  pub const fn is_pending(&self) -> bool {
    matches!(self, Self::Pending)
  }

  /// Check if the bead is ready for execution.
  #[must_use]
  pub const fn is_ready(&self) -> bool {
    matches!(self, Self::Ready)
  }

  /// Check if the bead is completed.
  #[must_use]
  pub const fn is_completed(&self) -> bool {
    matches!(self, Self::Completed)
  }

  /// Check if the bead can transition to a new state.
  ///
  /// Valid transitions:
  /// - Pending -> Ready (dependencies satisfied)
  /// - Pending -> Completed (skipped)
  /// - Ready -> Completed (finished)
  /// - Any -> Same (no-op)
  ///
  /// Invalid transitions:
  /// - Ready -> Pending (no going back)
  /// - Completed -> Pending (no going back)
  /// - Completed -> Ready (no going back)
  #[must_use]
  pub const fn can_transition_to(&self, next: Self) -> bool {
    matches!(
      (*self, next),
      (Self::Pending, Self::Pending | Self::Ready | Self::Completed)
        | (Self::Ready, Self::Ready | Self::Completed)
        | (Self::Completed, Self::Completed)
    )
  }

  /// Transition to a new state with exhaustive pattern matching.
  ///
  /// # Errors
  /// Returns `BeadStateError::InvalidTransition` if the transition is not allowed.
  pub const fn transition_to(self, next: Self) -> Result<Self, BeadStateError> {
    if self.can_transition_to(next) {
      Ok(next)
    } else {
      Err(BeadStateError::InvalidTransition {
        from: self.as_str(),
        to: next.as_str(),
      })
    }
  }

  /// Get string representation of the state.
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Pending => "pending",
      Self::Ready => "ready",
      Self::Completed => "completed",
    }
  }

  /// Mark the bead as ready (if pending).
  #[must_use]
  pub const fn into_ready(self) -> Self {
    match self {
      Self::Pending => Self::Ready,
      other => other,
    }
  }

  /// Mark the bead as completed.
  #[must_use]
  pub const fn into_completed(self) -> Self {
    Self::Completed
  }
}

/// Errors for bead state transitions.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BeadStateError {
  #[error("invalid bead state transition from '{from}' to '{to}'")]
  InvalidTransition {
    from: &'static str,
    to: &'static str,
  },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanBead {
  pub id: String,
  pub title: String,
  #[serde(default)]
  pub description: String,
  pub phase: u32,
  #[serde(default)]
  pub priority: u32,
  #[serde(default)]
  pub dependencies: Vec<String>,
  /// Execution state (replaces ready/completed booleans).
  #[serde(default)]
  pub state: BeadState,
  #[serde(default)]
  pub effort: u32,
  #[serde(default)]
  pub tags: Vec<String>,
}

impl Default for PlanBead {
  fn default() -> Self {
    Self {
      id: String::new(),
      title: String::new(),
      description: String::new(),
      phase: 1,
      priority: 0,
      dependencies: Vec::new(),
      state: BeadState::Pending,
      effort: 0,
      tags: Vec::new(),
    }
  }
}

impl PlanBead {
  /// Creates a new bead with mandatory identity fields.
  ///
  /// # Errors
  /// Returns `PlanError::EmptyBeadId` or `PlanError::EmptyBeadTitle` when required fields are blank.
  pub fn new(id: String, title: String, phase: u32) -> Result<Self, PlanError> {
    if id.trim().is_empty() {
      return Err(PlanError::EmptyBeadId);
    }
    if title.trim().is_empty() {
      return Err(PlanError::EmptyBeadTitle);
    }
    Ok(Self {
      id,
      title,
      phase,
      ..Self::default()
    })
  }

  #[must_use]
  pub fn with_description(self, description: String) -> Self {
    Self {
      description,
      ..self
    }
  }

  #[must_use]
  pub fn with_priority(self, priority: u32) -> Self {
    Self { priority, ..self }
  }

  #[must_use]
  pub fn with_dependency(self, dependency: String) -> Self {
    let dependencies = self
      .dependencies
      .iter()
      .cloned()
      .chain((!self.dependencies.contains(&dependency)).then_some(dependency))
      .collect();
    Self {
      dependencies,
      ..self
    }
  }

  #[must_use]
  pub fn with_effort(self, effort: u32) -> Self {
    Self { effort, ..self }
  }

  #[must_use]
  pub fn with_tag(self, tag: String) -> Self {
    let tags = self
      .tags
      .iter()
      .cloned()
      .chain((!self.tags.contains(&tag)).then_some(tag))
      .collect();
    Self { tags, ..self }
  }

  /// Set the bead state.
  #[must_use]
  pub fn with_state(self, state: BeadState) -> Self {
    Self { state, ..self }
  }

  /// Transition to a new state with validation.
  ///
  /// # Errors
  /// Returns `PlanError::InvalidStateTransition` if the transition is not allowed.
  pub fn transition_to(self, next: BeadState) -> Result<Self, PlanError> {
    // Capture values for error before moving self
    let id = self.id.clone();
    let from_str = self.state.as_str().to_string();
    let to_str = next.as_str().to_string();

    self.state.transition_to(next).map_or_else(
      |_| {
        Err(PlanError::InvalidStateTransition {
          bead_id: id,
          from: from_str,
          to: to_str,
        })
      },
      |new_state| {
        Ok(Self {
          state: new_state,
          ..self
        })
      },
    )
  }

  /// Mark the bead as ready (builder pattern).
  #[must_use]
  pub fn mark_ready(self) -> Self {
    Self {
      state: self.state.into_ready(),
      ..self
    }
  }

  /// Mark the bead as completed (builder pattern).
  #[must_use]
  pub fn mark_completed(self) -> Self {
    Self {
      state: BeadState::Completed,
      ..self
    }
  }

  /// Check if the bead is ready for execution.
  #[must_use]
  pub const fn is_ready(&self) -> bool {
    self.state.is_ready()
  }

  /// Check if the bead is completed.
  #[must_use]
  pub const fn is_completed(&self) -> bool {
    self.state.is_completed()
  }

  /// Check if the bead is pending.
  #[must_use]
  pub const fn is_pending(&self) -> bool {
    self.state.is_pending()
  }

  #[must_use]
  pub fn dependencies_satisfied(&self, completed_ids: &[&str]) -> bool {
    self
      .dependencies
      .iter()
      .all(|dep| completed_ids.contains(&dep.as_str()))
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::expect_used)]

  use super::*;

  #[test]
  fn bead_state_default_is_pending() {
    let state = BeadState::default();
    assert!(state.is_pending());
    assert!(!state.is_ready());
    assert!(!state.is_completed());
  }

  #[test]
  fn bead_state_is_predicates_are_exhaustive() {
    // Ensure all variants are covered
    for state in [BeadState::Pending, BeadState::Ready, BeadState::Completed] {
      let is_pending = state.is_pending();
      let is_ready = state.is_ready();
      let is_completed = state.is_completed();

      // Exactly one predicate should be true
      assert_eq!(
        usize::from(is_pending) + usize::from(is_ready) + usize::from(is_completed),
        1,
        "State {state:?} should match exactly one predicate"
      );
    }
  }

  #[test]
  fn bead_state_transitions_are_valid() {
    // Valid transitions
    assert!(BeadState::Pending.can_transition_to(BeadState::Ready));
    assert!(BeadState::Pending.can_transition_to(BeadState::Completed));
    assert!(BeadState::Ready.can_transition_to(BeadState::Completed));

    // Self-transitions
    assert!(BeadState::Pending.can_transition_to(BeadState::Pending));
    assert!(BeadState::Ready.can_transition_to(BeadState::Ready));
    assert!(BeadState::Completed.can_transition_to(BeadState::Completed));

    // Invalid transitions
    assert!(!BeadState::Ready.can_transition_to(BeadState::Pending));
    assert!(!BeadState::Completed.can_transition_to(BeadState::Pending));
    assert!(!BeadState::Completed.can_transition_to(BeadState::Ready));
  }

  #[test]
  fn bead_state_into_ready_only_affects_pending() {
    assert_eq!(BeadState::Pending.into_ready(), BeadState::Ready);
    assert_eq!(BeadState::Ready.into_ready(), BeadState::Ready);
    assert_eq!(BeadState::Completed.into_ready(), BeadState::Completed);
  }

  #[test]
  fn bead_state_into_completed_always_succeeds() {
    assert_eq!(BeadState::Pending.into_completed(), BeadState::Completed);
    assert_eq!(BeadState::Ready.into_completed(), BeadState::Completed);
    assert_eq!(BeadState::Completed.into_completed(), BeadState::Completed);
  }

  #[test]
  fn plan_bead_state_methods_delegate_correctly() {
    let pending = PlanBead::new("id".to_string(), "title".to_string(), 1)
      .expect("valid bead")
      .with_state(BeadState::Pending);
    let ready = PlanBead::new("id".to_string(), "title".to_string(), 1)
      .expect("valid bead")
      .with_state(BeadState::Ready);
    let completed = PlanBead::new("id".to_string(), "title".to_string(), 1)
      .expect("valid bead")
      .with_state(BeadState::Completed);

    assert!(pending.is_pending() && !pending.is_ready() && !pending.is_completed());
    assert!(!ready.is_pending() && ready.is_ready() && !ready.is_completed());
    assert!(!completed.is_pending() && !completed.is_ready() && completed.is_completed());
  }

  #[test]
  fn plan_bead_builder_methods_work() {
    let bead = PlanBead::new("id".to_string(), "title".to_string(), 1)
      .expect("valid bead")
      .mark_ready();
    assert!(bead.is_ready());

    let bead = bead.mark_completed();
    assert!(bead.is_completed());
  }

  #[test]
  fn plan_bead_serialization_roundtrip() {
    let bead = PlanBead::new("test-id".to_string(), "Test Title".to_string(), 2)
      .expect("valid bead")
      .with_state(BeadState::Completed);

    let json = serde_json::to_string(&bead).expect("serialize");
    let decoded: PlanBead = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(bead.id, decoded.id);
    assert_eq!(bead.state, decoded.state);
  }

  // ============================================
  // P0: State Machine Validation Tests
  // ============================================

  #[test]
  fn bead_state_transition_to_success_for_valid() {
    let pending = BeadState::Pending;

    let result = pending.transition_to(BeadState::Ready);
    assert!(result.is_ok());
    assert_eq!(result, Ok(BeadState::Ready));

    let result2 = pending.transition_to(BeadState::Completed);
    assert!(result2.is_ok());
    assert_eq!(result2, Ok(BeadState::Completed));

    let ready = BeadState::Ready;
    let result3 = ready.transition_to(BeadState::Completed);
    assert!(result3.is_ok());
    assert_eq!(result3, Ok(BeadState::Completed));
  }

  #[test]
  fn bead_state_transition_to_error_for_invalid() {
    let ready = BeadState::Ready;
    let result = ready.transition_to(BeadState::Pending);
    assert!(result.is_err());
    assert_eq!(
      result,
      Err(BeadStateError::InvalidTransition {
        from: "ready",
        to: "pending",
      })
    );

    let completed = BeadState::Completed;
    let result2 = completed.transition_to(BeadState::Pending);
    assert!(result2.is_err());
    assert_eq!(
      result2,
      Err(BeadStateError::InvalidTransition {
        from: "completed",
        to: "pending",
      })
    );

    let result3 = completed.transition_to(BeadState::Ready);
    assert!(result3.is_err());
    assert_eq!(
      result3,
      Err(BeadStateError::InvalidTransition {
        from: "completed",
        to: "ready",
      })
    );
  }

  #[test]
  fn bead_state_as_str() {
    assert_eq!(BeadState::Pending.as_str(), "pending");
    assert_eq!(BeadState::Ready.as_str(), "ready");
    assert_eq!(BeadState::Completed.as_str(), "completed");
  }

  #[test]
  fn plan_bead_transition_to_validates() {
    let bead = PlanBead::new("bead-1".to_string(), "Test".to_string(), 1)
      .expect("valid bead")
      .with_state(BeadState::Pending);

    // Valid transition
    let result = bead.transition_to(BeadState::Ready);
    assert!(result.is_ok());
    let updated = result.expect("valid transition");
    assert!(updated.is_ready());

    // Invalid transition
    let ready_bead = PlanBead::new("bead-2".to_string(), "Test".to_string(), 1)
      .expect("valid bead")
      .with_state(BeadState::Ready);

    let result2 = ready_bead.transition_to(BeadState::Pending);
    assert!(result2.is_err());
    assert!(matches!(
      result2,
      Err(PlanError::InvalidStateTransition { bead_id, from, to })
      if bead_id == "bead-2" && from == "ready" && to == "pending"
    ));
  }

  #[test]
  fn plan_bead_cannot_transition_from_completed() {
    let make_completed_bead = || {
      PlanBead::new("bead-3".to_string(), "Test".to_string(), 1)
        .expect("valid bead")
        .with_state(BeadState::Completed)
    };

    // Cannot go back to pending
    let result1 = make_completed_bead().transition_to(BeadState::Pending);
    assert!(result1.is_err());

    // Cannot go back to ready
    let result2 = make_completed_bead().transition_to(BeadState::Ready);
    assert!(result2.is_err());

    // Can stay completed (no-op)
    let result3 = make_completed_bead().transition_to(BeadState::Completed);
    assert!(result3.is_ok());
  }
}
