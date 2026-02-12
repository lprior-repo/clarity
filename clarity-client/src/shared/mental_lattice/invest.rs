//! INVEST Behavior Specification Framework
//!
//! From The Product-Minded Engineer:
//! INVEST is a mnemonic for evaluating user stories and behaviors.
//! Well-defined behaviors are Independent, Negotiable, Valuable,
//! Estimable, Small, and Testable.
//!
//! # Core Concepts
//!
//! ## INVEST Criteria
//!
//! - **Independent**: Can be delivered without dependencies on other stories
//! - **Negotiable**: Details can be refined through collaboration
//! - **Valuable**: Provides clear value to users or stakeholders
//! - **Estimable**: Effort can be reasonably estimated
//! - **Small**: Fits within a single iteration/sprint
//! - **Testable**: Has clear acceptance criteria
//!
//! ## Behavior Scores
//!
//! Each criterion is scored 0.0-1.0:
//! - 0.0-0.3: Poor (violates criterion)
//! - 0.4-0.6: Acceptable (partially meets)
//! - 0.7-1.0: Good (fully meets)
//!
//! # Design Principles
//!
//! 1. **Quantifiable Quality**: Each criterion has a numeric score
//! 2. **Blocking Detection**: Low scores block implementation
//! 3. **Improvement Suggestions**: Specific advice for weak areas
//! 4. **Aggregate Quality**: Overall INVEST score for prioritization

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

pub const MIN_SCORE: f32 = 0.0;
pub const MAX_SCORE: f32 = 1.0;
pub const GOOD_THRESHOLD: f32 = 0.7;
pub const ACCEPTABLE_THRESHOLD: f32 = 0.4;
pub const BLOCKING_THRESHOLD: f32 = 0.3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InvestCriterion {
  Independent,
  Negotiable,
  Valuable,
  Estimable,
  Small,
  Testable,
}

impl fmt::Display for InvestCriterion {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Independent => write!(f, "Independent"),
      Self::Negotiable => write!(f, "Negotiable"),
      Self::Valuable => write!(f, "Valuable"),
      Self::Estimable => write!(f, "Estimable"),
      Self::Small => write!(f, "Small"),
      Self::Testable => write!(f, "Testable"),
    }
  }
}

impl InvestCriterion {
  #[must_use]
  pub const fn description(&self) -> &'static str {
    match self {
      Self::Independent => "Can be delivered without dependencies on other stories",
      Self::Negotiable => "Details can be refined through collaboration",
      Self::Valuable => "Provides clear value to users or stakeholders",
      Self::Estimable => "Effort can be reasonably estimated",
      Self::Small => "Fits within a single iteration or sprint",
      Self::Testable => "Has clear acceptance criteria",
    }
  }

  #[must_use]
  pub const fn improvement_hint(&self) -> &'static str {
    match self {
      Self::Independent => "Break down dependencies or reorder priorities",
      Self::Negotiable => "Leave room for discussion; avoid over-specification",
      Self::Valuable => "Connect to user outcomes or business metrics",
      Self::Estimable => "Clarify scope or split into smaller pieces",
      Self::Small => "Split into multiple smaller stories",
      Self::Testable => "Add specific acceptance criteria or examples",
    }
  }

  #[must_use]
  pub fn all() -> &'static [Self] {
    &[
      Self::Independent,
      Self::Negotiable,
      Self::Valuable,
      Self::Estimable,
      Self::Small,
      Self::Testable,
    ]
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreLevel {
  Poor,
  Acceptable,
  Good,
}

impl fmt::Display for ScoreLevel {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Poor => write!(f, "Poor"),
      Self::Acceptable => write!(f, "Acceptable"),
      Self::Good => write!(f, "Good"),
    }
  }
}

impl ScoreLevel {
  #[must_use]
  pub fn from_score(score: f32) -> Self {
    if score >= GOOD_THRESHOLD {
      Self::Good
    } else if score >= ACCEPTABLE_THRESHOLD {
      Self::Acceptable
    } else {
      Self::Poor
    }
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CriterionScore {
  pub criterion: InvestCriterion,
  pub score: f32,
  pub justification: String,
  pub improvement_suggestion: Option<String>,
}

impl CriterionScore {
  pub fn new(
    criterion: InvestCriterion,
    score: f32,
    justification: String,
  ) -> Result<Self, InvestError> {
    if justification.trim().is_empty() {
      return Err(InvestError::EmptyField("justification".to_string()));
    }

    Ok(Self {
      criterion,
      score: score.clamp(MIN_SCORE, MAX_SCORE),
      justification: justification.trim().to_string(),
      improvement_suggestion: None,
    })
  }

  #[must_use]
  pub fn with_improvement(mut self, suggestion: String) -> Self {
    self.improvement_suggestion = Some(suggestion);
    self
  }

  #[must_use]
  pub fn level(&self) -> ScoreLevel {
    ScoreLevel::from_score(self.score)
  }

  #[must_use]
  pub fn is_blocking(&self) -> bool {
    self.score < BLOCKING_THRESHOLD
  }

  #[must_use]
  pub fn needs_improvement(&self) -> bool {
    self.score < GOOD_THRESHOLD
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BehaviorSpec {
  pub id: Uuid,
  pub title: String,
  pub description: String,
  pub criterion_scores: Vec<CriterionScore>,
  pub overall_score: f32,
  pub is_ready: bool,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl BehaviorSpec {
  pub fn new(title: String, description: String) -> Result<Self, InvestError> {
    if title.trim().is_empty() {
      return Err(InvestError::EmptyField("title".to_string()));
    }
    if description.trim().is_empty() {
      return Err(InvestError::EmptyField("description".to_string()));
    }

    let now = Utc::now();
    Ok(Self {
      id: Uuid::new_v4(),
      title: title.trim().to_string(),
      description: description.trim().to_string(),
      criterion_scores: Vec::new(),
      overall_score: 0.0,
      is_ready: false,
      created_at: now,
      updated_at: now,
    })
  }

  #[must_use]
  pub fn with_criterion_score(mut self, score: CriterionScore) -> Self {
    self
      .criterion_scores
      .retain(|s| s.criterion != score.criterion);
    self.criterion_scores.push(score);
    self.recalculate_scores();
    self.updated_at = Utc::now();
    self
  }

  fn recalculate_scores(&mut self) {
    if self.criterion_scores.is_empty() {
      self.overall_score = 0.0;
      self.is_ready = false;
      return;
    }

    let sum: f32 = self.criterion_scores.iter().map(|s| s.score).sum();
    self.overall_score = sum / self.criterion_scores.len() as f32;

    self.is_ready = self.criterion_scores.len() == InvestCriterion::all().len()
      && self.criterion_scores.iter().all(|s| !s.is_blocking());
  }

  #[must_use]
  pub fn score_for(&self, criterion: InvestCriterion) -> Option<f32> {
    self
      .criterion_scores
      .iter()
      .find(|s| s.criterion == criterion)
      .map(|s| s.score)
  }

  #[must_use]
  pub fn blocking_criteria(&self) -> Vec<&CriterionScore> {
    self
      .criterion_scores
      .iter()
      .filter(|s| s.is_blocking())
      .collect()
  }

  #[must_use]
  pub fn needs_improvement_criteria(&self) -> Vec<&CriterionScore> {
    self
      .criterion_scores
      .iter()
      .filter(|s| s.needs_improvement())
      .collect()
  }

  #[must_use]
  pub fn missing_criteria(&self) -> Vec<InvestCriterion> {
    InvestCriterion::all()
      .iter()
      .filter(|c| !self.criterion_scores.iter().any(|s| s.criterion == **c))
      .copied()
      .collect()
  }

  #[must_use]
  pub fn is_complete(&self) -> bool {
    self.criterion_scores.len() == InvestCriterion::all().len()
  }

  pub fn validate(&self) -> Result<(), InvestError> {
    let missing = self.missing_criteria();
    if !missing.is_empty() {
      return Err(InvestError::MissingCriteria {
        criteria: missing.iter().map(ToString::to_string).collect(),
      });
    }

    let blocking = self.blocking_criteria();
    if !blocking.is_empty() {
      return Err(InvestError::BlockingScores {
        criteria: blocking.iter().map(|s| s.criterion.to_string()).collect(),
      });
    }

    Ok(())
  }

  #[must_use]
  pub fn quality_level(&self) -> ScoreLevel {
    ScoreLevel::from_score(self.overall_score)
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvestReview {
  pub id: Uuid,
  pub behavior_specs: Vec<BehaviorSpec>,
  pub average_score: f32,
  pub ready_count: usize,
  pub blocked_count: usize,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl InvestReview {
  pub fn new() -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      behavior_specs: Vec::new(),
      average_score: 0.0,
      ready_count: 0,
      blocked_count: 0,
      created_at: now,
      updated_at: now,
    }
  }

  #[must_use]
  pub fn with_behavior_spec(mut self, spec: BehaviorSpec) -> Self {
    self.behavior_specs.push(spec);
    self.recalculate_metrics();
    self.updated_at = Utc::now();
    self
  }

  fn recalculate_metrics(&mut self) {
    if self.behavior_specs.is_empty() {
      self.average_score = 0.0;
      self.ready_count = 0;
      self.blocked_count = 0;
      return;
    }

    let sum: f32 = self.behavior_specs.iter().map(|s| s.overall_score).sum();
    self.average_score = sum / self.behavior_specs.len() as f32;

    self.ready_count = self.behavior_specs.iter().filter(|s| s.is_ready).count();
    self.blocked_count = self
      .behavior_specs
      .iter()
      .filter(|s| !s.blocking_criteria().is_empty())
      .count();
  }

  #[must_use]
  pub fn ready_behaviors(&self) -> Vec<&BehaviorSpec> {
    self.behavior_specs.iter().filter(|s| s.is_ready).collect()
  }

  #[must_use]
  pub fn blocked_behaviors(&self) -> Vec<&BehaviorSpec> {
    self
      .behavior_specs
      .iter()
      .filter(|s| !s.blocking_criteria().is_empty())
      .collect()
  }

  #[must_use]
  pub fn highest_priority(&self) -> Option<&BehaviorSpec> {
    self.behavior_specs.iter().max_by(|a, b| {
      a.overall_score
        .partial_cmp(&b.overall_score)
        .unwrap_or(std::cmp::Ordering::Equal)
    })
  }

  #[must_use]
  pub fn is_healthy(&self) -> bool {
    self.blocked_count == 0 && self.ready_count > 0
  }
}

impl Default for InvestReview {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Error, PartialEq)]
pub enum InvestError {
  #[error("required field is empty: {0}")]
  EmptyField(String),

  #[error("missing INVEST criteria: {criteria:?}")]
  MissingCriteria { criteria: Vec<String> },

  #[error("blocking scores in criteria: {criteria:?}")]
  BlockingScores { criteria: Vec<String> },
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn invest_criterion_descriptions_not_empty() {
    for criterion in InvestCriterion::all() {
      assert!(!criterion.description().is_empty());
      assert!(!criterion.improvement_hint().is_empty());
    }
  }

  #[test]
  fn score_level_from_score() {
    assert_eq!(ScoreLevel::from_score(0.8), ScoreLevel::Good);
    assert_eq!(ScoreLevel::from_score(0.7), ScoreLevel::Good);
    assert_eq!(ScoreLevel::from_score(0.6), ScoreLevel::Acceptable);
    assert_eq!(ScoreLevel::from_score(0.4), ScoreLevel::Acceptable);
    assert_eq!(ScoreLevel::from_score(0.3), ScoreLevel::Poor);
    assert_eq!(ScoreLevel::from_score(0.0), ScoreLevel::Poor);
  }

  #[test]
  fn criterion_score_new_requires_justification() {
    let result = CriterionScore::new(InvestCriterion::Independent, 0.8, "".to_string());
    assert!(result.is_err());
  }

  #[test]
  fn criterion_score_clamps_score() {
    let high = CriterionScore::new(InvestCriterion::Independent, 1.5, "test".to_string()).unwrap();
    assert!((high.score - 1.0).abs() < f32::EPSILON);

    let low = CriterionScore::new(InvestCriterion::Independent, -0.5, "test".to_string()).unwrap();
    assert!((low.score - 0.0).abs() < f32::EPSILON);
  }

  #[test]
  fn criterion_score_is_blocking_below_threshold() {
    let blocking =
      CriterionScore::new(InvestCriterion::Independent, 0.2, "test".to_string()).unwrap();
    assert!(blocking.is_blocking());

    let not_blocking =
      CriterionScore::new(InvestCriterion::Independent, 0.4, "test".to_string()).unwrap();
    assert!(!not_blocking.is_blocking());
  }

  #[test]
  fn behavior_spec_new_requires_title() {
    let result = BehaviorSpec::new("".to_string(), "description".to_string());
    assert!(result.is_err());
  }

  #[test]
  fn behavior_spec_new_requires_description() {
    let result = BehaviorSpec::new("title".to_string(), "".to_string());
    assert!(result.is_err());
  }

  #[test]
  fn behavior_spec_calculates_overall_score() {
    let spec = BehaviorSpec::new("Title".to_string(), "Description".to_string())
      .unwrap()
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Independent, 0.8, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Valuable, 0.6, "test".to_string()).unwrap(),
      );

    assert!((spec.overall_score - 0.7).abs() < f32::EPSILON);
  }

  #[test]
  fn behavior_spec_updates_score_on_replace() {
    let spec = BehaviorSpec::new("Title".to_string(), "Description".to_string())
      .unwrap()
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Independent, 0.8, "test1".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Independent, 0.4, "test2".to_string()).unwrap(),
      );

    assert_eq!(spec.criterion_scores.len(), 1);
    assert!((spec.overall_score - 0.4).abs() < f32::EPSILON);
  }

  #[test]
  fn behavior_spec_is_ready_only_when_complete_and_no_blocking() {
    let incomplete = BehaviorSpec::new("Title".to_string(), "Description".to_string())
      .unwrap()
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Independent, 0.8, "test".to_string()).unwrap(),
      );

    assert!(!incomplete.is_ready);

    let complete_but_blocked = BehaviorSpec::new("Title".to_string(), "Description".to_string())
      .unwrap()
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Independent, 0.8, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Negotiable, 0.8, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Valuable, 0.8, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Estimable, 0.8, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Small, 0.2, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Testable, 0.8, "test".to_string()).unwrap(),
      );

    assert!(!complete_but_blocked.is_ready);

    let ready = BehaviorSpec::new("Title".to_string(), "Description".to_string())
      .unwrap()
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Independent, 0.8, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Negotiable, 0.8, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Valuable, 0.8, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Estimable, 0.8, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Small, 0.8, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Testable, 0.8, "test".to_string()).unwrap(),
      );

    assert!(ready.is_ready);
  }

  #[test]
  fn behavior_spec_missing_criteria() {
    let spec = BehaviorSpec::new("Title".to_string(), "Description".to_string())
      .unwrap()
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Independent, 0.8, "test".to_string()).unwrap(),
      );

    let missing = spec.missing_criteria();
    assert_eq!(missing.len(), 5);
    assert!(!missing.contains(&InvestCriterion::Independent));
  }

  #[test]
  fn behavior_spec_validate_fails_for_missing_criteria() {
    let spec = BehaviorSpec::new("Title".to_string(), "Description".to_string())
      .unwrap()
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Independent, 0.8, "test".to_string()).unwrap(),
      );

    assert!(spec.validate().is_err());
  }

  #[test]
  fn behavior_spec_validate_fails_for_blocking_scores() {
    let spec = BehaviorSpec::new("Title".to_string(), "Description".to_string())
      .unwrap()
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Independent, 0.8, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Negotiable, 0.8, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Valuable, 0.8, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Estimable, 0.8, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Small, 0.2, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Testable, 0.8, "test".to_string()).unwrap(),
      );

    assert!(spec.validate().is_err());
  }

  #[test]
  fn invest_review_calculates_metrics() {
    let ready = BehaviorSpec::new("Ready".to_string(), "Desc".to_string())
      .unwrap()
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Independent, 0.8, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Negotiable, 0.8, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Valuable, 0.8, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Estimable, 0.8, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Small, 0.8, "test".to_string()).unwrap(),
      )
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Testable, 0.8, "test".to_string()).unwrap(),
      );

    let blocked = BehaviorSpec::new("Blocked".to_string(), "Desc".to_string())
      .unwrap()
      .with_criterion_score(
        CriterionScore::new(InvestCriterion::Independent, 0.2, "test".to_string()).unwrap(),
      );

    let review = InvestReview::new()
      .with_behavior_spec(ready)
      .with_behavior_spec(blocked);

    assert_eq!(review.ready_count, 1);
    assert_eq!(review.blocked_count, 1);
  }

  #[test]
  fn invest_review_is_healthy_only_when_no_blocked_and_has_ready() {
    let empty = InvestReview::new();
    assert!(!empty.is_healthy());

    let with_ready = InvestReview::new().with_behavior_spec(
      BehaviorSpec::new("Ready".to_string(), "Desc".to_string())
        .unwrap()
        .with_criterion_score(
          CriterionScore::new(InvestCriterion::Independent, 0.8, "test".to_string()).unwrap(),
        )
        .with_criterion_score(
          CriterionScore::new(InvestCriterion::Negotiable, 0.8, "test".to_string()).unwrap(),
        )
        .with_criterion_score(
          CriterionScore::new(InvestCriterion::Valuable, 0.8, "test".to_string()).unwrap(),
        )
        .with_criterion_score(
          CriterionScore::new(InvestCriterion::Estimable, 0.8, "test".to_string()).unwrap(),
        )
        .with_criterion_score(
          CriterionScore::new(InvestCriterion::Small, 0.8, "test".to_string()).unwrap(),
        )
        .with_criterion_score(
          CriterionScore::new(InvestCriterion::Testable, 0.8, "test".to_string()).unwrap(),
        ),
    );

    assert!(with_ready.is_healthy());
  }
}
