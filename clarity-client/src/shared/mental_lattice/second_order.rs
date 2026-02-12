//! Second-Order Thinking Framework
//!
//! From The Product-Minded Engineer:
//! "Every action has consequences. Those consequences have consequences.
//! Second-order thinking is about tracing these chains to understand
//! the full impact of decisions."
//!
//! # Core Concepts
//!
//! ## Consequence Orders
//!
//! - **First-order**: Direct, immediate effects of an action
//! - **Second-order**: Effects caused by the first-order effects
//! - **Nth-order**: Further downstream cascading effects
//!
//! ## Time Horizons
//!
//! Consequences unfold over different time scales:
//! - **Immediate**: Seconds to minutes
//! - **Short-term**: Hours to days
//! - **Medium-term**: Weeks to months
//! - **Long-term**: Months to years
//!
//! ## Blind Spots
//!
//! Common patterns of missed consequences:
//! - **User adaptation**: Users change behavior in unexpected ways
//! - **System load**: Success creates scaling challenges
//! - **Competitive response**: Market reacts to your moves
//! - **Second-order users**: People affected but not using the product
//!
//! # Design Principles
//!
//! 1. **Explicit Chain Tracing**: Make consequence chains visible
//! 2. **Time-Aware**: Consider when effects materialize
//! 3. **Likelihood Weighting**: Not all consequences are equally likely
//! 4. **Impact Scoring**: Quantify the magnitude of effects
//! 5. **Blind Spot Detection**: Systematically find missed consequences

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

pub const MIN_CHAIN_DEPTH: usize = 2;
pub const MAX_ORDER: u8 = 5;
pub const HIGH_IMPACT_THRESHOLD: f32 = 0.7;
pub const HIGH_LIKELIHOOD_THRESHOLD: f32 = 0.7;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeHorizon {
  Immediate,
  ShortTerm,
  MediumTerm,
  LongTerm,
}

impl fmt::Display for TimeHorizon {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Immediate => write!(f, "Immediate"),
      Self::ShortTerm => write!(f, "Short-term"),
      Self::MediumTerm => write!(f, "Medium-term"),
      Self::LongTerm => write!(f, "Long-term"),
    }
  }
}

impl TimeHorizon {
  #[must_use]
  pub const fn order(&self) -> u8 {
    match self {
      Self::Immediate => 0,
      Self::ShortTerm => 1,
      Self::MediumTerm => 2,
      Self::LongTerm => 3,
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsequenceCategory {
  UserBehavior,
  SystemPerformance,
  BusinessMetric,
  CompetitiveResponse,
  SocialImpact,
  TechnicalDebt,
}

impl fmt::Display for ConsequenceCategory {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UserBehavior => write!(f, "User Behavior"),
      Self::SystemPerformance => write!(f, "System Performance"),
      Self::BusinessMetric => write!(f, "Business Metric"),
      Self::CompetitiveResponse => write!(f, "Competitive Response"),
      Self::SocialImpact => write!(f, "Social Impact"),
      Self::TechnicalDebt => write!(f, "Technical Debt"),
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlindSpotType {
  UserAdaptation,
  SystemLoad,
  CompetitiveResponse,
  SecondOrderUsers,
  EdgeCaseBehavior,
  IncentiveMisalignment,
}

impl fmt::Display for BlindSpotType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UserAdaptation => write!(f, "User Adaptation"),
      Self::SystemLoad => write!(f, "System Load"),
      Self::CompetitiveResponse => write!(f, "Competitive Response"),
      Self::SecondOrderUsers => write!(f, "Second-Order Users"),
      Self::EdgeCaseBehavior => write!(f, "Edge Case Behavior"),
      Self::IncentiveMisalignment => write!(f, "Incentive Misalignment"),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Consequence {
  pub id: Uuid,
  pub order: u8,
  pub description: String,
  pub category: ConsequenceCategory,
  pub time_horizon: TimeHorizon,
  pub impact_score: f32,
  pub likelihood_score: f32,
  pub parent_id: Option<Uuid>,
  pub created_at: DateTime<Utc>,
}

impl Consequence {
  pub fn new(
    order: u8,
    description: String,
    category: ConsequenceCategory,
    time_horizon: TimeHorizon,
  ) -> Result<Self, SecondOrderError> {
    if description.trim().is_empty() {
      return Err(SecondOrderError::EmptyField("description".to_string()));
    }
    if order > MAX_ORDER {
      return Err(SecondOrderError::OrderTooHigh {
        max: MAX_ORDER,
        actual: order,
      });
    }

    Ok(Self {
      id: Uuid::new_v4(),
      order,
      description: description.trim().to_string(),
      category,
      time_horizon,
      impact_score: 0.5,
      likelihood_score: 0.5,
      parent_id: None,
      created_at: Utc::now(),
    })
  }

  #[must_use]
  pub fn with_parent(mut self, parent_id: Uuid) -> Self {
    self.parent_id = Some(parent_id);
    self
  }

  #[must_use]
  pub const fn with_impact(mut self, score: f32) -> Self {
    self.impact_score = score.clamp(0.0, 1.0);
    self
  }

  #[must_use]
  pub const fn with_likelihood(mut self, score: f32) -> Self {
    self.likelihood_score = score.clamp(0.0, 1.0);
    self
  }

  #[must_use]
  pub fn is_high_impact(&self) -> bool {
    self.impact_score >= HIGH_IMPACT_THRESHOLD
  }

  #[must_use]
  pub fn is_high_likelihood(&self) -> bool {
    self.likelihood_score >= HIGH_LIKELIHOOD_THRESHOLD
  }

  #[must_use]
  pub fn expected_value(&self) -> f32 {
    self.impact_score * self.likelihood_score
  }

  #[must_use]
  pub fn is_critical(&self) -> bool {
    self.is_high_impact() && self.is_high_likelihood()
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlindSpot {
  pub id: Uuid,
  pub blind_spot_type: BlindSpotType,
  pub description: String,
  pub question_to_ask: String,
  pub severity: BlindSpotSeverity,
  pub detected_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlindSpotSeverity {
  Minor,
  Moderate,
  Major,
  Critical,
}

impl Default for BlindSpotSeverity {
  fn default() -> Self {
    Self::Moderate
  }
}

impl fmt::Display for BlindSpotSeverity {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Minor => write!(f, "Minor"),
      Self::Moderate => write!(f, "Moderate"),
      Self::Major => write!(f, "Major"),
      Self::Critical => write!(f, "Critical"),
    }
  }
}

impl BlindSpot {
  pub fn new(
    blind_spot_type: BlindSpotType,
    description: String,
    question_to_ask: String,
  ) -> Result<Self, SecondOrderError> {
    if description.trim().is_empty() {
      return Err(SecondOrderError::EmptyField("description".to_string()));
    }
    if question_to_ask.trim().is_empty() {
      return Err(SecondOrderError::EmptyField("question_to_ask".to_string()));
    }

    Ok(Self {
      id: Uuid::new_v4(),
      blind_spot_type,
      description,
      question_to_ask,
      severity: BlindSpotSeverity::default(),
      detected_at: Utc::now(),
    })
  }

  #[must_use]
  pub const fn with_severity(mut self, severity: BlindSpotSeverity) -> Self {
    self.severity = severity;
    self
  }

  #[must_use]
  pub fn is_blocking(&self) -> bool {
    matches!(
      self.severity,
      BlindSpotSeverity::Major | BlindSpotSeverity::Critical
    )
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConsequenceChain {
  pub id: Uuid,
  pub decision: String,
  pub consequences: Vec<Consequence>,
  pub blind_spots: Vec<BlindSpot>,
  pub max_depth_reached: u8,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl ConsequenceChain {
  pub fn new(decision: String) -> Result<Self, SecondOrderError> {
    if decision.trim().is_empty() {
      return Err(SecondOrderError::EmptyField("decision".to_string()));
    }

    let now = Utc::now();
    Ok(Self {
      id: Uuid::new_v4(),
      decision: decision.trim().to_string(),
      consequences: Vec::new(),
      blind_spots: Vec::new(),
      max_depth_reached: 0,
      created_at: now,
      updated_at: now,
    })
  }

  #[must_use]
  pub fn with_consequence(mut self, consequence: Consequence) -> Self {
    if consequence.order > self.max_depth_reached {
      self.max_depth_reached = consequence.order;
    }
    self.consequences.push(consequence);
    self.updated_at = Utc::now();
    self
  }

  #[must_use]
  pub fn with_blind_spot(mut self, blind_spot: BlindSpot) -> Self {
    self.blind_spots.push(blind_spot);
    self.updated_at = Utc::now();
    self
  }

  #[must_use]
  pub fn first_order_consequences(&self) -> Vec<&Consequence> {
    self.consequences.iter().filter(|c| c.order == 1).collect()
  }

  #[must_use]
  pub fn second_order_consequences(&self) -> Vec<&Consequence> {
    self.consequences.iter().filter(|c| c.order == 2).collect()
  }

  #[must_use]
  pub fn nth_order_consequences(&self, n: u8) -> Vec<&Consequence> {
    self.consequences.iter().filter(|c| c.order == n).collect()
  }

  #[must_use]
  pub fn critical_consequences(&self) -> Vec<&Consequence> {
    self
      .consequences
      .iter()
      .filter(|c| c.is_critical())
      .collect()
  }

  #[must_use]
  pub fn chain_for_consequence(&self, consequence_id: Uuid) -> Vec<&Consequence> {
    let mut chain = Vec::new();
    let mut current_id = Some(consequence_id);

    while let Some(id) = current_id {
      if let Some(consequence) = self.consequences.iter().find(|c| c.id == id) {
        chain.push(consequence);
        current_id = consequence.parent_id;
      } else {
        break;
      }
    }

    chain.reverse();
    chain
  }

  pub fn is_complete(&self) -> bool {
    self.max_depth_reached >= MIN_CHAIN_DEPTH as u8
  }

  #[must_use]
  pub fn total_expected_value(&self) -> f32 {
    self.consequences.iter().map(|c| c.expected_value()).sum()
  }

  #[must_use]
  pub fn blocking_blind_spots(&self) -> Vec<&BlindSpot> {
    self
      .blind_spots
      .iter()
      .filter(|b| b.is_blocking())
      .collect()
  }

  pub fn validate(&self) -> Result<(), SecondOrderError> {
    if self.max_depth_reached < MIN_CHAIN_DEPTH as u8 {
      return Err(SecondOrderError::InsufficientDepth {
        required: MIN_CHAIN_DEPTH,
        actual: self.max_depth_reached as usize,
      });
    }
    if !self.blocking_blind_spots().is_empty() {
      return Err(SecondOrderError::BlockingBlindSpots {
        count: self.blocking_blind_spots().len(),
      });
    }
    Ok(())
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SecondOrderAnalysis {
  pub id: Uuid,
  pub decision_context: String,
  pub chains: Vec<ConsequenceChain>,
  pub overall_expected_value: f32,
  pub highest_impact_consequence: Option<Uuid>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl SecondOrderAnalysis {
  pub fn new(decision_context: String) -> Result<Self, SecondOrderError> {
    if decision_context.trim().is_empty() {
      return Err(SecondOrderError::EmptyField("decision_context".to_string()));
    }

    let now = Utc::now();
    Ok(Self {
      id: Uuid::new_v4(),
      decision_context,
      chains: Vec::new(),
      overall_expected_value: 0.0,
      highest_impact_consequence: None,
      created_at: now,
      updated_at: now,
    })
  }

  #[must_use]
  pub fn with_chain(mut self, chain: ConsequenceChain) -> Self {
    self.chains.push(chain);
    self.recalculate_metrics();
    self.updated_at = Utc::now();
    self
  }

  fn recalculate_metrics(&mut self) {
    self.overall_expected_value = self.chains.iter().map(|c| c.total_expected_value()).sum();

    self.highest_impact_consequence = self
      .chains
      .iter()
      .flat_map(|c| c.consequences.iter())
      .max_by(|a, b| {
        a.expected_value()
          .partial_cmp(&b.expected_value())
          .unwrap_or(std::cmp::Ordering::Equal)
      })
      .map(|c| c.id);
  }

  #[must_use]
  pub fn all_critical_consequences(&self) -> Vec<&Consequence> {
    self
      .chains
      .iter()
      .flat_map(|c| c.critical_consequences())
      .collect()
  }

  #[must_use]
  pub fn all_blind_spots(&self) -> Vec<&BlindSpot> {
    self
      .chains
      .iter()
      .flat_map(|c| c.blind_spots.iter())
      .collect()
  }

  #[must_use]
  pub fn is_ready_for_decision(&self) -> bool {
    self.chains.iter().all(|c| c.is_complete())
      && self.all_blind_spots().iter().all(|b| !b.is_blocking())
  }
}

#[derive(Debug, Error, PartialEq)]
pub enum SecondOrderError {
  #[error("required field is empty: {0}")]
  EmptyField(String),

  #[error("consequence order {actual} exceeds maximum {max}")]
  OrderTooHigh { max: u8, actual: u8 },

  #[error("insufficient chain depth: need {required} but have {actual}")]
  InsufficientDepth { required: usize, actual: usize },

  #[error("blocking blind spots detected: {count} unresolved")]
  BlockingBlindSpots { count: usize },
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn consequence_new_requires_description() {
    let result = Consequence::new(
      1,
      "".to_string(),
      ConsequenceCategory::UserBehavior,
      TimeHorizon::ShortTerm,
    );
    assert!(result.is_err());
  }

  #[test]
  fn consequence_new_rejects_order_too_high() {
    let result = Consequence::new(
      10,
      "test".to_string(),
      ConsequenceCategory::UserBehavior,
      TimeHorizon::ShortTerm,
    );
    assert!(result.is_err());
  }

  #[test]
  fn consequence_new_succeeds_with_valid_input() {
    let result = Consequence::new(
      1,
      "Users will click more".to_string(),
      ConsequenceCategory::UserBehavior,
      TimeHorizon::ShortTerm,
    );
    assert!(result.is_ok());
    let c = result.unwrap();
    assert_eq!(c.order, 1);
    assert_eq!(c.description, "Users will click more");
  }

  #[test]
  fn consequence_expected_value_calculates_correctly() {
    let c = Consequence::new(
      1,
      "test".to_string(),
      ConsequenceCategory::UserBehavior,
      TimeHorizon::ShortTerm,
    )
    .unwrap()
    .with_impact(0.8)
    .with_likelihood(0.5);

    assert!((c.expected_value() - 0.4).abs() < f32::EPSILON);
  }

  #[test]
  fn consequence_is_critical_requires_high_impact_and_likelihood() {
    let not_critical = Consequence::new(
      1,
      "test".to_string(),
      ConsequenceCategory::UserBehavior,
      TimeHorizon::ShortTerm,
    )
    .unwrap()
    .with_impact(0.9)
    .with_likelihood(0.5);

    assert!(!not_critical.is_critical());

    let critical = Consequence::new(
      1,
      "test".to_string(),
      ConsequenceCategory::UserBehavior,
      TimeHorizon::ShortTerm,
    )
    .unwrap()
    .with_impact(0.9)
    .with_likelihood(0.8);

    assert!(critical.is_critical());
  }

  #[test]
  fn blind_spot_new_requires_description() {
    let result = BlindSpot::new(
      BlindSpotType::UserAdaptation,
      "".to_string(),
      "question?".to_string(),
    );
    assert!(result.is_err());
  }

  #[test]
  fn blind_spot_new_requires_question() {
    let result = BlindSpot::new(
      BlindSpotType::UserAdaptation,
      "desc".to_string(),
      "".to_string(),
    );
    assert!(result.is_err());
  }

  #[test]
  fn blind_spot_is_blocking_for_major_and_critical() {
    let minor = BlindSpot::new(
      BlindSpotType::UserAdaptation,
      "desc".to_string(),
      "q?".to_string(),
    )
    .unwrap()
    .with_severity(BlindSpotSeverity::Minor);
    assert!(!minor.is_blocking());

    let critical = BlindSpot::new(
      BlindSpotType::UserAdaptation,
      "desc".to_string(),
      "q?".to_string(),
    )
    .unwrap()
    .with_severity(BlindSpotSeverity::Critical);
    assert!(critical.is_blocking());
  }

  #[test]
  fn consequence_chain_new_requires_decision() {
    let result = ConsequenceChain::new("".to_string());
    assert!(result.is_err());
  }

  #[test]
  fn consequence_chain_tracks_max_depth() {
    let chain = ConsequenceChain::new("Add feature X".to_string()).unwrap();

    let c1 = Consequence::new(
      1,
      "first".to_string(),
      ConsequenceCategory::UserBehavior,
      TimeHorizon::ShortTerm,
    )
    .unwrap();
    let c2 = Consequence::new(
      2,
      "second".to_string(),
      ConsequenceCategory::UserBehavior,
      TimeHorizon::MediumTerm,
    )
    .unwrap();
    let c3 = Consequence::new(
      3,
      "third".to_string(),
      ConsequenceCategory::BusinessMetric,
      TimeHorizon::LongTerm,
    )
    .unwrap();

    let chain = chain
      .with_consequence(c1)
      .with_consequence(c2)
      .with_consequence(c3);

    assert_eq!(chain.max_depth_reached, 3);
  }

  #[test]
  fn consequence_chain_filters_by_order() {
    let c1 = Consequence::new(
      1,
      "first".to_string(),
      ConsequenceCategory::UserBehavior,
      TimeHorizon::ShortTerm,
    )
    .unwrap();
    let c2 = Consequence::new(
      2,
      "second".to_string(),
      ConsequenceCategory::UserBehavior,
      TimeHorizon::MediumTerm,
    )
    .unwrap();
    let c3 = Consequence::new(
      1,
      "another first".to_string(),
      ConsequenceCategory::BusinessMetric,
      TimeHorizon::ShortTerm,
    )
    .unwrap();

    let chain = ConsequenceChain::new("test".to_string())
      .unwrap()
      .with_consequence(c1)
      .with_consequence(c2)
      .with_consequence(c3);

    assert_eq!(chain.first_order_consequences().len(), 2);
    assert_eq!(chain.second_order_consequences().len(), 1);
  }

  #[test]
  fn consequence_chain_is_complete_requires_min_depth() {
    let shallow = ConsequenceChain::new("test".to_string())
      .unwrap()
      .with_consequence(
        Consequence::new(
          1,
          "c1".to_string(),
          ConsequenceCategory::UserBehavior,
          TimeHorizon::ShortTerm,
        )
        .unwrap(),
      );

    assert!(!shallow.is_complete());

    let deep = ConsequenceChain::new("test".to_string())
      .unwrap()
      .with_consequence(
        Consequence::new(
          1,
          "c1".to_string(),
          ConsequenceCategory::UserBehavior,
          TimeHorizon::ShortTerm,
        )
        .unwrap(),
      )
      .with_consequence(
        Consequence::new(
          2,
          "c2".to_string(),
          ConsequenceCategory::UserBehavior,
          TimeHorizon::MediumTerm,
        )
        .unwrap(),
      );

    assert!(deep.is_complete());
  }

  #[test]
  fn consequence_chain_validate_fails_for_insufficient_depth() {
    let chain = ConsequenceChain::new("test".to_string())
      .unwrap()
      .with_consequence(
        Consequence::new(
          1,
          "c1".to_string(),
          ConsequenceCategory::UserBehavior,
          TimeHorizon::ShortTerm,
        )
        .unwrap(),
      );

    assert!(chain.validate().is_err());
  }

  #[test]
  fn consequence_chain_validate_fails_for_blocking_blind_spots() {
    let chain = ConsequenceChain::new("test".to_string())
      .unwrap()
      .with_consequence(
        Consequence::new(
          1,
          "c1".to_string(),
          ConsequenceCategory::UserBehavior,
          TimeHorizon::ShortTerm,
        )
        .unwrap(),
      )
      .with_consequence(
        Consequence::new(
          2,
          "c2".to_string(),
          ConsequenceCategory::UserBehavior,
          TimeHorizon::MediumTerm,
        )
        .unwrap(),
      )
      .with_blind_spot(
        BlindSpot::new(
          BlindSpotType::UserAdaptation,
          "desc".to_string(),
          "q?".to_string(),
        )
        .unwrap()
        .with_severity(BlindSpotSeverity::Critical),
      );

    assert!(chain.validate().is_err());
  }

  #[test]
  fn consequence_chain_traces_parent_chain() {
    let parent = Consequence::new(
      1,
      "parent".to_string(),
      ConsequenceCategory::UserBehavior,
      TimeHorizon::ShortTerm,
    )
    .unwrap();
    let parent_id = parent.id;

    let child = Consequence::new(
      2,
      "child".to_string(),
      ConsequenceCategory::UserBehavior,
      TimeHorizon::MediumTerm,
    )
    .unwrap()
    .with_parent(parent_id);
    let child_id = child.id;

    let grandchild = Consequence::new(
      3,
      "grandchild".to_string(),
      ConsequenceCategory::BusinessMetric,
      TimeHorizon::LongTerm,
    )
    .unwrap()
    .with_parent(child_id);
    let grandchild_id = grandchild.id;

    let chain = ConsequenceChain::new("test".to_string())
      .unwrap()
      .with_consequence(parent)
      .with_consequence(child)
      .with_consequence(grandchild);

    let traced = chain.chain_for_consequence(grandchild_id);
    assert_eq!(traced.len(), 3);
    assert_eq!(traced[0].order, 1);
    assert_eq!(traced[1].order, 2);
    assert_eq!(traced[2].order, 3);
  }

  #[test]
  fn second_order_analysis_new_requires_context() {
    let result = SecondOrderAnalysis::new("".to_string());
    assert!(result.is_err());
  }

  #[test]
  fn second_order_analysis_calculates_total_expected_value() {
    let chain1 = ConsequenceChain::new("test".to_string())
      .unwrap()
      .with_consequence(
        Consequence::new(
          1,
          "c1".to_string(),
          ConsequenceCategory::UserBehavior,
          TimeHorizon::ShortTerm,
        )
        .unwrap()
        .with_impact(0.5)
        .with_likelihood(1.0),
      )
      .with_consequence(
        Consequence::new(
          2,
          "c2".to_string(),
          ConsequenceCategory::UserBehavior,
          TimeHorizon::MediumTerm,
        )
        .unwrap()
        .with_impact(0.5)
        .with_likelihood(1.0),
      );

    let analysis = SecondOrderAnalysis::new("test".to_string())
      .unwrap()
      .with_chain(chain1);

    assert!((analysis.overall_expected_value - 1.0).abs() < f32::EPSILON);
  }

  #[test]
  fn second_order_analysis_is_ready_for_decision() {
    let incomplete_chain = ConsequenceChain::new("test".to_string())
      .unwrap()
      .with_consequence(
        Consequence::new(
          1,
          "c1".to_string(),
          ConsequenceCategory::UserBehavior,
          TimeHorizon::ShortTerm,
        )
        .unwrap(),
      );

    let analysis_incomplete = SecondOrderAnalysis::new("test".to_string())
      .unwrap()
      .with_chain(incomplete_chain);

    assert!(!analysis_incomplete.is_ready_for_decision());

    let complete_chain = ConsequenceChain::new("test".to_string())
      .unwrap()
      .with_consequence(
        Consequence::new(
          1,
          "c1".to_string(),
          ConsequenceCategory::UserBehavior,
          TimeHorizon::ShortTerm,
        )
        .unwrap(),
      )
      .with_consequence(
        Consequence::new(
          2,
          "c2".to_string(),
          ConsequenceCategory::UserBehavior,
          TimeHorizon::MediumTerm,
        )
        .unwrap(),
      );

    let analysis_complete = SecondOrderAnalysis::new("test".to_string())
      .unwrap()
      .with_chain(complete_chain);

    assert!(analysis_complete.is_ready_for_decision());
  }

  #[test]
  fn time_horizon_order() {
    assert!(TimeHorizon::Immediate.order() < TimeHorizon::ShortTerm.order());
    assert!(TimeHorizon::ShortTerm.order() < TimeHorizon::MediumTerm.order());
    assert!(TimeHorizon::MediumTerm.order() < TimeHorizon::LongTerm.order());
  }
}
