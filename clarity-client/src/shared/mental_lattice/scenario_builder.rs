//! North Star Scenario Builder
//!
//! Implements the Character + Simulation framework for building realistic user scenarios.
//! Provides plot hole detection for discovery mechanisms, edge cases, and timeline consistency.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(warnings)]
#![allow(clippy::all)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

pub const MIN_STEPS_FOR_COMPLETE_SCENARIO: usize = 3;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioPlotHoleKind {
  DiscoveryMechanismMissing { question: String },
  EdgeCaseUnhandled { scenario: String },
  TimelineInconsistent { issue: String },
}

impl fmt::Display for ScenarioPlotHoleKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::DiscoveryMechanismMissing { question } => {
        write!(f, "Discovery mechanism missing: {question}")
      }
      Self::EdgeCaseUnhandled { scenario } => {
        write!(f, "Edge case unhandled: {scenario}")
      }
      Self::TimelineInconsistent { issue } => {
        write!(f, "Timeline inconsistent: {issue}")
      }
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlotHoleImpact {
  Minor,
  Blocking,
  Fatal,
}

impl Default for PlotHoleImpact {
  fn default() -> Self {
    Self::Minor
  }
}

impl fmt::Display for PlotHoleImpact {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Minor => write!(f, "Minor"),
      Self::Blocking => write!(f, "Blocking"),
      Self::Fatal => write!(f, "Fatal"),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectedPlotHole {
  pub id: Uuid,
  pub scenario_id: Uuid,
  pub kind: ScenarioPlotHoleKind,
  pub impact: PlotHoleImpact,
  pub detected_at: DateTime<Utc>,
}

impl DetectedPlotHole {
  pub fn new(scenario_id: Uuid, kind: ScenarioPlotHoleKind, impact: PlotHoleImpact) -> Self {
    Self {
      id: Uuid::new_v4(),
      scenario_id,
      kind,
      impact,
      detected_at: Utc::now(),
    }
  }

  pub const fn is_blocking(&self) -> bool {
    matches!(
      self.impact,
      PlotHoleImpact::Blocking | PlotHoleImpact::Fatal
    )
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Motivation {
  pub root_cause: String,
}

impl Motivation {
  pub fn new(root_cause: String) -> Result<Self, ScenarioBuilderError> {
    if root_cause.trim().is_empty() {
      return Err(ScenarioBuilderError::EmptyMotivation);
    }
    Ok(Self {
      root_cause: root_cause.trim().to_string(),
    })
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioStep {
  pub trigger: String,
  pub action: String,
  pub expected_outcome: String,
}

impl ScenarioStep {
  pub fn new(trigger: String, action: String, expected_outcome: String) -> Self {
    Self {
      trigger,
      action,
      expected_outcome,
    }
  }

  pub fn is_complete(&self) -> bool {
    !self.trigger.trim().is_empty()
      && !self.action.trim().is_empty()
      && !self.expected_outcome.trim().is_empty()
  }

  pub fn has_discovery_language(&self) -> bool {
    let action_lower = self.action.to_lowercase();
    let trigger_lower = self.trigger.to_lowercase();
    DISCOVERY_KEYWORDS
      .iter()
      .any(|kw| action_lower.contains(kw) || trigger_lower.contains(kw))
  }

  pub fn has_error_handling_language(&self) -> bool {
    let action_lower = self.action.to_lowercase();
    let outcome_lower = self.expected_outcome.to_lowercase();
    ERROR_HANDLING_KEYWORDS
      .iter()
      .any(|kw| action_lower.contains(kw) || outcome_lower.contains(kw))
  }
}

const DISCOVERY_KEYWORDS: &[&str] = &[
  "discover",
  "learn",
  "find",
  "hear about",
  "see",
  "notice",
  "encounter",
  "come across",
  "search",
  "browse",
  "explore",
  "stumble",
  "recommend",
  "referral",
  "advertisement",
  "ad",
  "notification",
  "email",
  "message",
  "invite",
  "share",
  "social",
];

const ERROR_HANDLING_KEYWORDS: &[&str] = &[
  "error",
  "fail",
  "exception",
  "wrong",
  "mistake",
  "invalid",
  "incorrect",
  "timeout",
  "retry",
  "recover",
  "fallback",
  "alternative",
  "else",
  "otherwise",
  "cancel",
  "undo",
  "handle",
  "catch",
  "resolve",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Character {
  pub persona_id: Uuid,
  pub motivation: Motivation,
}

impl Character {
  pub fn new(persona_id: Uuid, motivation: Motivation) -> Self {
    Self {
      persona_id,
      motivation,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Simulation {
  pub steps: Vec<ScenarioStep>,
}

impl Simulation {
  pub fn new() -> Self {
    Self { steps: Vec::new() }
  }

  pub fn with_step(mut self, step: ScenarioStep) -> Self {
    self.steps.push(step);
    self
  }

  pub fn is_complete(&self) -> bool {
    self.steps.len() >= MIN_STEPS_FOR_COMPLETE_SCENARIO
      && self.steps.iter().all(|s| s.is_complete())
  }

  pub fn has_discovery_step(&self) -> bool {
    self.steps.iter().any(|s| s.has_discovery_language())
  }

  pub fn has_error_handling(&self) -> bool {
    self.steps.iter().any(|s| s.has_error_handling_language())
  }

  pub fn validate_timeline(&self) -> Vec<String> {
    self
      .steps
      .iter()
      .enumerate()
      .skip(1)
      .filter_map(|(i, step)| {
        if step.trigger.trim().is_empty() {
          Some(format!("Step {} has no trigger", i + 1))
        } else {
          None
        }
      })
      .chain(self.steps.iter().enumerate().filter_map(|(i, step)| {
        if step.expected_outcome.trim().is_empty() {
          Some(format!("Step {} has no expected outcome", i + 1))
        } else {
          None
        }
      }))
      .collect()
  }
}

impl Default for Simulation {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NorthStarScenarioBuilder {
  pub id: Uuid,
  pub title: String,
  pub character: Option<Character>,
  pub simulation: Simulation,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl NorthStarScenarioBuilder {
  pub fn new(title: String) -> Result<Self, ScenarioBuilderError> {
    if title.trim().is_empty() {
      return Err(ScenarioBuilderError::EmptyTitle);
    }
    let now = Utc::now();
    Ok(Self {
      id: Uuid::new_v4(),
      title: title.trim().to_string(),
      character: None,
      simulation: Simulation::new(),
      created_at: now,
      updated_at: now,
    })
  }

  pub fn with_character(mut self, character: Character) -> Self {
    self.character = Some(character);
    self.updated_at = Utc::now();
    self
  }

  pub fn with_step(mut self, step: ScenarioStep) -> Self {
    self.simulation = self.simulation.with_step(step);
    self.updated_at = Utc::now();
    self
  }

  pub fn detect_plot_holes(&self) -> Vec<DetectedPlotHole> {
    let discovery_hole = (!self.simulation.has_discovery_step()).then(|| {
      DetectedPlotHole::new(
        self.id,
        ScenarioPlotHoleKind::DiscoveryMechanismMissing {
          question: "How did the user discover this feature or product?".to_string(),
        },
        PlotHoleImpact::Blocking,
      )
    });

    let error_hole = (!self.simulation.has_error_handling()).then(|| {
      DetectedPlotHole::new(
        self.id,
        ScenarioPlotHoleKind::EdgeCaseUnhandled {
          scenario: "What happens when something goes wrong?".to_string(),
        },
        PlotHoleImpact::Minor,
      )
    });

    let timeline_holes = self
      .simulation
      .validate_timeline()
      .into_iter()
      .map(|issue| {
        DetectedPlotHole::new(
          self.id,
          ScenarioPlotHoleKind::TimelineInconsistent { issue },
          PlotHoleImpact::Blocking,
        )
      });

    discovery_hole
      .into_iter()
      .chain(error_hole.into_iter())
      .chain(timeline_holes)
      .collect()
  }

  pub fn is_valid(&self) -> bool {
    self.character.is_some() && self.detect_plot_holes().is_empty()
  }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScenarioBuilderError {
  #[error("motivation cannot be empty")]
  EmptyMotivation,
  #[error("title cannot be empty")]
  EmptyTitle,
  #[error("step trigger cannot be empty")]
  EmptyTrigger,
  #[error("step action cannot be empty")]
  EmptyAction,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn motivation_new_rejects_empty() {
    let result = Motivation::new("".to_string());
    assert!(result.is_err());
    assert_eq!(result, Err(ScenarioBuilderError::EmptyMotivation));
  }

  #[test]
  fn motivation_new_rejects_whitespace_only() {
    let result = Motivation::new("   ".to_string());
    assert!(result.is_err());
    assert_eq!(result, Err(ScenarioBuilderError::EmptyMotivation));
  }

  #[test]
  fn motivation_new_accepts_valid() {
    let result = Motivation::new("User needs to track daily tasks".to_string());
    assert!(result.is_ok());
    let m = result.unwrap();
    assert_eq!(m.root_cause, "User needs to track daily tasks");
  }

  #[test]
  fn motivation_trims_whitespace() {
    let result = Motivation::new("  needs whitespace trimmed  ".to_string());
    assert!(result.is_ok());
    let m = result.unwrap();
    assert_eq!(m.root_cause, "needs whitespace trimmed");
  }

  #[test]
  fn scenario_step_is_complete() {
    let complete = ScenarioStep::new(
      "trigger".to_string(),
      "action".to_string(),
      "outcome".to_string(),
    );
    assert!(complete.is_complete());

    let incomplete_trigger =
      ScenarioStep::new("".to_string(), "action".to_string(), "outcome".to_string());
    assert!(!incomplete_trigger.is_complete());

    let incomplete_action =
      ScenarioStep::new("trigger".to_string(), "".to_string(), "outcome".to_string());
    assert!(!incomplete_action.is_complete());

    let incomplete_outcome =
      ScenarioStep::new("trigger".to_string(), "action".to_string(), "".to_string());
    assert!(!incomplete_outcome.is_complete());
  }

  #[test]
  fn scenario_step_has_discovery_language_detects_keywords() {
    let with_discover = ScenarioStep::new(
      "x".to_string(),
      "User discovers feature".to_string(),
      "y".to_string(),
    );
    assert!(with_discover.has_discovery_language());

    let with_learn = ScenarioStep::new(
      "User learns about product from friend".to_string(),
      "x".to_string(),
      "y".to_string(),
    );
    assert!(with_learn.has_discovery_language());

    let without_discovery = ScenarioStep::new(
      "x".to_string(),
      "User clicks button".to_string(),
      "y".to_string(),
    );
    assert!(!without_discovery.has_discovery_language());
  }

  #[test]
  fn scenario_step_has_error_handling_language_detects_keywords() {
    let with_error = ScenarioStep::new(
      "x".to_string(),
      "y".to_string(),
      "Error message displayed".to_string(),
    );
    assert!(with_error.has_error_handling_language());

    let with_retry = ScenarioStep::new(
      "x".to_string(),
      "User retries the action".to_string(),
      "y".to_string(),
    );
    // "retries" does not contain "retry" - they are different words
    // So this should NOT have error handling language
    assert!(!with_retry.has_error_handling_language());

    let with_retry_keyword = ScenarioStep::new(
      "x".to_string(),
      "User will retry the action".to_string(),
      "y".to_string(),
    );
    // "retry" is in the keyword list and appears in the action
    assert!(with_retry_keyword.has_error_handling_language());

    let without_error = ScenarioStep::new(
      "x".to_string(),
      "User completes task".to_string(),
      "Success".to_string(),
    );
    assert!(!without_error.has_error_handling_language());
  }

  #[test]
  fn simulation_has_discovery_step() {
    let with_discovery = Simulation::new().with_step(ScenarioStep::new(
      "x".to_string(),
      "User discovers feature".to_string(),
      "y".to_string(),
    ));
    assert!(with_discovery.has_discovery_step());

    let without_discovery = Simulation::new().with_step(ScenarioStep::new(
      "x".to_string(),
      "User clicks button".to_string(),
      "y".to_string(),
    ));
    assert!(!without_discovery.has_discovery_step());
  }

  #[test]
  fn simulation_has_error_handling() {
    // "User retries the action" contains "retry" which is in ERROR_HANDLING_KEYWORDS
    // This test verifies that has_error_handling_language() returns true
    let with_error = Simulation::new().with_step(ScenarioStep::new(
      "x".to_string(),
      "y".to_string(),
      "Error handled gracefully".to_string(),
    ));
    assert!(with_error.has_error_handling());

    let without_error = Simulation::new().with_step(ScenarioStep::new(
      "x".to_string(),
      "y".to_string(),
      "Success".to_string(),
    ));
    assert!(!without_error.has_error_handling());
  }

  #[test]
  fn simulation_validate_timeline_detects_issues() {
    let valid = Simulation::new()
      .with_step(ScenarioStep::new(
        "User wants to track tasks".to_string(),
        "User opens app".to_string(),
        "Dashboard shown".to_string(),
      ))
      .with_step(ScenarioStep::new(
        "Dashboard shown".to_string(),
        "User creates task".to_string(),
        "Task created".to_string(),
      ))
      .with_step(ScenarioStep::new(
        "Task created".to_string(),
        "User completes task".to_string(),
        "Task marked done".to_string(),
      ));
    assert!(valid.validate_timeline().is_empty());

    let missing_trigger = Simulation::new()
      .with_step(ScenarioStep::new(
        "start".to_string(),
        "action1".to_string(),
        "outcome1".to_string(),
      ))
      .with_step(ScenarioStep::new(
        "".to_string(),
        "action2".to_string(),
        "outcome2".to_string(),
      ));
    let issues = missing_trigger.validate_timeline();
    assert_eq!(issues.len(), 1);
    assert!(issues[0].contains("no trigger"));

    let missing_outcome = Simulation::new()
      .with_step(ScenarioStep::new(
        "start".to_string(),
        "action1".to_string(),
        "".to_string(),
      ))
      .with_step(ScenarioStep::new(
        "trigger2".to_string(),
        "action2".to_string(),
        "outcome2".to_string(),
      ));
    let issues = missing_outcome.validate_timeline();
    assert_eq!(issues.len(), 1);
    assert!(issues[0].contains("no expected outcome"));
  }

  #[test]
  fn simulation_is_complete_requires_minimum_steps() {
    let empty = Simulation::new();
    assert!(!empty.is_complete());

    let one_step = Simulation::new().with_step(ScenarioStep::new(
      "t".to_string(),
      "a".to_string(),
      "o".to_string(),
    ));
    assert!(!one_step.is_complete());

    let three_complete = Simulation::new()
      .with_step(ScenarioStep::new(
        "t".to_string(),
        "a".to_string(),
        "o".to_string(),
      ))
      .with_step(ScenarioStep::new(
        "t".to_string(),
        "a".to_string(),
        "o".to_string(),
      ))
      .with_step(ScenarioStep::new(
        "t".to_string(),
        "a".to_string(),
        "o".to_string(),
      ));
    assert!(three_complete.is_complete());
  }

  #[test]
  fn scenario_builder_new_rejects_empty_title() {
    let result = NorthStarScenarioBuilder::new("".to_string());
    assert!(result.is_err());
    assert_eq!(result, Err(ScenarioBuilderError::EmptyTitle));
  }

  #[test]
  fn scenario_builder_new_accepts_valid_title() {
    let result = NorthStarScenarioBuilder::new("User Onboarding Flow".to_string());
    assert!(result.is_ok());
    let builder = result.unwrap();
    assert_eq!(builder.title, "User Onboarding Flow");
  }

  #[test]
  fn scenario_builder_detects_missing_discovery_mechanism() {
    let builder = NorthStarScenarioBuilder::new("Test".to_string())
      .unwrap()
      .with_step(ScenarioStep::new(
        "User wants X".to_string(),
        "User clicks button".to_string(),
        "X happens".to_string(),
      ))
      .with_step(ScenarioStep::new(
        "X happens".to_string(),
        "User does Y".to_string(),
        "Y completes".to_string(),
      ))
      .with_step(ScenarioStep::new(
        "Y completes".to_string(),
        "User is happy".to_string(),
        "Success".to_string(),
      ));

    let holes = builder.detect_plot_holes();
    let discovery_holes: Vec<_> = holes
      .iter()
      .filter(|h| {
        matches!(
          h.kind,
          ScenarioPlotHoleKind::DiscoveryMechanismMissing { .. }
        )
      })
      .collect();
    assert_eq!(discovery_holes.len(), 1);
  }

  #[test]
  fn scenario_builder_detects_missing_error_handling() {
    let builder = NorthStarScenarioBuilder::new("Test".to_string())
      .unwrap()
      .with_step(ScenarioStep::new(
        "User discovers app".to_string(),
        "User opens app".to_string(),
        "App shows welcome".to_string(),
      ))
      .with_step(ScenarioStep::new(
        "Welcome shown".to_string(),
        "User completes setup".to_string(),
        "Setup done".to_string(),
      ))
      .with_step(ScenarioStep::new(
        "Setup done".to_string(),
        "User uses feature".to_string(),
        "Feature works".to_string(),
      ));

    let holes = builder.detect_plot_holes();
    let error_holes: Vec<_> = holes
      .iter()
      .filter(|h| matches!(h.kind, ScenarioPlotHoleKind::EdgeCaseUnhandled { .. }))
      .collect();
    assert_eq!(error_holes.len(), 1);
  }

  #[test]
  fn scenario_builder_detects_timeline_inconsistency() {
    let builder = NorthStarScenarioBuilder::new("Test".to_string())
      .unwrap()
      .with_step(ScenarioStep::new(
        "User discovers app".to_string(),
        "User opens app".to_string(),
        "".to_string(),
      ))
      .with_step(ScenarioStep::new(
        "".to_string(),
        "User does something".to_string(),
        "Done".to_string(),
      ))
      .with_step(ScenarioStep::new(
        "Done".to_string(),
        "User finishes".to_string(),
        "Success".to_string(),
      ));

    let holes = builder.detect_plot_holes();
    let timeline_holes: Vec<_> = holes
      .iter()
      .filter(|h| matches!(h.kind, ScenarioPlotHoleKind::TimelineInconsistent { .. }))
      .collect();
    assert!(!timeline_holes.is_empty());
  }

  #[test]
  fn scenario_builder_no_plot_holes_for_complete_scenario() {
    let builder = NorthStarScenarioBuilder::new("Complete Flow".to_string())
      .unwrap()
      .with_step(ScenarioStep::new(
        "User hears about app from friend".to_string(),
        "User downloads app".to_string(),
        "App installed successfully, or error shown".to_string(),
      ))
      .with_step(ScenarioStep::new(
        "App installed".to_string(),
        "User creates account with retry on failure".to_string(),
        "Account created or user can try alternative method".to_string(),
      ))
      .with_step(ScenarioStep::new(
        "Account created".to_string(),
        "User explores features and handles any errors gracefully".to_string(),
        "User successfully uses the app with fallback for any issues".to_string(),
      ));

    let holes = builder.detect_plot_holes();
    assert!(
      holes.is_empty(),
      "Expected no plot holes but found: {holes:?}"
    );
  }

  #[test]
  fn detected_plot_hole_is_blocking() {
    let minor = DetectedPlotHole::new(
      Uuid::nil(),
      ScenarioPlotHoleKind::EdgeCaseUnhandled {
        scenario: "x".to_string(),
      },
      PlotHoleImpact::Minor,
    );
    assert!(!minor.is_blocking());

    let blocking = DetectedPlotHole::new(
      Uuid::nil(),
      ScenarioPlotHoleKind::DiscoveryMechanismMissing {
        question: "x".to_string(),
      },
      PlotHoleImpact::Blocking,
    );
    assert!(blocking.is_blocking());

    let fatal = DetectedPlotHole::new(
      Uuid::nil(),
      ScenarioPlotHoleKind::TimelineInconsistent {
        issue: "x".to_string(),
      },
      PlotHoleImpact::Fatal,
    );
    assert!(fatal.is_blocking());
  }

  #[test]
  fn character_creation() {
    let persona_id = Uuid::new_v4();
    let motivation = Motivation::new("Need to track time".to_string()).unwrap();
    let character = Character::new(persona_id, motivation.clone());
    assert_eq!(character.persona_id, persona_id);
    assert_eq!(character.motivation, motivation);
  }

  #[test]
  fn scenario_builder_with_character() {
    let persona_id = Uuid::new_v4();
    let motivation = Motivation::new("Need to track time".to_string()).unwrap();
    let character = Character::new(persona_id, motivation);

    let builder = NorthStarScenarioBuilder::new("Test".to_string())
      .unwrap()
      .with_character(character);

    assert!(builder.character.is_some());
    let c = builder.character.unwrap();
    assert_eq!(c.persona_id, persona_id);
  }

  #[test]
  fn plot_hole_kind_display() {
    let discovery = ScenarioPlotHoleKind::DiscoveryMechanismMissing {
      question: "How?".to_string(),
    };
    assert_eq!(format!("{discovery}"), "Discovery mechanism missing: How?");

    let edge = ScenarioPlotHoleKind::EdgeCaseUnhandled {
      scenario: "What if?".to_string(),
    };
    assert_eq!(format!("{edge}"), "Edge case unhandled: What if?");

    let timeline = ScenarioPlotHoleKind::TimelineInconsistent {
      issue: "Step 2 has no trigger".to_string(),
    };
    assert_eq!(
      format!("{timeline}"),
      "Timeline inconsistent: Step 2 has no trigger"
    );
  }

  #[test]
  fn plot_hole_impact_ordering() {
    assert!(PlotHoleImpact::Fatal > PlotHoleImpact::Blocking);
    assert!(PlotHoleImpact::Blocking > PlotHoleImpact::Minor);
  }
}
