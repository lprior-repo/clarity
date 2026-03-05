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
  pub fn from_str(s: &str) -> Result<Self, ProfileParseError> {
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
    Self::from_str(s)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum InterviewStage {
  #[default]
  Discovery,
  Refinement,
  Validation,
  Complete,
  Paused,
}

impl InterviewStage {
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
  /// Valid transitions:
  /// - Discovery -> Refinement, Validation, Paused
  /// - Refinement -> Validation, Complete, Paused
  /// - Validation -> Complete, Paused
  /// - Paused -> Discovery, Refinement, Validation (resume to any active stage)
  /// - Complete -> Complete (terminal state, no transitions out)
  #[must_use]
  pub fn can_transition_to(&self, next: Self) -> bool {
    match (*self, next) {
      // No-op transitions (staying in same state) - always valid
      (Self::Discovery, Self::Discovery) => true,
      (Self::Refinement, Self::Refinement) => true,
      (Self::Validation, Self::Validation) => true,
      (Self::Complete, Self::Complete) => true,
      (Self::Paused, Self::Paused) => true,
      // Discovery can go to Refinement, Validation, or Paused
      (Self::Discovery, Self::Refinement) => true,
      (Self::Discovery, Self::Validation) => true,
      (Self::Discovery, Self::Paused) => true,
      // Refinement can go to Validation, Complete, or Paused
      (Self::Refinement, Self::Validation) => true,
      (Self::Refinement, Self::Complete) => true,
      (Self::Refinement, Self::Paused) => true,
      // Validation can go to Complete or Paused
      (Self::Validation, Self::Complete) => true,
      (Self::Validation, Self::Paused) => true,
      // Paused can resume to any active stage
      (Self::Paused, Self::Discovery) => true,
      (Self::Paused, Self::Refinement) => true,
      (Self::Paused, Self::Validation) => true,
      // Complete is terminal - no transitions out
      (Self::Complete, Self::Discovery) => false,
      (Self::Complete, Self::Refinement) => false,
      (Self::Complete, Self::Validation) => false,
      (Self::Complete, Self::Paused) => false,
      // All remaining invalid transitions (explicitly listed for exhaustiveness)
      (Self::Discovery, Self::Complete) => false,
      (Self::Refinement, Self::Discovery) => false,
      (Self::Validation, Self::Discovery) => false,
      (Self::Validation, Self::Refinement) => false,
      (Self::Paused, Self::Complete) => false,
    }
  }

  /// Transition to the next stage with exhaustive pattern matching.
  ///
  /// # Errors
  /// Returns `InterviewStageError::InvalidTransition` if the transition is not allowed.
  pub fn transition_to(self, next: Self) -> Result<Self, InterviewStageError> {
    match (self, next) {
      // No-op transitions (staying in same state) - always valid
      (Self::Discovery, Self::Discovery) => Ok(Self::Discovery),
      (Self::Refinement, Self::Refinement) => Ok(Self::Refinement),
      (Self::Validation, Self::Validation) => Ok(Self::Validation),
      (Self::Complete, Self::Complete) => Ok(Self::Complete),
      (Self::Paused, Self::Paused) => Ok(Self::Paused),
      // Discovery can go to Refinement, Validation, or Paused
      (Self::Discovery, Self::Refinement) => Ok(Self::Refinement),
      (Self::Discovery, Self::Validation) => Ok(Self::Validation),
      (Self::Discovery, Self::Paused) => Ok(Self::Paused),
      // Refinement can go to Validation, Complete, or Paused
      (Self::Refinement, Self::Validation) => Ok(Self::Validation),
      (Self::Refinement, Self::Complete) => Ok(Self::Complete),
      (Self::Refinement, Self::Paused) => Ok(Self::Paused),
      // Validation can go to Complete or Paused
      (Self::Validation, Self::Complete) => Ok(Self::Complete),
      (Self::Validation, Self::Paused) => Ok(Self::Paused),
      // Paused can resume to any active stage
      (Self::Paused, Self::Discovery) => Ok(Self::Discovery),
      (Self::Paused, Self::Refinement) => Ok(Self::Refinement),
      (Self::Paused, Self::Validation) => Ok(Self::Validation),
      // All invalid transitions explicitly rejected
      (Self::Complete, Self::Discovery) => Err(InterviewStageError::InvalidTransition {
        from: "complete".to_string(),
        to: "discovery".to_string(),
      }),
      (Self::Complete, Self::Refinement) => Err(InterviewStageError::InvalidTransition {
        from: "complete".to_string(),
        to: "refinement".to_string(),
      }),
      (Self::Complete, Self::Validation) => Err(InterviewStageError::InvalidTransition {
        from: "complete".to_string(),
        to: "validation".to_string(),
      }),
      (Self::Complete, Self::Paused) => Err(InterviewStageError::InvalidTransition {
        from: "complete".to_string(),
        to: "paused".to_string(),
      }),
      (Self::Discovery, Self::Complete) => Err(InterviewStageError::InvalidTransition {
        from: "discovery".to_string(),
        to: "complete".to_string(),
      }),
      (Self::Refinement, Self::Discovery) => Err(InterviewStageError::InvalidTransition {
        from: "refinement".to_string(),
        to: "discovery".to_string(),
      }),
      (Self::Validation, Self::Discovery) => Err(InterviewStageError::InvalidTransition {
        from: "validation".to_string(),
        to: "discovery".to_string(),
      }),
      (Self::Validation, Self::Refinement) => Err(InterviewStageError::InvalidTransition {
        from: "validation".to_string(),
        to: "refinement".to_string(),
      }),
      (Self::Paused, Self::Complete) => Err(InterviewStageError::InvalidTransition {
        from: "paused".to_string(),
        to: "complete".to_string(),
      }),
    }
  }

  /// Check if this stage is terminal (Complete).
  #[must_use]
  pub const fn is_terminal(&self) -> bool {
    matches!(self, Self::Complete)
  }

  /// Check if this stage is active (not Complete or Paused).
  #[must_use]
  pub const fn is_active(&self) -> bool {
    matches!(self, Self::Discovery | Self::Refinement | Self::Validation)
  }

  /// Check if this stage is paused.
  #[must_use]
  pub const fn is_paused(&self) -> bool {
    matches!(self, Self::Paused)
  }
}

/// Errors for interview stage transitions.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InterviewStageError {
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
    assert!(InterviewStage::Complete.transition_to(InterviewStage::Discovery).is_err());
    assert!(InterviewStage::Discovery.transition_to(InterviewStage::Complete).is_err());
    assert!(InterviewStage::Validation.transition_to(InterviewStage::Discovery).is_err());
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
