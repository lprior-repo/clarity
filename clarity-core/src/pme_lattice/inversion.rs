//! Inversion Module - First Principle: Avoid Stupidity Framework
//!
//! Defines cognitive biases and systematic methods to avoid stupid decisions
//! in product design through inversion thinking (thinking backward from failure).

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::missing_const_for_fn)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// COGNITIVE BIAS TYPES
// ============================================================================

/// Common cognitive biases in product decisions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitiveBias {
  /// Seeking information that confirms existing beliefs
  ConfirmationBias,
  /// Focusing on successes while ignoring failures
  SurvivorshipBias,
  /// Continuing an endeavor due to previously invested resources
  SunkCostFallacy,
  /// Overweighting information that is recent or vivid
  AvailabilityHeuristic,
  /// Over-relying on the first piece of information received
  AnchoringBias,
  /// Underestimating risks and overestimating positive outcomes
  OptimismBias,
  /// Adopting beliefs because others hold them
  BandwagonEffect,
  /// Overestimating competence in areas of low expertise
  DunningKruger,
}

impl fmt::Display for CognitiveBias {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::ConfirmationBias => write!(f, "Confirmation Bias"),
      Self::SurvivorshipBias => write!(f, "Survivorship Bias"),
      Self::SunkCostFallacy => write!(f, "Sunk Cost Fallacy"),
      Self::AvailabilityHeuristic => write!(f, "Availability Heuristic"),
      Self::AnchoringBias => write!(f, "Anchoring Bias"),
      Self::OptimismBias => write!(f, "Optimism Bias"),
      Self::BandwagonEffect => write!(f, "Bandwagon Effect"),
      Self::DunningKruger => write!(f, "Dunning-Kruger Effect"),
    }
  }
}

// ============================================================================
// INVERSION CATEGORY TYPES
// ============================================================================

/// Categories for inversion questions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InversionCategory {
  /// Market-related failures
  MarketFailure,
  /// Product-related failures
  ProductFailure,
  /// Team-related failures
  TeamFailure,
  /// Execution-related failures
  ExecutionFailure,
  /// Competition-related failures
  CompetitionFailure,
}

impl fmt::Display for InversionCategory {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::MarketFailure => write!(f, "Market Failure"),
      Self::ProductFailure => write!(f, "Product Failure"),
      Self::TeamFailure => write!(f, "Team Failure"),
      Self::ExecutionFailure => write!(f, "Execution Failure"),
      Self::CompetitionFailure => write!(f, "Competition Failure"),
    }
  }
}

// ============================================================================
// INVERSION QUESTION
// ============================================================================

/// Question designed to think backward from failure scenarios
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InversionQuestion {
  /// Unique identifier
  pub id: Uuid,
  /// Category of the inversion question
  pub category: InversionCategory,
  /// The question to ask
  pub question: String,
  /// Description of what failure looks like
  pub negative_scenario: String,
  /// Strategy to prevent this failure
  pub prevention_strategy: Option<String>,
  /// When the question was created
  pub created_at: DateTime<Utc>,
}

impl InversionQuestion {
  /// Create a new inversion question
  ///
  /// # Errors
  /// Returns `InversionError::EmptyField` if `question` or `negative_scenario` is empty
  pub fn new(
    category: InversionCategory,
    question: String,
    negative_scenario: String,
  ) -> Result<Self, InversionError> {
    if question.trim().is_empty() {
      return Err(InversionError::EmptyField {
        field: "question".to_string(),
      });
    }
    if negative_scenario.trim().is_empty() {
      return Err(InversionError::EmptyField {
        field: "negative_scenario".to_string(),
      });
    }

    Ok(Self {
      id: Uuid::new_v4(),
      category,
      question,
      negative_scenario,
      prevention_strategy: None,
      created_at: Utc::now(),
    })
  }

  /// Add a prevention strategy
  #[must_use]
  pub fn with_prevention_strategy(mut self, strategy: String) -> Self {
    self.prevention_strategy = Some(strategy);
    self
  }
}

// ============================================================================
// STUPIDITY CHECK
// ============================================================================

/// Checklist item to detect and avoid cognitive biases
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StupidityCheck {
  /// Unique identifier
  pub id: Uuid,
  /// The bias this check addresses
  pub bias_type: CognitiveBias,
  /// The question to ask
  pub check_question: String,
  /// Whether the check passed (None = not answered yet)
  pub passed: Option<bool>,
  /// Evidence supporting the pass/fail decision
  pub evidence: Option<String>,
  /// When the check was created
  pub created_at: DateTime<Utc>,
}

impl StupidityCheck {
  /// Create a new unanswered stupidity check
  #[must_use]
  pub fn new(bias_type: CognitiveBias, check_question: String) -> Self {
    Self {
      id: Uuid::new_v4(),
      bias_type,
      check_question,
      passed: None,
      evidence: None,
      created_at: Utc::now(),
    }
  }

  /// Mark the check as passed
  #[must_use]
  pub fn pass(mut self) -> Self {
    self.passed = Some(true);
    self
  }

  /// Mark the check as failed
  #[must_use]
  pub fn fail(mut self) -> Self {
    self.passed = Some(false);
    self
  }

  /// Add evidence for the decision
  #[must_use]
  pub fn with_evidence(mut self, evidence: String) -> Self {
    self.evidence = Some(evidence);
    self
  }
}

// ============================================================================
// INVERSION ANALYSIS
// ============================================================================

/// Result of running an inversion analysis on a scenario
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InversionAnalysis {
  /// Unique identifier
  pub id: Uuid,
  /// Optional scenario being analyzed
  pub scenario_id: Option<Uuid>,
  /// Cognitive biases detected in the scenario
  pub biases_detected: Vec<CognitiveBias>,
  /// Stupidity checks performed
  pub checks: Vec<StupidityCheck>,
  /// Inversion questions asked
  pub inversion_questions: Vec<InversionQuestion>,
  /// Failure modes identified through inversion
  pub failure_modes_identified: Vec<String>,
  /// Strategies to prevent identified failures
  pub prevention_strategies: Vec<String>,
  /// Creation timestamp
  pub created_at: DateTime<Utc>,
  /// Last update timestamp
  pub updated_at: DateTime<Utc>,
}

impl InversionAnalysis {
  /// Create a new empty inversion analysis
  #[must_use]
  pub fn new() -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      scenario_id: None,
      biases_detected: Vec::new(),
      checks: Vec::new(),
      inversion_questions: Vec::new(),
      failure_modes_identified: Vec::new(),
      prevention_strategies: Vec::new(),
      created_at: now,
      updated_at: now,
    }
  }

  /// Associate with a scenario
  #[must_use]
  pub fn with_scenario(mut self, scenario_id: Uuid) -> Self {
    self.scenario_id = Some(scenario_id);
    self.updated_at = Utc::now();
    self
  }

  /// Add a detected cognitive bias
  #[must_use]
  pub fn with_bias(mut self, bias: CognitiveBias) -> Self {
    if !self.biases_detected.contains(&bias) {
      self.biases_detected.push(bias);
      self.updated_at = Utc::now();
    }
    self
  }

  /// Add a stupidity check
  #[must_use]
  pub fn with_check(mut self, check: StupidityCheck) -> Self {
    self.checks.push(check);
    self.updated_at = Utc::now();
    self
  }

  /// Add an inversion question
  #[must_use]
  pub fn with_question(mut self, question: InversionQuestion) -> Self {
    self.inversion_questions.push(question);
    self.updated_at = Utc::now();
    self
  }

  /// Add a failure mode
  #[must_use]
  pub fn with_failure_mode(mut self, failure_mode: String) -> Self {
    self.failure_modes_identified.push(failure_mode);
    self.updated_at = Utc::now();
    self
  }

  /// Add a prevention strategy
  #[must_use]
  pub fn with_prevention(mut self, strategy: String) -> Self {
    self.prevention_strategies.push(strategy);
    self.updated_at = Utc::now();
    self
  }

  /// Check if all checks have passed
  #[must_use]
  pub fn all_checks_passed(&self) -> bool {
    self.checks.iter().all(|check| check.passed == Some(true))
  }

  /// Check if there are blocking issues (failed or unanswered checks)
  #[must_use]
  pub fn has_blocking_issues(&self) -> bool {
    self.checks.iter().any(|check| check.passed != Some(true))
  }

  /// Count failed checks
  #[must_use]
  pub fn failed_checks_count(&self) -> usize {
    self
      .checks
      .iter()
      .filter(|check| check.passed == Some(false))
      .count()
  }

  /// Count passed checks
  #[must_use]
  pub fn passed_checks_count(&self) -> usize {
    self
      .checks
      .iter()
      .filter(|check| check.passed == Some(true))
      .count()
  }

  /// Count unanswered checks
  #[must_use]
  pub fn unanswered_checks_count(&self) -> usize {
    self
      .checks
      .iter()
      .filter(|check| check.passed.is_none())
      .count()
  }

  /// Calculate risk score based on biases and check results
  ///
  /// Returns a value between 0.0 (low risk) and 1.0 (high risk)
  #[must_use]
  pub fn calculate_risk_score(&self) -> f32 {
    if self.checks.is_empty() && self.biases_detected.is_empty() {
      return 0.0;
    }

    let bias_penalty = (self.biases_detected.len() as f32 * 0.1).min(0.3);

    let failed_ratio = if self.checks.is_empty() {
      0.0
    } else {
      self.failed_checks_count() as f32 / self.checks.len() as f32
    };

    let unanswered_ratio = if self.checks.is_empty() {
      0.0
    } else {
      self.unanswered_checks_count() as f32 / self.checks.len() as f32
    };

    let check_penalty = failed_ratio * 0.5 + unanswered_ratio * 0.3;

    (bias_penalty + check_penalty).clamp(0.0, 1.0)
  }
}

impl Default for InversionAnalysis {
  fn default() -> Self {
    Self::new()
  }
}

// ============================================================================
// ERRORS
// ============================================================================

/// Errors for the inversion module
#[derive(Debug, Error, PartialEq, Eq)]
pub enum InversionError {
  /// A required field was empty
  #[error("required field is empty: {field}")]
  EmptyField { field: String },

  /// Validation failed
  #[error("validation failed: {0}")]
  ValidationFailed(String),
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn all_cognitive_biases_have_display() {
    let biases = [
      CognitiveBias::ConfirmationBias,
      CognitiveBias::SurvivorshipBias,
      CognitiveBias::SunkCostFallacy,
      CognitiveBias::AvailabilityHeuristic,
      CognitiveBias::AnchoringBias,
      CognitiveBias::OptimismBias,
      CognitiveBias::BandwagonEffect,
      CognitiveBias::DunningKruger,
    ];

    for bias in biases {
      let display = bias.to_string();
      assert!(!display.is_empty());
    }
  }

  #[test]
  fn all_inversion_categories_have_display() {
    let categories = [
      InversionCategory::MarketFailure,
      InversionCategory::ProductFailure,
      InversionCategory::TeamFailure,
      InversionCategory::ExecutionFailure,
      InversionCategory::CompetitionFailure,
    ];

    for category in categories {
      let display = category.to_string();
      assert!(!display.is_empty());
    }
  }
}
