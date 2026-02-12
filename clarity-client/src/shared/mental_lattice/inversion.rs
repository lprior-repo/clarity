//! First Principle: Avoid Stupidity
//!
//! From The Product-Minded Engineer:
//! "The best way to be smart is to avoid being stupid." - Charlie Munger
//!
//! This module implements the Inversion Thinking framework for the Mental Lattice.
//! It provides tools to detect cognitive biases and apply systematic methods
//! to avoid stupid decisions in product design.
//!
//! # Architecture
//!
//! The module consists of three main components:
//! - **CognitiveBias**: Types of biases that affect decision-making
//! - **BiasDetection**: Detected bias instances in decisions
//! - **AntiStupidityExercise**: Systematic methods to counteract biases
//!
//! # Core Concepts
//!
//! ## Cognitive Biases
//!
//! Systematic patterns of deviation from rationality:
//! - **Confirmation Bias**: Seeking only evidence that supports beliefs
//! - **Survivorship Bias**: Focusing on successes, ignoring failures
//! - **Sunk Cost Fallacy**: Continuing due to past investment
//! - **Planning Fallacy**: Underestimating time and costs
//! - **Anchoring**: Over-relying on first piece of information
//!
//! ## Anti-Stupidity Methods
//!
//! Systematic approaches to counteract biases:
//! - **Inversion Thinking**: "How do I fail?" instead of "How do I succeed?"
//! - **Pre-Mortem**: Assume failure, ask why it happened
//! - **Red Teaming**: Dedicated adversarial review
//! - **Steel Manning**: Argue against yourself strongest
//! - **Falsification Test**: What would prove this wrong?
//!
//! # Dependencies
//!
//! - `uuid`: For generating unique IDs
//! - `serde`: For serialization
//! - `chrono`: For timestamp handling
//! - `thiserror`: For error types

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Minimum confidence/risk score
pub const MIN_SCORE: f32 = 0.0;

/// Maximum confidence/risk score
pub const MAX_SCORE: f32 = 1.0;

/// Threshold for "likely stupid" classification
pub const STUPIDITY_THRESHOLD: f32 = 0.6;

/// Threshold for "critical" bias severity
pub const CRITICAL_SEVERITY_THRESHOLD: f32 = 0.8;

// ============================================================================
// COGNITIVE BIAS TYPES
// ============================================================================

/// Types of cognitive biases that affect decision-making
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitiveBias {
  ConfirmationBias,
  AvailabilityHeuristic,
  Anchoring,
  RecencyBias,
  SurvivorshipBias,
  SunkCostFallacy,
  GamblerFallacy,
  OverconfidenceBias,
  DunningKruger,
  PlanningFallacy,
  BandwagonEffect,
  AuthorityBias,
  Groupthink,
  HindsightBias,
  FalseConsensus,
  AttributionError,
}

impl fmt::Display for CognitiveBias {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::ConfirmationBias => write!(f, "Confirmation Bias"),
      Self::AvailabilityHeuristic => write!(f, "Availability Heuristic"),
      Self::Anchoring => write!(f, "Anchoring"),
      Self::RecencyBias => write!(f, "Recency Bias"),
      Self::SurvivorshipBias => write!(f, "Survivorship Bias"),
      Self::SunkCostFallacy => write!(f, "Sunk Cost Fallacy"),
      Self::GamblerFallacy => write!(f, "Gambler's Fallacy"),
      Self::OverconfidenceBias => write!(f, "Overconfidence Bias"),
      Self::DunningKruger => write!(f, "Dunning-Kruger Effect"),
      Self::PlanningFallacy => write!(f, "Planning Fallacy"),
      Self::BandwagonEffect => write!(f, "Bandwagon Effect"),
      Self::AuthorityBias => write!(f, "Authority Bias"),
      Self::Groupthink => write!(f, "Groupthink"),
      Self::HindsightBias => write!(f, "Hindsight Bias"),
      Self::FalseConsensus => write!(f, "False Consensus Effect"),
      Self::AttributionError => write!(f, "Fundamental Attribution Error"),
    }
  }
}

impl CognitiveBias {
  /// Get default severity for this bias type
  #[must_use]
  pub const fn default_severity(&self) -> BiasSeverity {
    match self {
      Self::ConfirmationBias
      | Self::SunkCostFallacy
      | Self::SurvivorshipBias
      | Self::PlanningFallacy => BiasSeverity::High,
      Self::OverconfidenceBias | Self::DunningKruger | Self::Groupthink => BiasSeverity::High,
      Self::Anchoring | Self::AvailabilityHeuristic | Self::RecencyBias | Self::BandwagonEffect => {
        BiasSeverity::Moderate
      }
      Self::GamblerFallacy
      | Self::AuthorityBias
      | Self::HindsightBias
      | Self::FalseConsensus
      | Self::AttributionError => BiasSeverity::Low,
    }
  }

  /// Get suggested mitigation for this bias type
  #[must_use]
  pub const fn suggested_mitigation(&self) -> &'static str {
    match self {
      Self::ConfirmationBias => "Actively seek disconfirming evidence",
      Self::AvailabilityHeuristic => "Check base rates and statistics",
      Self::Anchoring => "Get multiple independent estimates",
      Self::RecencyBias => "Review historical data before deciding",
      Self::SurvivorshipBias => "Study failures as well as successes",
      Self::SunkCostFallacy => "Evaluate future costs/benefits only",
      Self::GamblerFallacy => "Remember: past events don't affect independent ones",
      Self::OverconfidenceBias => "Track your prediction accuracy",
      Self::DunningKruger => "Seek expert review and feedback",
      Self::PlanningFallacy => "Use reference class forecasting",
      Self::BandwagonEffect => "Question if you'd decide the same alone",
      Self::AuthorityBias => "Evaluate arguments on merits, not source",
      Self::Groupthink => "Assign a devil's advocate",
      Self::HindsightBias => "Document predictions before outcomes",
      Self::FalseConsensus => "Survey actual opinions",
      Self::AttributionError => "Consider situational factors",
    }
  }
}

// ============================================================================
// BIAS SEVERITY
// ============================================================================

/// Severity level of a cognitive bias impact
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BiasSeverity {
  Low,
  Moderate,
  High,
  Critical,
}

impl Default for BiasSeverity {
  fn default() -> Self {
    Self::Moderate
  }
}

impl fmt::Display for BiasSeverity {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Low => write!(f, "Low"),
      Self::Moderate => write!(f, "Moderate"),
      Self::High => write!(f, "High"),
      Self::Critical => write!(f, "Critical"),
    }
  }
}

impl BiasSeverity {
  /// Convert severity to a numeric weight for risk calculation
  #[must_use]
  pub const fn weight(&self) -> f32 {
    match self {
      Self::Low => 0.25,
      Self::Moderate => 0.5,
      Self::High => 0.75,
      Self::Critical => 1.0,
    }
  }
}

// ============================================================================
// ANTI-STUPIDITY METHODS
// ============================================================================

/// Systematic methods to avoid stupid decisions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AntiStupidityMethod {
  InversionThinking,
  PreMortem,
  ReverseBrainstorming,
  RedTeaming,
  SteelManning,
  NullHypothesis,
  FalsificationTest,
  BaseRateCheck,
  OutsideView,
  SecondOrderThinking,
  IncentiveAudit,
  AssumptionAudit,
}

impl fmt::Display for AntiStupidityMethod {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InversionThinking => write!(f, "Inversion Thinking"),
      Self::PreMortem => write!(f, "Pre-Mortem Analysis"),
      Self::ReverseBrainstorming => write!(f, "Reverse Brainstorming"),
      Self::RedTeaming => write!(f, "Red Teaming"),
      Self::SteelManning => write!(f, "Steel Manning"),
      Self::NullHypothesis => write!(f, "Null Hypothesis Testing"),
      Self::FalsificationTest => write!(f, "Falsification Test"),
      Self::BaseRateCheck => write!(f, "Base Rate Check"),
      Self::OutsideView => write!(f, "Outside View"),
      Self::SecondOrderThinking => write!(f, "Second-Order Thinking"),
      Self::IncentiveAudit => write!(f, "Incentive Audit"),
      Self::AssumptionAudit => write!(f, "Assumption Audit"),
    }
  }
}

impl AntiStupidityMethod {
  /// Get description of what this method does
  #[must_use]
  pub const fn description(&self) -> &'static str {
    match self {
      Self::InversionThinking => "Think backwards from failure to find success",
      Self::PreMortem => "Assume the project failed, ask why",
      Self::ReverseBrainstorming => "How to cause the problem (to avoid it)",
      Self::RedTeaming => "Dedicated adversarial review",
      Self::SteelManning => "Argue against yourself with strongest points",
      Self::NullHypothesis => "What if the opposite is true?",
      Self::FalsificationTest => "What evidence would prove this wrong?",
      Self::BaseRateCheck => "Compare to population statistics",
      Self::OutsideView => "Look at similar cases, not just this one",
      Self::SecondOrderThinking => "What are consequences of consequences?",
      Self::IncentiveAudit => "What incentives are driving behavior?",
      Self::AssumptionAudit => "List and challenge all assumptions",
    }
  }
}

// ============================================================================
// ERRORS
// ============================================================================

/// Inversion domain errors
#[derive(Debug, Error, PartialEq)]
pub enum InversionError {
  #[error("required field is empty: {0}")]
  EmptyField(String),

  #[error("invalid score {0}: must be between 0.0 and 1.0")]
  InvalidScore(f32),

  #[error("no findings provided for exercise")]
  NoFindings,
}

// ============================================================================
// BIAS DETECTION
// ============================================================================

/// Detected cognitive bias in a decision
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BiasDetection {
  pub id: Uuid,
  pub bias_type: CognitiveBias,
  pub description: String,
  pub severity: BiasSeverity,
  pub evidence: String,
  pub mitigation: Option<String>,
  pub detected_at: DateTime<Utc>,
}

impl BiasDetection {
  /// Create a new bias detection
  ///
  /// # Errors
  /// Returns error if description or evidence is empty
  pub fn new(
    bias_type: CognitiveBias,
    description: String,
    evidence: String,
  ) -> Result<Self, InversionError> {
    if description.trim().is_empty() {
      return Err(InversionError::EmptyField("description".to_string()));
    }
    if evidence.trim().is_empty() {
      return Err(InversionError::EmptyField("evidence".to_string()));
    }

    Ok(Self {
      id: Uuid::new_v4(),
      bias_type,
      description,
      severity: bias_type.default_severity(),
      evidence,
      mitigation: Some(bias_type.suggested_mitigation().to_string()),
      detected_at: Utc::now(),
    })
  }

  /// Set custom severity
  #[must_use]
  pub const fn with_severity(mut self, severity: BiasSeverity) -> Self {
    self.severity = severity;
    self
  }

  /// Set custom mitigation
  #[must_use]
  pub fn with_mitigation(mut self, mitigation: String) -> Self {
    self.mitigation = Some(mitigation);
    self
  }

  /// Check if this is a critical bias
  #[must_use]
  pub const fn is_critical(&self) -> bool {
    matches!(self.severity, BiasSeverity::Critical | BiasSeverity::High)
  }
}

// ============================================================================
// ANTI-STUPIDITY EXERCISE
// ============================================================================

/// A completed anti-stupidity exercise
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AntiStupidityExercise {
  pub id: Uuid,
  pub method: AntiStupidityMethod,
  pub context: String,
  pub findings: Vec<String>,
  pub actions: Vec<String>,
  pub confidence_score: f32,
  pub created_at: DateTime<Utc>,
}

impl AntiStupidityExercise {
  /// Create a new anti-stupidity exercise
  ///
  /// # Errors
  /// Returns error if context is empty
  pub fn new(method: AntiStupidityMethod, context: String) -> Result<Self, InversionError> {
    if context.trim().is_empty() {
      return Err(InversionError::EmptyField("context".to_string()));
    }

    Ok(Self {
      id: Uuid::new_v4(),
      method,
      context,
      findings: Vec::new(),
      actions: Vec::new(),
      confidence_score: MIN_SCORE,
      created_at: Utc::now(),
    })
  }

  /// Add a finding
  #[must_use]
  pub fn with_finding(mut self, finding: String) -> Self {
    self.findings.push(finding);
    self.confidence_score = self.calculate_confidence();
    self
  }

  /// Add an action
  #[must_use]
  pub fn with_action(mut self, action: String) -> Self {
    self.actions.push(action);
    self
  }

  /// Set confidence score (clamped to valid range)
  #[must_use]
  pub const fn with_confidence(mut self, score: f32) -> Self {
    self.confidence_score = score.clamp(MIN_SCORE, MAX_SCORE);
    self
  }

  /// Calculate confidence based on findings
  fn calculate_confidence(&self) -> f32 {
    let finding_count = self.findings.len() as f32;
    let action_count = self.actions.len() as f32;

    if finding_count == 0.0 {
      return MIN_SCORE;
    }

    let finding_score = (finding_count / 5.0).min(0.5);
    let action_score = (action_count / 3.0).min(0.5);

    (finding_score + action_score).clamp(MIN_SCORE, MAX_SCORE)
  }

  /// Check if exercise is complete (has findings and actions)
  #[must_use]
  pub fn is_complete(&self) -> bool {
    !self.findings.is_empty() && !self.actions.is_empty()
  }
}

// ============================================================================
// INVERSION REVIEW
// ============================================================================

/// Complete decision review with inversion analysis
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InversionReview {
  pub id: Uuid,
  pub decision_context: String,
  pub detected_biases: Vec<BiasDetection>,
  pub exercises: Vec<AntiStupidityExercise>,
  pub stupidity_risk_score: f32,
  pub recommendations: Vec<String>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl InversionReview {
  /// Create a new inversion review
  ///
  /// # Errors
  /// Returns error if decision_context is empty
  pub fn new(decision_context: String) -> Result<Self, InversionError> {
    if decision_context.trim().is_empty() {
      return Err(InversionError::EmptyField("decision_context".to_string()));
    }

    let now = Utc::now();
    Ok(Self {
      id: Uuid::new_v4(),
      decision_context,
      detected_biases: Vec::new(),
      exercises: Vec::new(),
      stupidity_risk_score: MIN_SCORE,
      recommendations: Vec::new(),
      created_at: now,
      updated_at: now,
    })
  }

  /// Add a detected bias
  #[must_use]
  pub fn with_bias_detection(mut self, detection: BiasDetection) -> Self {
    self.detected_biases.push(detection);
    self.recalculate_risk_score();
    self.generate_recommendations();
    self.updated_at = Utc::now();
    self
  }

  /// Add an anti-stupidity exercise
  #[must_use]
  pub fn with_exercise(mut self, exercise: AntiStupidityExercise) -> Self {
    self.exercises.push(exercise);
    self.recalculate_risk_score();
    self.updated_at = Utc::now();
    self
  }

  /// Set stupidity risk score (clamped to valid range)
  #[must_use]
  pub const fn with_risk_score(mut self, score: f32) -> Self {
    self.stupidity_risk_score = score.clamp(MIN_SCORE, MAX_SCORE);
    self
  }

  /// Add a recommendation
  #[must_use]
  pub fn with_recommendation(mut self, recommendation: String) -> Self {
    self.recommendations.push(recommendation);
    self.updated_at = Utc::now();
    self
  }

  /// Recalculate risk score based on biases and exercises
  fn recalculate_risk_score(&mut self) {
    if self.detected_biases.is_empty() {
      self.stupidity_risk_score = MIN_SCORE;
      return;
    }

    let bias_risk: f32 = self
      .detected_biases
      .iter()
      .map(|b| b.severity.weight())
      .sum::<f32>()
      / self.detected_biases.len().max(1) as f32;

    let exercise_mitigation: f32 = self
      .exercises
      .iter()
      .map(|e| e.confidence_score * 0.3)
      .sum::<f32>()
      .min(0.5);

    self.stupidity_risk_score = (bias_risk - exercise_mitigation).clamp(MIN_SCORE, MAX_SCORE);
  }

  /// Generate recommendations from detected biases
  fn generate_recommendations(&mut self) {
    self.recommendations = self
      .detected_biases
      .iter()
      .filter_map(|b| b.mitigation.as_ref())
      .map(String::clone)
      .collect();
  }

  /// Check if decision is likely stupid (high risk score)
  #[must_use]
  pub const fn is_likely_stupid(&self) -> bool {
    self.stupidity_risk_score >= STUPIDITY_THRESHOLD
  }

  /// Get critical biases (high or critical severity)
  #[must_use]
  pub fn critical_biases(&self) -> Vec<&BiasDetection> {
    self
      .detected_biases
      .iter()
      .filter(|b| b.is_critical())
      .collect()
  }

  /// Get incomplete exercises
  #[must_use]
  pub fn incomplete_exercises(&self) -> Vec<&AntiStupidityExercise> {
    self.exercises.iter().filter(|e| !e.is_complete()).collect()
  }

  /// Calculate decision quality score (inverse of stupidity risk)
  #[must_use]
  pub const fn decision_quality_score(&self) -> f32 {
    MAX_SCORE - self.stupidity_risk_score
  }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_cognitive_bias_display() {
    assert_eq!(
      CognitiveBias::ConfirmationBias.to_string(),
      "Confirmation Bias"
    );
    assert_eq!(
      CognitiveBias::SunkCostFallacy.to_string(),
      "Sunk Cost Fallacy"
    );
    assert_eq!(
      CognitiveBias::PlanningFallacy.to_string(),
      "Planning Fallacy"
    );
  }

  #[test]
  fn test_bias_severity_ordering() {
    assert!(BiasSeverity::Low < BiasSeverity::Moderate);
    assert!(BiasSeverity::Moderate < BiasSeverity::High);
    assert!(BiasSeverity::High < BiasSeverity::Critical);
  }

  #[test]
  fn test_anti_stupidity_method_display() {
    assert_eq!(
      AntiStupidityMethod::PreMortem.to_string(),
      "Pre-Mortem Analysis"
    );
    assert_eq!(AntiStupidityMethod::RedTeaming.to_string(), "Red Teaming");
  }

  #[test]
  fn test_bias_detection_new_requires_non_empty_description() {
    let result = BiasDetection::new(
      CognitiveBias::ConfirmationBias,
      "".to_string(),
      "evidence".to_string(),
    );
    assert!(result.is_err());
  }

  #[test]
  fn test_bias_detection_new_requires_non_empty_evidence() {
    let result = BiasDetection::new(
      CognitiveBias::ConfirmationBias,
      "description".to_string(),
      "".to_string(),
    );
    assert!(result.is_err());
  }

  #[test]
  fn test_bias_detection_new_succeeds_with_valid_input() {
    let result = BiasDetection::new(
      CognitiveBias::ConfirmationBias,
      "Only seeking confirming evidence".to_string(),
      "Ignored 3 user interviews that contradicted hypothesis".to_string(),
    );
    assert!(result.is_ok());
    let detection = result.unwrap();
    assert_eq!(detection.bias_type, CognitiveBias::ConfirmationBias);
    assert!(detection.mitigation.is_some());
  }

  #[test]
  fn test_bias_detection_builder_pattern() {
    let detection = BiasDetection::new(
      CognitiveBias::SunkCostFallacy,
      "Continuing despite negative signals".to_string(),
      "We've already invested 6 months".to_string(),
    )
    .unwrap()
    .with_severity(BiasSeverity::Critical)
    .with_mitigation("Evaluate future value only".to_string());

    assert_eq!(detection.severity, BiasSeverity::Critical);
    assert_eq!(
      detection.mitigation,
      Some("Evaluate future value only".to_string())
    );
  }

  #[test]
  fn test_bias_detection_is_critical() {
    let critical = BiasDetection::new(
      CognitiveBias::SunkCostFallacy,
      "test".to_string(),
      "evidence".to_string(),
    )
    .unwrap()
    .with_severity(BiasSeverity::Critical);

    let low = BiasDetection::new(
      CognitiveBias::AttributionError,
      "test".to_string(),
      "evidence".to_string(),
    )
    .unwrap()
    .with_severity(BiasSeverity::Low);

    assert!(critical.is_critical());
    assert!(!low.is_critical());
  }

  #[test]
  fn test_anti_stupidity_exercise_new_requires_context() {
    let result = AntiStupidityExercise::new(AntiStupidityMethod::PreMortem, "".to_string());
    assert!(result.is_err());
  }

  #[test]
  fn test_anti_stupidity_exercise_builder_pattern() {
    let exercise = AntiStupidityExercise::new(
      AntiStupidityMethod::PreMortem,
      "New feature launch decision".to_string(),
    )
    .unwrap()
    .with_finding("Could fail due to poor onboarding".to_string())
    .with_finding("Might overwhelm support team".to_string())
    .with_action("Add onboarding tutorial".to_string())
    .with_action("Brief support team".to_string());

    assert_eq!(exercise.findings.len(), 2);
    assert_eq!(exercise.actions.len(), 2);
    assert!(exercise.is_complete());
  }

  #[test]
  fn test_anti_stupidity_exercise_confidence_clamped() {
    let exercise = AntiStupidityExercise::new(AntiStupidityMethod::RedTeaming, "test".to_string())
      .unwrap()
      .with_confidence(1.5);

    assert!((exercise.confidence_score - 1.0).abs() < f32::EPSILON);

    let exercise = AntiStupidityExercise::new(AntiStupidityMethod::RedTeaming, "test".to_string())
      .unwrap()
      .with_confidence(-0.5);

    assert!((exercise.confidence_score - 0.0).abs() < f32::EPSILON);
  }

  #[test]
  fn test_anti_stupidity_exercise_calculates_confidence() {
    let exercise = AntiStupidityExercise::new(AntiStupidityMethod::PreMortem, "test".to_string())
      .unwrap()
      .with_finding("f1".to_string())
      .with_finding("f2".to_string())
      .with_finding("f3".to_string())
      .with_finding("f4".to_string())
      .with_finding("f5".to_string())
      .with_action("a1".to_string())
      .with_action("a2".to_string())
      .with_action("a3".to_string());

    assert!(exercise.confidence_score > MIN_SCORE);
    assert!(exercise.confidence_score <= MAX_SCORE);
  }

  #[test]
  fn test_inversion_review_new_requires_context() {
    let result = InversionReview::new("".to_string());
    assert!(result.is_err());
  }

  #[test]
  fn test_inversion_review_new_succeeds_with_valid_input() {
    let result = InversionReview::new("Should we pivot to a new market?".to_string());
    assert!(result.is_ok());
    let review = result.unwrap();
    assert_eq!(review.decision_context, "Should we pivot to a new market?");
    assert_eq!(review.stupidity_risk_score, MIN_SCORE);
  }

  #[test]
  fn test_inversion_review_stupidity_risk_clamped() {
    let review = InversionReview::new("test".to_string())
      .unwrap()
      .with_risk_score(1.5);

    assert!((review.stupidity_risk_score - 1.0).abs() < f32::EPSILON);
  }

  #[test]
  fn test_inversion_review_builder_pattern() {
    let detection = BiasDetection::new(
      CognitiveBias::ConfirmationBias,
      "test".to_string(),
      "evidence".to_string(),
    )
    .unwrap();

    let exercise =
      AntiStupidityExercise::new(AntiStupidityMethod::PreMortem, "test".to_string()).unwrap();

    let review = InversionReview::new("Important decision".to_string())
      .unwrap()
      .with_bias_detection(detection)
      .with_exercise(exercise)
      .with_recommendation("Seek disconfirming evidence".to_string());

    assert_eq!(review.detected_biases.len(), 1);
    assert_eq!(review.exercises.len(), 1);
    assert_eq!(review.recommendations.len(), 1);
  }

  #[test]
  fn test_inversion_review_calculates_risk_score() {
    let high_bias = BiasDetection::new(
      CognitiveBias::ConfirmationBias,
      "test".to_string(),
      "evidence".to_string(),
    )
    .unwrap()
    .with_severity(BiasSeverity::High);

    let review = InversionReview::new("test".to_string())
      .unwrap()
      .with_bias_detection(high_bias);

    assert!(review.stupidity_risk_score > MIN_SCORE);
  }

  #[test]
  fn test_inversion_review_exercise_reduces_risk() {
    let bias = BiasDetection::new(
      CognitiveBias::ConfirmationBias,
      "test".to_string(),
      "evidence".to_string(),
    )
    .unwrap()
    .with_severity(BiasSeverity::High);

    let review_without_exercise = InversionReview::new("test".to_string())
      .unwrap()
      .with_bias_detection(bias.clone());

    let exercise = AntiStupidityExercise::new(AntiStupidityMethod::PreMortem, "test".to_string())
      .unwrap()
      .with_finding("f1".to_string())
      .with_finding("f2".to_string())
      .with_finding("f3".to_string())
      .with_finding("f4".to_string())
      .with_finding("f5".to_string())
      .with_action("a1".to_string());

    let review_with_exercise = InversionReview::new("test".to_string())
      .unwrap()
      .with_bias_detection(bias)
      .with_exercise(exercise);

    assert!(
      review_with_exercise.stupidity_risk_score < review_without_exercise.stupidity_risk_score
    );
  }

  #[test]
  fn test_inversion_review_is_stupid_threshold() {
    let critical_bias = BiasDetection::new(
      CognitiveBias::SunkCostFallacy,
      "test".to_string(),
      "evidence".to_string(),
    )
    .unwrap()
    .with_severity(BiasSeverity::Critical);

    let review = InversionReview::new("test".to_string())
      .unwrap()
      .with_bias_detection(critical_bias);

    assert!(review.is_likely_stupid());
  }

  #[test]
  fn test_inversion_review_generates_recommendations() {
    let bias = BiasDetection::new(
      CognitiveBias::ConfirmationBias,
      "test".to_string(),
      "evidence".to_string(),
    )
    .unwrap();

    let review = InversionReview::new("test".to_string())
      .unwrap()
      .with_bias_detection(bias);

    assert!(!review.recommendations.is_empty());
    assert!(review
      .recommendations
      .iter()
      .any(|r| r.contains("disconfirming")));
  }

  #[test]
  fn test_inversion_review_critical_biases() {
    let critical_bias = BiasDetection::new(
      CognitiveBias::SunkCostFallacy,
      "test".to_string(),
      "evidence".to_string(),
    )
    .unwrap()
    .with_severity(BiasSeverity::Critical);

    let low_bias = BiasDetection::new(
      CognitiveBias::AttributionError,
      "test".to_string(),
      "evidence".to_string(),
    )
    .unwrap()
    .with_severity(BiasSeverity::Low);

    let review = InversionReview::new("test".to_string())
      .unwrap()
      .with_bias_detection(critical_bias)
      .with_bias_detection(low_bias);

    let critical = review.critical_biases();
    assert_eq!(critical.len(), 1);
  }

  #[test]
  fn test_inversion_review_decision_quality_score() {
    let review = InversionReview::new("test".to_string())
      .unwrap()
      .with_risk_score(0.3);

    assert!((review.decision_quality_score() - 0.7).abs() < f32::EPSILON);
  }

  #[test]
  fn test_bias_severity_weights() {
    assert!((BiasSeverity::Low.weight() - 0.25).abs() < f32::EPSILON);
    assert!((BiasSeverity::Moderate.weight() - 0.5).abs() < f32::EPSILON);
    assert!((BiasSeverity::High.weight() - 0.75).abs() < f32::EPSILON);
    assert!((BiasSeverity::Critical.weight() - 1.0).abs() < f32::EPSILON);
  }

  #[test]
  fn test_cognitive_bias_default_severity() {
    assert_eq!(
      CognitiveBias::ConfirmationBias.default_severity(),
      BiasSeverity::High
    );
    assert_eq!(
      CognitiveBias::PlanningFallacy.default_severity(),
      BiasSeverity::High
    );
    assert_eq!(
      CognitiveBias::AttributionError.default_severity(),
      BiasSeverity::Low
    );
  }

  #[test]
  fn test_cognitive_bias_suggested_mitigation() {
    let mitigation = CognitiveBias::ConfirmationBias.suggested_mitigation();
    assert!(!mitigation.is_empty());
    assert!(mitigation.contains("disconfirming"));
  }

  #[test]
  fn test_anti_stupidity_method_description() {
    let desc = AntiStupidityMethod::PreMortem.description();
    assert!(!desc.is_empty());
    assert!(desc.contains("fail"));
  }

  #[test]
  fn test_inversion_review_incomplete_exercises() {
    let complete_exercise =
      AntiStupidityExercise::new(AntiStupidityMethod::PreMortem, "test".to_string())
        .unwrap()
        .with_finding("f1".to_string())
        .with_action("a1".to_string());

    let incomplete_exercise =
      AntiStupidityExercise::new(AntiStupidityMethod::RedTeaming, "test".to_string()).unwrap();

    let review = InversionReview::new("test".to_string())
      .unwrap()
      .with_exercise(complete_exercise)
      .with_exercise(incomplete_exercise);

    let incomplete = review.incomplete_exercises();
    assert_eq!(incomplete.len(), 1);
  }
}
