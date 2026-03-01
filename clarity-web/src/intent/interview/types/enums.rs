//! Enumerated types for interview sessions.
//!
//! This module defines the core enumerations used throughout the interview system:
//!
//! - [`Profile`]: Intent profile types (API, CLI, Event, etc.)
//! - [`InterviewStage`]: Interview lifecycle stages with state machine semantics
//! - [`Perspective`]: Question perspective (User, Developer, Ops, etc.)
//! - [`QuestionPriority`]: Question importance levels
//! - [`QuestionCategory`]: Question classification types

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unreachable_patterns)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::str::FromStr;

use super::ProfileParseError;

/// Profile type - determines which questions to ask and required fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Profile {
  #[default]
  Api,
  Cli,
  Event,
  Data,
  Workflow,
  Ui,
}

impl Profile {
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Api => "api",
      Self::Cli => "cli",
      Self::Event => "event",
      Self::Data => "data",
      Self::Workflow => "workflow",
      Self::Ui => "ui",
    }
  }

  /// # Errors
  /// Returns `ProfileParseError` when input is not a known profile.
  pub fn parse(s: &str) -> Result<Self, ProfileParseError> {
    let normalized = s.trim().to_ascii_lowercase();
    match normalized.as_str() {
      "api" => Ok(Self::Api),
      "cli" => Ok(Self::Cli),
      "event" => Ok(Self::Event),
      "data" => Ok(Self::Data),
      "workflow" => Ok(Self::Workflow),
      "ui" => Ok(Self::Ui),
      _ => Err(ProfileParseError::UnknownProfile {
        input: s.to_string(),
      }),
    }
  }

  #[must_use]
  pub const fn required_fields(&self) -> &'static [&'static str] {
    match self {
      Self::Api => &[
        "base_url",
        "auth_method",
        "happy_path",
        "error_cases",
        "response_format",
      ],
      Self::Cli => &["command_name", "happy_path", "help_text", "exit_codes"],
      Self::Event => &["event_type", "payload_schema", "trigger"],
      Self::Data => &["data_model", "access_patterns", "retention"],
      Self::Workflow => &["steps", "happy_path", "error_recovery"],
      Self::Ui => &["user_flows", "happy_path", "states"],
    }
  }
}

impl FromStr for Profile {
  type Err = ProfileParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::parse(s)
  }
}

/// Interview lifecycle stage with explicit state machine semantics.
///
/// The stage represents the high-level phase of an interview session and
/// enforces valid transitions between states.
///
/// # Stage Descriptions
///
/// | Stage | Description | Typical Round |
/// |-------|-------------|---------------|
/// | Discovery | Initial information gathering | 1-2 |
/// | Refinement | Deep-dive into specific areas | 3 |
/// | Validation | Confirm understanding and fill gaps | 4 |
/// | Complete | Session finished | 5+ |
/// | Paused | Temporarily suspended | any |
///
/// # State Machine
///
/// ```text
///                    ┌─────────────────────────────────────┐
///                    │                                     │
///                    ▼                                     │
///              ┌──────────┐                                │
///              │Discovery │────────────────────┐           │
///              └────┬─────┘                    │           │
///                   │                          │           │
///                   ▼                          ▼           │
///             ┌───────────┐              ┌───────┐        │
///             │Refinement │─────────────▶│Paused │◀───────┤
///             └─────┬─────┘              └───────┘        │
///                   │                       ▲             │
///                   ▼                       │             │
///            ┌───────────┐                  │             │
///            │Validation │──────────────────┤             │
///            └─────┬─────┘                  │             │
///                  │                        │             │
///                  ▼                        │             │
///            ┌──────────┐                   │             │
///            │Complete  │───────────────────┘             │
///            └──────────┘  (terminal)                     │
///                                                         │
///   Resume from Paused ──────────────────────────────────┘
///   (to Discovery/Refinement/Validation)
/// ```
///
/// # Examples
///
/// ## Checking Valid Transitions
///
/// ```
/// # use clarity_web::intent::interview::types::InterviewStage;
/// let stage = InterviewStage::Discovery;
///
/// // Check if transition is valid
/// assert!(stage.can_transition_to(InterviewStage::Refinement));
/// assert!(stage.can_transition_to(InterviewStage::Paused));
/// assert!(!stage.can_transition_to(InterviewStage::Complete)); // Can't skip to complete
/// ```
///
/// ## Performing Transitions
///
/// ```
/// # use clarity_web::intent::interview::types::InterviewStage;
/// let stage = InterviewStage::Discovery;
///
/// // Transition to refinement
/// let next = stage.transition_to(InterviewStage::Refinement).unwrap();
/// assert_eq!(next, InterviewStage::Refinement);
///
/// // Invalid transition returns error
/// let result = InterviewStage::Complete.transition_to(InterviewStage::Discovery);
/// assert!(result.is_err());
/// ```
///
/// ## Checking Stage Properties
///
/// ```
/// # use clarity_web::intent::interview::types::InterviewStage;
/// assert!(InterviewStage::Complete.is_terminal());
/// assert!(InterviewStage::Discovery.is_active());
/// assert!(InterviewStage::Paused.is_paused());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum InterviewStage {
  /// Initial information gathering phase.
  ///
  /// In this stage, the system asks broad questions to understand the
  /// user's intent. Typically covers rounds 1-2.
  #[default]
  Discovery,

  /// Deep-dive into specific areas based on initial answers.
  ///
  /// Focuses on clarifying ambiguous answers and exploring edge cases.
  /// Typically covers round 3.
  Refinement,

  /// Confirm understanding and fill remaining gaps.
  ///
  /// Reviews all collected information and ensures completeness.
  /// Typically covers round 4.
  Validation,

  /// Session has finished.
  ///
  /// This is a terminal state - no further modifications are allowed.
  /// The session can no longer accept new answers or advance rounds.
  Complete,

  /// Session is temporarily suspended.
  ///
  /// The session can be resumed to any active stage (Discovery,
  /// Refinement, or Validation). No modifications are allowed while paused.
  Paused,
}

impl InterviewStage {
  /// Returns the string representation of this stage.
  ///
  /// The returned string matches the serde serialization format (lowercase).
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::InterviewStage;
  /// assert_eq!(InterviewStage::Discovery.as_str(), "discovery");
  /// assert_eq!(InterviewStage::Refinement.as_str(), "refinement");
  /// assert_eq!(InterviewStage::Validation.as_str(), "validation");
  /// assert_eq!(InterviewStage::Complete.as_str(), "complete");
  /// assert_eq!(InterviewStage::Paused.as_str(), "paused");
  /// ```
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Discovery => "discovery",
      Self::Refinement => "refinement",
      Self::Validation => "validation",
      Self::Complete => "complete",
      Self::Paused => "paused",
    }
  }

  /// Check if transition to another stage is valid.
  ///
  /// # Valid Transitions
  ///
  /// | From | Valid To |
  /// |------|----------|
  /// | Discovery | Discovery, Refinement, Validation, Paused |
  /// | Refinement | Refinement, Validation, Complete, Paused |
  /// | Validation | Validation, Complete, Paused |
  /// | Paused | Discovery, Refinement, Validation, Paused |
  /// | Complete | Complete (terminal) |
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::InterviewStage;
  /// // Valid forward progression
  /// assert!(InterviewStage::Discovery.can_transition_to(InterviewStage::Refinement));
  /// assert!(InterviewStage::Refinement.can_transition_to(InterviewStage::Validation));
  /// assert!(InterviewStage::Validation.can_transition_to(InterviewStage::Complete));
  ///
  /// // Can skip stages (Discovery -> Validation)
  /// assert!(InterviewStage::Discovery.can_transition_to(InterviewStage::Validation));
  ///
  /// // Can pause from any active stage
  /// assert!(InterviewStage::Discovery.can_transition_to(InterviewStage::Paused));
  /// assert!(InterviewStage::Refinement.can_transition_to(InterviewStage::Paused));
  /// assert!(InterviewStage::Validation.can_transition_to(InterviewStage::Paused));
  ///
  /// // Can resume from paused to any active stage
  /// assert!(InterviewStage::Paused.can_transition_to(InterviewStage::Discovery));
  /// assert!(InterviewStage::Paused.can_transition_to(InterviewStage::Refinement));
  /// assert!(InterviewStage::Paused.can_transition_to(InterviewStage::Validation));
  ///
  /// // Cannot go backwards
  /// assert!(!InterviewStage::Refinement.can_transition_to(InterviewStage::Discovery));
  /// assert!(!InterviewStage::Validation.can_transition_to(InterviewStage::Refinement));
  ///
  /// // Cannot transition out of Complete
  /// assert!(!InterviewStage::Complete.can_transition_to(InterviewStage::Discovery));
  ///
  /// // No-op transitions are always valid
  /// assert!(InterviewStage::Discovery.can_transition_to(InterviewStage::Discovery));
  /// ```
  #[must_use]
  pub const fn can_transition_to(&self, next: Self) -> bool {
    matches!(
      (*self, next),
      (
        Self::Discovery | Self::Paused,
        Self::Discovery | Self::Refinement | Self::Validation | Self::Paused
      ) | (
        Self::Refinement,
        Self::Refinement | Self::Validation | Self::Complete | Self::Paused
      ) | (
        Self::Validation,
        Self::Validation | Self::Complete | Self::Paused
      ) | (Self::Complete, Self::Complete)
    )
  }

  /// Transition to the next stage with exhaustive pattern matching.
  ///
  /// This method consumes the current stage and returns either the new stage
  /// or an error if the transition is invalid.
  ///
  /// # Errors
  ///
  /// Returns [`InterviewStageError::InvalidTransition`] if the transition
  /// violates the state machine rules.
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::InterviewStage;
  /// // Valid transition
  /// let stage = InterviewStage::Discovery;
  /// let next = stage.transition_to(InterviewStage::Refinement).unwrap();
  /// assert_eq!(next, InterviewStage::Refinement);
  ///
  /// // Invalid transition
  /// let result = InterviewStage::Complete.transition_to(InterviewStage::Discovery);
  /// assert!(result.is_err());
  /// ```
  pub fn transition_to(self, next: Self) -> Result<Self, InterviewStageError> {
    if self.can_transition_to(next) {
      Ok(next)
    } else {
      Err(InterviewStageError::InvalidTransition {
        from: self.as_str().to_string(),
        to: next.as_str().to_string(),
      })
    }
  }

  /// Check if this stage is terminal (Complete).
  ///
  /// A terminal stage cannot transition to any other stage except itself.
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::InterviewStage;
  /// assert!(InterviewStage::Complete.is_terminal());
  /// assert!(!InterviewStage::Discovery.is_terminal());
  /// assert!(!InterviewStage::Paused.is_terminal());
  /// ```
  #[must_use]
  pub const fn is_terminal(&self) -> bool {
    matches!(self, Self::Complete)
  }

  /// Check if this stage is active (not Complete or Paused).
  ///
  /// Active stages allow modifications to the session (adding answers,
  /// resolving gaps, etc.).
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::InterviewStage;
  /// assert!(InterviewStage::Discovery.is_active());
  /// assert!(InterviewStage::Refinement.is_active());
  /// assert!(InterviewStage::Validation.is_active());
  /// assert!(!InterviewStage::Complete.is_active());
  /// assert!(!InterviewStage::Paused.is_active());
  /// ```
  #[must_use]
  pub const fn is_active(&self) -> bool {
    matches!(self, Self::Discovery | Self::Refinement | Self::Validation)
  }

  /// Check if this stage is paused.
  ///
  /// A paused session cannot be modified until resumed.
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::InterviewStage;
  /// assert!(InterviewStage::Paused.is_paused());
  /// assert!(!InterviewStage::Discovery.is_paused());
  /// assert!(!InterviewStage::Complete.is_paused());
  /// ```
  #[must_use]
  pub const fn is_paused(&self) -> bool {
    matches!(self, Self::Paused)
  }
}

/// Errors for interview stage transitions.
///
/// This error type is returned when attempting an invalid stage transition
/// via [`InterviewStage::transition_to`].
///
/// # Examples
///
/// ```
/// # use clarity_web::intent::interview::types::{InterviewStage, InterviewStageError};
/// let result = InterviewStage::Complete.transition_to(InterviewStage::Discovery);
/// match result {
///     Err(InterviewStageError::InvalidTransition { from, to }) => {
///         assert_eq!(from, "complete");
///         assert_eq!(to, "discovery");
///     }
///     _ => panic!("Expected InvalidTransition error"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InterviewStageError {
  /// The requested transition violates the state machine rules.
  ///
  /// Contains the source stage (`from`) and target stage (`to`).
  #[error("invalid stage transition from '{from}' to '{to}'")]
  InvalidTransition { from: String, to: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Perspective {
  #[default]
  User,
  Developer,
  Ops,
  Security,
  Business,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum QuestionPriority {
  Critical,
  #[default]
  Important,
  NiceToHave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum QuestionCategory {
  #[default]
  HappyPath,
  ErrorCase,
  EdgeCase,
  Constraint,
  Dependency,
  NonFunctional,
}

#[cfg(test)]
mod tests {
  use super::*;

  // ============================================
  // InterviewStage Exhaustive Transition Tests
  // ============================================

  #[test]
  fn interview_stage_all_transitions_from_discovery() {
    let discovery = InterviewStage::Discovery;

    // Valid transitions from Discovery
    assert!(discovery.can_transition_to(InterviewStage::Refinement));
    assert!(discovery.can_transition_to(InterviewStage::Validation));
    assert!(discovery.can_transition_to(InterviewStage::Paused));
    assert!(discovery.can_transition_to(InterviewStage::Discovery)); // no-op

    // Invalid transitions from Discovery
    assert!(!discovery.can_transition_to(InterviewStage::Complete));
  }

  #[test]
  fn interview_stage_all_transitions_from_refinement() {
    let refinement = InterviewStage::Refinement;

    // Valid transitions from Refinement
    assert!(refinement.can_transition_to(InterviewStage::Validation));
    assert!(refinement.can_transition_to(InterviewStage::Complete));
    assert!(refinement.can_transition_to(InterviewStage::Paused));
    assert!(refinement.can_transition_to(InterviewStage::Refinement)); // no-op

    // Invalid transitions from Refinement
    assert!(!refinement.can_transition_to(InterviewStage::Discovery));
  }

  #[test]
  fn interview_stage_all_transitions_from_validation() {
    let validation = InterviewStage::Validation;

    // Valid transitions from Validation
    assert!(validation.can_transition_to(InterviewStage::Complete));
    assert!(validation.can_transition_to(InterviewStage::Paused));
    assert!(validation.can_transition_to(InterviewStage::Validation)); // no-op

    // Invalid transitions from Validation
    assert!(!validation.can_transition_to(InterviewStage::Discovery));
    assert!(!validation.can_transition_to(InterviewStage::Refinement));
  }

  #[test]
  fn interview_stage_all_transitions_from_paused() {
    let paused = InterviewStage::Paused;

    // Valid transitions from Paused (resume to any active stage)
    assert!(paused.can_transition_to(InterviewStage::Discovery));
    assert!(paused.can_transition_to(InterviewStage::Refinement));
    assert!(paused.can_transition_to(InterviewStage::Validation));
    assert!(paused.can_transition_to(InterviewStage::Paused)); // no-op

    // Invalid transitions from Paused
    assert!(!paused.can_transition_to(InterviewStage::Complete));
  }

  #[test]
  fn interview_stage_all_transitions_from_complete() {
    let complete = InterviewStage::Complete;

    // Complete is terminal - no transitions out except no-op
    assert!(complete.can_transition_to(InterviewStage::Complete)); // no-op only

    // All other transitions invalid
    assert!(!complete.can_transition_to(InterviewStage::Discovery));
    assert!(!complete.can_transition_to(InterviewStage::Refinement));
    assert!(!complete.can_transition_to(InterviewStage::Validation));
    assert!(!complete.can_transition_to(InterviewStage::Paused));
  }

  #[test]
  fn interview_stage_transition_to_returns_correct_result() {
    // Valid transitions
    assert_eq!(
      InterviewStage::Discovery.transition_to(InterviewStage::Refinement),
      Ok(InterviewStage::Refinement)
    );
    assert_eq!(
      InterviewStage::Discovery.transition_to(InterviewStage::Validation),
      Ok(InterviewStage::Validation)
    );
    assert_eq!(
      InterviewStage::Refinement.transition_to(InterviewStage::Complete),
      Ok(InterviewStage::Complete)
    );
    assert_eq!(
      InterviewStage::Paused.transition_to(InterviewStage::Discovery),
      Ok(InterviewStage::Discovery)
    );

    // Invalid transitions
    assert!(InterviewStage::Complete
      .transition_to(InterviewStage::Discovery)
      .is_err());
    assert!(InterviewStage::Discovery
      .transition_to(InterviewStage::Complete)
      .is_err());
    assert!(InterviewStage::Validation
      .transition_to(InterviewStage::Discovery)
      .is_err());
  }

  #[test]
  fn interview_stage_no_op_transitions() {
    // All stages should allow staying in the same state
    for stage in [
      InterviewStage::Discovery,
      InterviewStage::Refinement,
      InterviewStage::Validation,
      InterviewStage::Complete,
      InterviewStage::Paused,
    ] {
      assert!(stage.can_transition_to(stage));
      assert_eq!(stage.transition_to(stage), Ok(stage));
    }
  }

  #[test]
  fn interview_stage_is_terminal() {
    assert!(InterviewStage::Complete.is_terminal());
    assert!(!InterviewStage::Discovery.is_terminal());
    assert!(!InterviewStage::Refinement.is_terminal());
    assert!(!InterviewStage::Validation.is_terminal());
    assert!(!InterviewStage::Paused.is_terminal());
  }

  #[test]
  fn interview_stage_is_active() {
    assert!(InterviewStage::Discovery.is_active());
    assert!(InterviewStage::Refinement.is_active());
    assert!(InterviewStage::Validation.is_active());
    assert!(!InterviewStage::Complete.is_active());
    assert!(!InterviewStage::Paused.is_active());
  }

  #[test]
  fn interview_stage_is_paused() {
    assert!(InterviewStage::Paused.is_paused());
    assert!(!InterviewStage::Discovery.is_paused());
    assert!(!InterviewStage::Refinement.is_paused());
    assert!(!InterviewStage::Validation.is_paused());
    assert!(!InterviewStage::Complete.is_paused());
  }

  #[test]
  fn interview_stage_as_str() {
    assert_eq!(InterviewStage::Discovery.as_str(), "discovery");
    assert_eq!(InterviewStage::Refinement.as_str(), "refinement");
    assert_eq!(InterviewStage::Validation.as_str(), "validation");
    assert_eq!(InterviewStage::Complete.as_str(), "complete");
    assert_eq!(InterviewStage::Paused.as_str(), "paused");
  }

  #[test]
  fn interview_stage_default_is_discovery() {
    assert_eq!(InterviewStage::default(), InterviewStage::Discovery);
  }
}
