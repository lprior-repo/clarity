#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unreachable_patterns)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// Additional clippy lints to allow
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::unused_self)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::manual_strip)]
#![allow(clippy::format_push_string)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::suspicious_else_formatting)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::must_use_unit)]
#![allow(clippy::collection_is_never_read)]
#![allow(clippy::needless_collect)]
#![allow(clippy::manual_checked_ops)]
#![allow(clippy::needless_pass_by_value)]

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
  #[allow(clippy::unnested_or_patterns)]
  pub const fn can_transition_to(&self, next: Self) -> bool {
    matches!(
      (*self, next),
      (
        Self::Pending,
        Self::Pending | Self::Blocked | Self::InProgress
      ) | (
        Self::InProgress,
        Self::InProgress | Self::Blocked | Self::Complete
      ) | (
        Self::Blocked,
        Self::Pending | Self::InProgress | Self::Blocked
      ) | (Self::Complete, Self::Complete)
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

/// Represents an action that can be taken during plan execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
  /// Answer a pending interview question.
  AnswerQuestion {
    /// The unique identifier of the question to answer.
    question_id: String,
    /// The answer content to provide.
    answer: String,
  },
  /// Resolve a gap in the requirements.
  ResolveGap {
    /// The unique identifier of the gap to resolve.
    gap_id: String,
    /// The resolution content for the gap.
    resolution: String,
  },
}

impl Action {
  /// Creates a new `AnswerQuestion` action.
  #[must_use]
  pub const fn answer_question(question_id: String, answer: String) -> Self {
    Self::AnswerQuestion {
      question_id,
      answer,
    }
  }

  /// Creates a new `ResolveGap` action.
  #[must_use]
  pub const fn resolve_gap(gap_id: String, resolution: String) -> Self {
    Self::ResolveGap { gap_id, resolution }
  }

  /// Returns the question ID if this is an `AnswerQuestion` action.
  #[must_use]
  pub fn question_id(&self) -> Option<&str> {
    match self {
      Self::AnswerQuestion { question_id, .. } => Some(question_id),
      Self::ResolveGap { .. } => None,
    }
  }

  /// Returns the answer if this is an `AnswerQuestion` action.
  #[must_use]
  pub fn answer(&self) -> Option<&str> {
    match self {
      Self::AnswerQuestion { answer, .. } => Some(answer),
      Self::ResolveGap { .. } => None,
    }
  }

  /// Returns the gap ID if this is a `ResolveGap` action.
  #[must_use]
  pub fn gap_id(&self) -> Option<&str> {
    match self {
      Self::ResolveGap { gap_id, .. } => Some(gap_id),
      Self::AnswerQuestion { .. } => None,
    }
  }

  /// Returns the resolution if this is a `ResolveGap` action.
  #[must_use]
  pub fn resolution(&self) -> Option<&str> {
    match self {
      Self::ResolveGap { resolution, .. } => Some(resolution),
      Self::AnswerQuestion { .. } => None,
    }
  }
}

/// Risk level classification for plan items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
  #[default]
  Low,
  Medium,
  High,
  Critical,
}

impl RiskLevel {
  /// Returns the numeric severity score for this risk level.
  ///
  /// Higher values indicate greater risk.
  #[must_use]
  pub const fn severity(&self) -> u8 {
    match self {
      Self::Low => 1,
      Self::Medium => 2,
      Self::High => 3,
      Self::Critical => 4,
    }
  }

  /// Returns `true` if this risk level requires immediate attention.
  #[must_use]
  pub const fn requires_immediate_attention(&self) -> bool {
    matches!(self, Self::High | Self::Critical)
  }

  /// Returns `true` if this is a critical risk level.
  #[must_use]
  pub const fn is_critical(&self) -> bool {
    matches!(self, Self::Critical)
  }
}

impl std::fmt::Display for RiskLevel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Low => write!(f, "low"),
      Self::Medium => write!(f, "medium"),
      Self::High => write!(f, "high"),
      Self::Critical => write!(f, "critical"),
    }
  }
}

/// Effort level classification for tasks.
///
/// Represents t-shirt sizing for task effort estimation.
/// Each level maps to an approximate hour estimate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Effort {
  /// Trivial task - less than 1 hour.
  Trivial,
  /// Small task - 1-2 hours.
  Small,
  /// Medium task - 2-4 hours (default).
  #[default]
  Medium,
  /// Large task - 4-8 hours.
  Large,
  /// Extra large task - more than 8 hours.
  ExtraLarge,
}

impl Effort {
  /// Returns the estimated hours for this effort level.
  ///
  /// Returns `None` for `ExtraLarge` since it has no upper bound.
  /// Use `hours_range()` for a bounded range instead.
  #[must_use]
  pub const fn hours_estimate(&self) -> Option<u8> {
    match self {
      Self::Trivial => Some(1),
      Self::Small => Some(2),
      Self::Medium => Some(4),
      Self::Large => Some(8),
      Self::ExtraLarge => None,
    }
  }

  /// Returns the minimum hours for this effort level.
  #[must_use]
  pub const fn min_hours(&self) -> u8 {
    match self {
      Self::Trivial => 0,
      Self::Small => 1,
      Self::Medium => 2,
      Self::Large => 4,
      Self::ExtraLarge => 8,
    }
  }

  /// Returns the maximum hours for this effort level.
  ///
  /// Returns `None` for `ExtraLarge` since it has no upper bound.
  #[must_use]
  pub const fn max_hours(&self) -> Option<u8> {
    match self {
      Self::Trivial => Some(1),
      Self::Small => Some(2),
      Self::Medium => Some(4),
      Self::Large => Some(8),
      Self::ExtraLarge => None,
    }
  }

  /// Returns the numeric weight for this effort level.
  ///
  /// Higher values indicate more effort.
  #[must_use]
  pub const fn weight(&self) -> u8 {
    match self {
      Self::Trivial => 1,
      Self::Small => 2,
      Self::Medium => 3,
      Self::Large => 4,
      Self::ExtraLarge => 5,
    }
  }

  /// Returns `true` if this effort level can be completed in a single day.
  #[must_use]
  pub const fn is_single_day(&self) -> bool {
    matches!(
      self,
      Self::Trivial | Self::Small | Self::Medium | Self::Large
    )
  }

  /// Returns `true` if this is a large or extra large effort task.
  #[must_use]
  pub const fn is_significant(&self) -> bool {
    matches!(self, Self::Large | Self::ExtraLarge)
  }

  /// Returns `true` if this is a trivial or small effort task.
  #[must_use]
  pub const fn is_quick(&self) -> bool {
    matches!(self, Self::Trivial | Self::Small)
  }
}

impl std::fmt::Display for Effort {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Trivial => write!(f, "trivial"),
      Self::Small => write!(f, "small"),
      Self::Medium => write!(f, "medium"),
      Self::Large => write!(f, "large"),
      Self::ExtraLarge => write!(f, "extra_large"),
    }
  }
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

  // ============================================
  // RiskLevel Tests
  // ============================================

  #[test]
  fn risk_level_default_is_low() {
    assert_eq!(RiskLevel::default(), RiskLevel::Low);
  }

  #[test]
  fn risk_level_severity_values() {
    assert_eq!(RiskLevel::Low.severity(), 1);
    assert_eq!(RiskLevel::Medium.severity(), 2);
    assert_eq!(RiskLevel::High.severity(), 3);
    assert_eq!(RiskLevel::Critical.severity(), 4);
  }

  #[test]
  fn risk_level_requires_immediate_attention() {
    assert!(!RiskLevel::Low.requires_immediate_attention());
    assert!(!RiskLevel::Medium.requires_immediate_attention());
    assert!(RiskLevel::High.requires_immediate_attention());
    assert!(RiskLevel::Critical.requires_immediate_attention());
  }

  #[test]
  fn risk_level_is_critical() {
    assert!(!RiskLevel::Low.is_critical());
    assert!(!RiskLevel::Medium.is_critical());
    assert!(!RiskLevel::High.is_critical());
    assert!(RiskLevel::Critical.is_critical());
  }

  #[test]
  fn risk_level_display() {
    assert_eq!(format!("{}", RiskLevel::Low), "low");
    assert_eq!(format!("{}", RiskLevel::Medium), "medium");
    assert_eq!(format!("{}", RiskLevel::High), "high");
    assert_eq!(format!("{}", RiskLevel::Critical), "critical");
  }

  #[test]
  fn risk_level_serde_roundtrip() {
    for level in [
      RiskLevel::Low,
      RiskLevel::Medium,
      RiskLevel::High,
      RiskLevel::Critical,
    ] {
      let json = serde_json::to_string(&level).ok();
      assert!(json.is_some());
      let parsed: Option<RiskLevel> = json.and_then(|s| serde_json::from_str(&s).ok());
      assert_eq!(parsed, Some(level));
    }
  }

  #[test]
  fn risk_level_serde_snake_case() {
    let low_json = serde_json::to_string(&RiskLevel::Low).ok();
    assert_eq!(low_json.as_deref(), Some("\"low\""));

    let critical_json = serde_json::to_string(&RiskLevel::Critical).ok();
    assert_eq!(critical_json.as_deref(), Some("\"critical\""));
  }

  // ============================================
  // Action Tests
  // ============================================

  #[test]
  fn action_answer_question_constructor() {
    let action = Action::answer_question("q-123".to_string(), "My answer".to_string());

    match &action {
      Action::AnswerQuestion {
        question_id,
        answer,
      } => {
        assert_eq!(question_id, "q-123");
        assert_eq!(answer, "My answer");
      }
      Action::ResolveGap { .. } => panic!("Expected AnswerQuestion variant"),
    }
  }

  #[test]
  fn action_question_id_accessor() {
    let action = Action::answer_question("q-456".to_string(), "response".to_string());

    assert_eq!(action.question_id(), Some("q-456"));
  }

  #[test]
  fn action_answer_accessor() {
    let action = Action::answer_question("q-789".to_string(), "the answer".to_string());

    assert_eq!(action.answer(), Some("the answer"));
  }

  #[test]
  fn action_equality() {
    let action1 = Action::answer_question("q-1".to_string(), "answer".to_string());
    let action2 = Action::answer_question("q-1".to_string(), "answer".to_string());
    let action3 = Action::answer_question("q-1".to_string(), "different".to_string());

    assert_eq!(action1, action2);
    assert_ne!(action1, action3);
  }

  #[test]
  fn action_clone() {
    let action = Action::answer_question("q-clone".to_string(), "clone me".to_string());
    let cloned = action.clone();

    assert_eq!(action, cloned);
  }

  #[test]
  fn action_serde_roundtrip() {
    let action = Action::answer_question("q-serde".to_string(), "serialize this".to_string());

    let json = serde_json::to_string(&action).ok();
    assert!(json.is_some());

    let parsed: Option<Action> = json.and_then(|s| serde_json::from_str(&s).ok());
    assert_eq!(parsed, Some(action));
  }

  #[test]
  fn action_serde_snake_case_format() {
    let action = Action::answer_question("q-1".to_string(), "answer".to_string());
    let json = serde_json::to_string(&action).ok();

    assert!(json.is_some());
    let json_str = json.as_deref();
    // Should contain "answer_question" as the variant name in snake_case
    assert!(json_str.map_or(false, |s| s.contains("answer_question")));
    // Should contain "question_id" as the field name in snake_case
    assert!(json_str.map_or(false, |s| s.contains("question_id")));
  }

  // ============================================
  // ResolveGap Action Tests
  // ============================================

  #[test]
  fn action_resolve_gap_constructor() {
    let action = Action::resolve_gap("gap-123".to_string(), "This is the resolution".to_string());

    match &action {
      Action::ResolveGap { gap_id, resolution } => {
        assert_eq!(gap_id, "gap-123");
        assert_eq!(resolution, "This is the resolution");
      }
      Action::AnswerQuestion { .. } => panic!("Expected ResolveGap variant"),
    }
  }

  #[test]
  fn action_gap_id_accessor() {
    let action = Action::resolve_gap("gap-456".to_string(), "resolved".to_string());

    assert_eq!(action.gap_id(), Some("gap-456"));
  }

  #[test]
  fn action_resolution_accessor() {
    let action = Action::resolve_gap("gap-789".to_string(), "the resolution text".to_string());

    assert_eq!(action.resolution(), Some("the resolution text"));
  }

  #[test]
  fn action_resolve_gap_accessors_return_none_for_answer_question() {
    let action = Action::answer_question("q-1".to_string(), "answer".to_string());

    assert_eq!(action.gap_id(), None);
    assert_eq!(action.resolution(), None);
  }

  #[test]
  fn action_answer_question_accessors_return_none_for_resolve_gap() {
    let action = Action::resolve_gap("gap-1".to_string(), "resolution".to_string());

    assert_eq!(action.question_id(), None);
    assert_eq!(action.answer(), None);
  }

  #[test]
  fn action_resolve_gap_equality() {
    let action1 = Action::resolve_gap("gap-1".to_string(), "resolution".to_string());
    let action2 = Action::resolve_gap("gap-1".to_string(), "resolution".to_string());
    let action3 = Action::resolve_gap("gap-1".to_string(), "different".to_string());

    assert_eq!(action1, action2);
    assert_ne!(action1, action3);
  }

  #[test]
  fn action_resolve_gap_clone() {
    let action = Action::resolve_gap("gap-clone".to_string(), "clone me".to_string());
    let cloned = action.clone();

    assert_eq!(action, cloned);
  }

  #[test]
  fn action_resolve_gap_serde_roundtrip() {
    let action = Action::resolve_gap("gap-serde".to_string(), "serialize this".to_string());

    let json = serde_json::to_string(&action).ok();
    assert!(json.is_some());

    let parsed: Option<Action> = json.and_then(|s| serde_json::from_str(&s).ok());
    assert_eq!(parsed, Some(action));
  }

  #[test]
  fn action_resolve_gap_serde_snake_case_format() {
    let action = Action::resolve_gap("gap-1".to_string(), "resolution".to_string());
    let json = serde_json::to_string(&action).ok();

    assert!(json.is_some());
    let json_str = json.as_deref();
    // Should contain "resolve_gap" as the variant name in snake_case
    assert!(json_str.map_or(false, |s| s.contains("resolve_gap")));
    // Should contain "gap_id" as the field name in snake_case
    assert!(json_str.map_or(false, |s| s.contains("gap_id")));
    // Should contain "resolution" as the field name
    assert!(json_str.map_or(false, |s| s.contains("resolution")));
  }

  #[test]
  fn action_variants_are_distinct() {
    let answer = Action::answer_question("q-1".to_string(), "answer".to_string());
    let resolve = Action::resolve_gap("gap-1".to_string(), "resolution".to_string());

    assert_ne!(answer, resolve);
  }

  // ============================================
  // Effort Tests
  // ============================================

  #[test]
  fn effort_default_is_medium() {
    assert_eq!(Effort::default(), Effort::Medium);
  }

  #[test]
  fn effort_hours_estimate_values() {
    assert_eq!(Effort::Trivial.hours_estimate(), Some(1));
    assert_eq!(Effort::Small.hours_estimate(), Some(2));
    assert_eq!(Effort::Medium.hours_estimate(), Some(4));
    assert_eq!(Effort::Large.hours_estimate(), Some(8));
    assert_eq!(Effort::ExtraLarge.hours_estimate(), None);
  }

  #[test]
  fn effort_min_hours_values() {
    assert_eq!(Effort::Trivial.min_hours(), 0);
    assert_eq!(Effort::Small.min_hours(), 1);
    assert_eq!(Effort::Medium.min_hours(), 2);
    assert_eq!(Effort::Large.min_hours(), 4);
    assert_eq!(Effort::ExtraLarge.min_hours(), 8);
  }

  #[test]
  fn effort_max_hours_values() {
    assert_eq!(Effort::Trivial.max_hours(), Some(1));
    assert_eq!(Effort::Small.max_hours(), Some(2));
    assert_eq!(Effort::Medium.max_hours(), Some(4));
    assert_eq!(Effort::Large.max_hours(), Some(8));
    assert_eq!(Effort::ExtraLarge.max_hours(), None);
  }

  #[test]
  fn effort_weight_values() {
    assert_eq!(Effort::Trivial.weight(), 1);
    assert_eq!(Effort::Small.weight(), 2);
    assert_eq!(Effort::Medium.weight(), 3);
    assert_eq!(Effort::Large.weight(), 4);
    assert_eq!(Effort::ExtraLarge.weight(), 5);
  }

  #[test]
  fn effort_is_single_day() {
    assert!(Effort::Trivial.is_single_day());
    assert!(Effort::Small.is_single_day());
    assert!(Effort::Medium.is_single_day());
    assert!(Effort::Large.is_single_day());
    assert!(!Effort::ExtraLarge.is_single_day());
  }

  #[test]
  fn effort_is_significant() {
    assert!(!Effort::Trivial.is_significant());
    assert!(!Effort::Small.is_significant());
    assert!(!Effort::Medium.is_significant());
    assert!(Effort::Large.is_significant());
    assert!(Effort::ExtraLarge.is_significant());
  }

  #[test]
  fn effort_is_quick() {
    assert!(Effort::Trivial.is_quick());
    assert!(Effort::Small.is_quick());
    assert!(!Effort::Medium.is_quick());
    assert!(!Effort::Large.is_quick());
    assert!(!Effort::ExtraLarge.is_quick());
  }

  #[test]
  fn effort_display() {
    assert_eq!(format!("{}", Effort::Trivial), "trivial");
    assert_eq!(format!("{}", Effort::Small), "small");
    assert_eq!(format!("{}", Effort::Medium), "medium");
    assert_eq!(format!("{}", Effort::Large), "large");
    assert_eq!(format!("{}", Effort::ExtraLarge), "extra_large");
  }

  #[test]
  fn effort_serde_roundtrip() {
    for effort in [
      Effort::Trivial,
      Effort::Small,
      Effort::Medium,
      Effort::Large,
      Effort::ExtraLarge,
    ] {
      let json = serde_json::to_string(&effort).ok();
      assert!(json.is_some());
      let parsed: Option<Effort> = json.and_then(|s| serde_json::from_str(&s).ok());
      assert_eq!(parsed, Some(effort));
    }
  }

  #[test]
  fn effort_serde_snake_case() {
    let trivial_json = serde_json::to_string(&Effort::Trivial).ok();
    assert_eq!(trivial_json.as_deref(), Some("\"trivial\""));

    let extra_large_json = serde_json::to_string(&Effort::ExtraLarge).ok();
    assert_eq!(extra_large_json.as_deref(), Some("\"extra_large\""));
  }

  #[test]
  fn effort_clone_and_equality() {
    let effort = Effort::Large;
    let cloned = effort.clone();
    assert_eq!(effort, cloned);
  }

  #[test]
  fn effort_ordering_by_weight() {
    // Weights should be monotonically increasing
    assert!(Effort::Trivial.weight() < Effort::Small.weight());
    assert!(Effort::Small.weight() < Effort::Medium.weight());
    assert!(Effort::Medium.weight() < Effort::Large.weight());
    assert!(Effort::Large.weight() < Effort::ExtraLarge.weight());
  }
}
