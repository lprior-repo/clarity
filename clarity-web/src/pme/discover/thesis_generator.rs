#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Thesis & Antithesis Generator
//!
//! Generates product theses with required null hypotheses (antitheses). The antithesis
//! represents the "WHY might this fail?" question, preventing optimism bias through
//! mandatory counter-argument generation.
//!
//! # Purpose
//!
//! - Challenge product assumptions systematically
//! - Force consideration of failure modes
//! - Enable validation planning through falsifiable hypotheses
//!
//! # Example
//!
//! ```
//! use clarity_web::pme::discover::thesis_generator::{ThesisAntithesisGenerator, Thesis};
//!
//! let thesis = Thesis::new(
//!     "Users will pay for automated report generation".to_string(),
//!     "Saves 10 hours/week for enterprise analysts".to_string(),
//! );
//!
//! let output = ThesisAntithesisGenerator::generate(thesis);
//! ```

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ============================================================================
// Thesis Types
// ============================================================================

/// A product thesis representing a core belief about value creation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Thesis {
  /// The core hypothesis statement
  pub statement: String,
  /// Supporting rationale/evidence
  pub rationale: String,
  /// Key assumptions underlying this thesis
  pub assumptions: Vec<String>,
  /// Confidence level (0.0-1.0)
  pub confidence: f64,
}

impl Thesis {
  /// Create a new thesis.
  #[must_use]
  pub fn new(statement: String, rationale: String) -> Self {
    Self {
      statement,
      rationale,
      assumptions: Vec::new(),
      confidence: 0.5,
    }
  }

  /// Add an assumption.
  #[must_use]
  pub fn with_assumption(mut self, assumption: String) -> Self {
    self.assumptions.push(assumption);
    self
  }

  /// Set confidence level.
  #[must_use]
  pub fn with_confidence(mut self, confidence: f64) -> Self {
    self.confidence = confidence.clamp(0.0, 1.0);
    self
  }

  /// Check if thesis is well-formed.
  #[must_use]
  pub fn is_valid(&self) -> bool {
    !self.statement.is_empty() && !self.rationale.is_empty()
  }

  /// Extract key terms for analysis.
  #[must_use]
  pub fn key_terms(&self) -> Vec<String> {
    let stop_words = [
      "the", "a", "an", "is", "are", "will", "be", "to", "for", "and", "or",
    ];

    self
      .statement
      .split_whitespace()
      .filter(|word| {
        let lower = word.to_lowercase();
        lower.len() > 3 && !stop_words.contains(&lower.as_str())
      })
      .map(|s| s.to_string())
      .unique()
      .collect()
  }
}

// ============================================================================
// Antithesis Types
// ============================================================================

/// An antithesis representing a potential failure mode of the thesis.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Antithesis {
  /// The counter-hypothesis statement
  pub statement: String,
  /// Why this failure mode might occur
  pub reasoning: String,
  /// Category of failure
  pub failure_category: FailureCategory,
  /// Probability estimate (0.0-1.0)
  pub probability: f64,
  /// Evidence that supports this antithesis
  pub supporting_evidence: Vec<String>,
  /// How to validate/falsify this antithesis
  pub validation_approach: Option<String>,
}

/// Categories of potential failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FailureCategory {
  /// Users don't have the problem (problem risk)
  ProblemDoesntExist,
  /// Users won't adopt the solution (solution risk)
  NoAdoption,
  /// Business model doesn't work (business risk)
  BusinessModelFailure,
  /// Technical feasibility issues (technical risk)
  TechnicalInfeasibility,
  /// Market timing is wrong (timing risk)
  WrongTiming,
  /// Competition is too strong (competitive risk)
  CompetitiveThreat,
  /// Resource constraints (resource risk)
  ResourceConstraints,
}

impl FailureCategory {
  /// Get a description of this failure category.
  #[must_use]
  pub const fn description(&self) -> &'static str {
    match self {
      Self::ProblemDoesntExist => {
        "The problem you're solving may not actually exist or be important enough"
      }
      Self::NoAdoption => "Users may not adopt your solution despite having the problem",
      Self::BusinessModelFailure => "The business model may not be sustainable or profitable",
      Self::TechnicalInfeasibility => "The solution may be technically impossible or too complex",
      Self::WrongTiming => "The market timing may be wrong - too early or too late",
      Self::CompetitiveThreat => "Competition may prevent success",
      Self::ResourceConstraints => "Insufficient resources to execute successfully",
    }
  }

  /// Get all failure categories.
  #[must_use]
  pub const fn all() -> [Self; 7] {
    [
      Self::ProblemDoesntExist,
      Self::NoAdoption,
      Self::BusinessModelFailure,
      Self::TechnicalInfeasibility,
      Self::WrongTiming,
      Self::CompetitiveThreat,
      Self::ResourceConstraints,
    ]
  }
}

impl Antithesis {
  /// Create a new antithesis.
  #[must_use]
  pub fn new(statement: String, reasoning: String, category: FailureCategory) -> Self {
    Self {
      statement,
      reasoning,
      failure_category: category,
      probability: 0.3,
      supporting_evidence: Vec::new(),
      validation_approach: None,
    }
  }

  /// Set probability.
  #[must_use]
  pub fn with_probability(mut self, probability: f64) -> Self {
    self.probability = probability.clamp(0.0, 1.0);
    self
  }

  /// Add supporting evidence.
  #[must_use]
  pub fn with_evidence(mut self, evidence: String) -> Self {
    self.supporting_evidence.push(evidence);
    self
  }

  /// Set validation approach.
  #[must_use]
  pub fn with_validation(mut self, approach: String) -> Self {
    self.validation_approach = Some(approach);
    self
  }

  /// Check if this antithesis is falsifiable.
  #[must_use]
  pub fn is_falsifiable(&self) -> bool {
    self.validation_approach.is_some()
  }
}

// ============================================================================
// Validation Status
// ============================================================================

/// Status of thesis validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationStatus {
  /// Not yet validated
  Unvalidated,
  /// Partially validated
  PartiallyValidated,
  /// Thesis appears correct
  Validated,
  /// Antithesis appears correct (thesis falsified)
  Falsified,
  /// Inconclusive results
  Inconclusive,
}

// ============================================================================
// Output Types
// ============================================================================

/// Output from the Thesis & Antithesis Generator.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThesisOutput {
  /// The original thesis
  pub thesis: Thesis,
  /// Generated antitheses
  pub antitheses: Vec<Antithesis>,
  /// Overall validation status
  pub validation_status: ValidationStatus,
  /// Risk assessment
  pub risk_score: f64,
  /// Recommendations for validation
  pub recommendations: Vec<String>,
  /// Summary of key concerns
  pub key_concerns: Vec<String>,
}

// ============================================================================
// Error Types
// ============================================================================

/// Errors from the Thesis Generator.
#[derive(Debug, Error)]
pub enum ThesisError {
  /// Thesis statement is empty
  #[error("Thesis statement cannot be empty")]
  EmptyStatement,

  /// Rationale is missing
  #[error("Thesis rationale is required")]
  MissingRationale,

  /// Invalid confidence value
  #[error("Confidence must be between 0.0 and 1.0")]
  InvalidConfidence,

  /// Unable to generate antithesis
  #[error("Unable to generate antithesis for thesis")]
  AntithesisGenerationFailed,
}

// ============================================================================
// Generator Implementation
// ============================================================================

/// Thesis & Antithesis Generator.
///
/// Generates product theses with mandatory counter-arguments to prevent optimism bias.
pub struct ThesisAntithesisGenerator;

impl ThesisAntithesisGenerator {
  /// Generate antitheses for a thesis.
  ///
  /// # Errors
  ///
  /// Returns an error if the thesis is invalid.
  pub fn generate(thesis: Thesis) -> Result<ThesisOutput, ThesisError> {
    if thesis.statement.is_empty() {
      return Err(ThesisError::EmptyStatement);
    }
    if thesis.rationale.is_empty() {
      return Err(ThesisError::MissingRationale);
    }

    // Generate antitheses for each failure category
    let antitheses = Self::generate_antitheses(&thesis);

    // Calculate risk score
    let risk_score = Self::calculate_risk_score(&antitheses);

    // Generate recommendations
    let recommendations = Self::generate_recommendations(&thesis, &antitheses);

    // Extract key concerns
    let key_concerns = Self::extract_key_concerns(&antitheses);

    Ok(ThesisOutput {
      thesis,
      antitheses,
      validation_status: ValidationStatus::Unvalidated,
      risk_score,
      recommendations,
      key_concerns,
    })
  }

  /// Generate antitheses for all failure categories.
  fn generate_antitheses(thesis: &Thesis) -> Vec<Antithesis> {
    FailureCategory::all()
      .iter()
      .filter_map(|&category| Self::generate_antithesis_for_category(thesis, category))
      .collect()
  }

  /// Generate an antithesis for a specific failure category.
  fn generate_antithesis_for_category(
    thesis: &Thesis,
    category: FailureCategory,
  ) -> Option<Antithesis> {
    let (statement, reasoning) = match category {
      FailureCategory::ProblemDoesntExist => {
        let statement = format!(
          "The problem '{}' may not actually exist or matter to users",
          Self::extract_problem(&thesis.statement)
        );
        let reasoning = "Users may not experience this pain point frequently or severely enough to seek a solution".to_string();
        (statement, reasoning)
      }
      FailureCategory::NoAdoption => {
        let statement = "Users with this problem may not adopt this solution".to_string();
        let reasoning = "Existing workarounds, switching costs, or lack of awareness may prevent adoption despite problem existence".to_string();
        (statement, reasoning)
      }
      FailureCategory::BusinessModelFailure => {
        let statement = "The business model may not generate sustainable revenue".to_string();
        let reasoning = "Willingness to pay, cost of acquisition, or operational costs may make the model unviable".to_string();
        (statement, reasoning)
      }
      FailureCategory::TechnicalInfeasibility => {
        let statement = "The solution may be technically impossible or impractical".to_string();
        let reasoning = "Technical complexity, resource requirements, or fundamental limitations may prevent implementation".to_string();
        (statement, reasoning)
      }
      FailureCategory::WrongTiming => {
        let statement = "Market timing may be wrong for this solution".to_string();
        let reasoning = "Technology infrastructure, user readiness, or market conditions may not be mature enough".to_string();
        (statement, reasoning)
      }
      FailureCategory::CompetitiveThreat => {
        let statement = "Competition may prevent success in the market".to_string();
        let reasoning = "Existing or new competitors may have advantages in distribution, resources, or capabilities".to_string();
        (statement, reasoning)
      }
      FailureCategory::ResourceConstraints => {
        let statement = "Insufficient resources to execute successfully".to_string();
        let reasoning =
          "Funding, talent, time, or other resources may be inadequate to reach product-market fit"
            .to_string();
        (statement, reasoning)
      }
    };

    // Calculate probability based on thesis confidence
    let probability = 1.0 - thesis.confidence + 0.1;

    Some(
      Antithesis::new(statement, reasoning, category)
        .with_probability(probability)
        .with_validation(Self::suggest_validation(category)),
    )
  }

  /// Extract the problem description from a thesis statement.
  fn extract_problem(statement: &str) -> String {
    // Simple extraction - take first 50 chars or until first comma
    statement
      .split(',')
      .next()
      .map(|s| {
        let trimmed = s.trim();
        if trimmed.len() > 50 {
          format!("{}...", &trimmed[..47])
        } else {
          trimmed.to_string()
        }
      })
      .unwrap_or_else(|| statement.to_string())
  }

  /// Suggest a validation approach for a failure category.
  fn suggest_validation(category: FailureCategory) -> String {
    match category {
      FailureCategory::ProblemDoesntExist => {
        "Conduct 10+ customer interviews to validate problem existence and severity".to_string()
      }
      FailureCategory::NoAdoption => {
        "Run a landing page test or fake door test to measure willingness to adopt".to_string()
      }
      FailureCategory::BusinessModelFailure => {
        "Test willingness to pay through pricing experiments or pre-sales".to_string()
      }
      FailureCategory::TechnicalInfeasibility => {
        "Build a technical proof of concept or consult domain experts".to_string()
      }
      FailureCategory::WrongTiming => {
        "Analyze market signals, competitor activity, and technology adoption curves".to_string()
      }
      FailureCategory::CompetitiveThreat => {
        "Conduct competitive analysis and identify sustainable differentiation".to_string()
      }
      FailureCategory::ResourceConstraints => {
        "Create detailed resource plan and validate access to required resources".to_string()
      }
    }
  }

  /// Calculate overall risk score based on antitheses.
  fn calculate_risk_score(antitheses: &[Antithesis]) -> f64 {
    if antitheses.is_empty() {
      return 0.5;
    }

    let max_probability = antitheses
      .iter()
      .map(|a| a.probability)
      .fold(0.0_f64, |acc, p| acc.max(p));

    let avg_probability = antitheses.iter().map(|a| a.probability).sum::<f64>()
      / f64::from(u8::try_from(antitheses.len()).unwrap_or(1));

    // Combine max and average for risk score
    (max_probability * 0.6 + avg_probability * 0.4).clamp(0.0, 1.0)
  }

  /// Generate recommendations for validation.
  fn generate_recommendations(thesis: &Thesis, antitheses: &[Antithesis]) -> Vec<String> {
    let mut recommendations = Vec::new();

    // High-priority: validate the highest-risk antitheses
    let high_risk: Vec<_> = antitheses
      .iter()
      .filter(|a| a.probability > 0.5)
      .sorted_by(|a, b| {
        b.probability
          .partial_cmp(&a.probability)
          .unwrap_or(std::cmp::Ordering::Equal)
      })
      .take(3)
      .collect();

    for antithesis in high_risk {
      if let Some(ref validation) = antithesis.validation_approach {
        recommendations.push(format!(
          "HIGH PRIORITY ({:.0}% risk): {}",
          antithesis.probability * 100.0,
          validation
        ));
      }
    }

    // Check for unvalidated assumptions
    if thesis.assumptions.iter().any(|a| !a.is_empty()) {
      recommendations
        .push("Document and validate each assumption underlying the thesis".to_string());
    }

    // Check confidence alignment
    if thesis.confidence > 0.8 && !antitheses.is_empty() {
      recommendations.push(
        "High confidence warrants extra scrutiny - ensure it's based on evidence not optimism"
          .to_string(),
      );
    }

    recommendations
  }

  /// Extract key concerns from antitheses.
  fn extract_key_concerns(antitheses: &[Antithesis]) -> Vec<String> {
    antitheses
      .iter()
      .filter(|a| a.probability > 0.4)
      .map(|a| format!("{}: {}", a.failure_category.description(), a.statement))
      .collect()
  }

  /// Update validation status based on evidence.
  #[must_use]
  pub fn update_validation(
    mut output: ThesisOutput,
    antithesis_id: usize,
    evidence_supports_thesis: bool,
  ) -> ThesisOutput {
    if let Some(antithesis) = output.antitheses.get_mut(antithesis_id) {
      if evidence_supports_thesis {
        antithesis.probability *= 0.5; // Reduce antithesis probability
      } else {
        antithesis.probability = (antithesis.probability * 1.5).min(1.0);
      }
    }

    // Recalculate risk score
    output.risk_score = Self::calculate_risk_score(&output.antitheses);

    // Update validation status
    output.validation_status = Self::determine_validation_status(&output.antitheses);

    output
  }

  /// Determine overall validation status.
  fn determine_validation_status(antitheses: &[Antithesis]) -> ValidationStatus {
    let high_prob_count = antitheses.iter().filter(|a| a.probability > 0.7).count();
    let low_prob_count = antitheses.iter().filter(|a| a.probability < 0.2).count();
    let total = antitheses.len();

    if total == 0 {
      return ValidationStatus::Unvalidated;
    }

    let high_ratio = f64::from(u8::try_from(high_prob_count).unwrap_or(0))
      / f64::from(u8::try_from(total).unwrap_or(1));
    let low_ratio = f64::from(u8::try_from(low_prob_count).unwrap_or(0))
      / f64::from(u8::try_from(total).unwrap_or(1));

    if high_ratio > 0.5 {
      ValidationStatus::Falsified
    } else if low_ratio > 0.7 {
      ValidationStatus::Validated
    } else if low_ratio > 0.3 {
      ValidationStatus::PartiallyValidated
    } else {
      ValidationStatus::Inconclusive
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn create_test_thesis() -> Thesis {
    Thesis::new(
      "Users will pay $50/month for automated report generation".to_string(),
      "Saves 10 hours/week for enterprise analysts who currently do this manually".to_string(),
    )
    .with_assumption("Enterprise analysts exist and have budget authority".to_string())
    .with_confidence(0.6)
  }

  #[test]
  fn test_thesis_creation() {
    let thesis = create_test_thesis();

    assert!(!thesis.statement.is_empty());
    assert!(!thesis.rationale.is_empty());
    assert_eq!(thesis.assumptions.len(), 1);
    assert!((thesis.confidence - 0.6).abs() < 0.01);
  }

  #[test]
  fn test_thesis_is_valid() {
    let valid = Thesis::new("Statement".to_string(), "Rationale".to_string());
    assert!(valid.is_valid());

    let invalid = Thesis::new("".to_string(), "Rationale".to_string());
    assert!(!invalid.is_valid());
  }

  #[test]
  fn test_thesis_key_terms() {
    let thesis = Thesis::new(
      "Users will purchase automated report generation".to_string(),
      "Rationale".to_string(),
    );

    let terms = thesis.key_terms();
    assert!(terms.contains(&"Users".to_string()));
    assert!(terms.contains(&"purchase".to_string()));
    assert!(terms.contains(&"automated".to_string()));
    assert!(!terms.contains(&"will".to_string())); // stop word
  }

  #[test]
  fn test_antithesis_creation() {
    let antithesis = Antithesis::new(
      "Users won't pay".to_string(),
      "Budget constraints".to_string(),
      FailureCategory::BusinessModelFailure,
    )
    .with_probability(0.4)
    .with_evidence("Competitor offers free tier".to_string())
    .with_validation("Run pricing test".to_string());

    assert_eq!(antithesis.statement, "Users won't pay");
    assert_eq!(
      antithesis.failure_category,
      FailureCategory::BusinessModelFailure
    );
    assert!((antithesis.probability - 0.4).abs() < 0.01);
    assert!(antithesis.is_falsifiable());
  }

  #[test]
  fn test_failure_category_all() {
    let all = FailureCategory::all();
    assert_eq!(all.len(), 7);
  }

  #[test]
  fn test_generator_empty_thesis() {
    let thesis = Thesis::new("".to_string(), "Rationale".to_string());
    let result = ThesisAntithesisGenerator::generate(thesis);

    assert!(result.is_err());
    assert!(matches!(result, Err(ThesisError::EmptyStatement)));
  }

  #[test]
  fn test_generator_missing_rationale() {
    let thesis = Thesis::new("Statement".to_string(), "".to_string());
    let result = ThesisAntithesisGenerator::generate(thesis);

    assert!(result.is_err());
    assert!(matches!(result, Err(ThesisError::MissingRationale)));
  }

  #[test]
  fn test_generator_success() {
    let thesis = create_test_thesis();
    let result = ThesisAntithesisGenerator::generate(thesis);

    assert!(result.is_ok());
    let output = result.expect("Should succeed");

    assert!(!output.antitheses.is_empty());
    assert!(output.risk_score >= 0.0 && output.risk_score <= 1.0);
    assert!(!output.recommendations.is_empty());
  }

  #[test]
  fn test_generator_produces_all_categories() {
    let thesis = create_test_thesis();
    let output = ThesisAntithesisGenerator::generate(thesis).expect("Should succeed");

    let categories: Vec<_> = output
      .antitheses
      .iter()
      .map(|a| a.failure_category)
      .collect();

    for category in FailureCategory::all() {
      assert!(
        categories.contains(&category),
        "Missing antithesis for {:?}",
        category
      );
    }
  }

  #[test]
  fn test_risk_score_calculation() {
    let thesis = Thesis::new("Test".to_string(), "Rationale".to_string()).with_confidence(0.9);

    let output = ThesisAntithesisGenerator::generate(thesis).expect("Should succeed");

    // High confidence thesis should have lower risk antitheses
    assert!(output.risk_score < 0.5);
  }

  #[test]
  fn test_update_validation_supports_thesis() {
    let thesis = create_test_thesis();
    let output = ThesisAntithesisGenerator::generate(thesis).expect("Should succeed");

    let original_prob = output.antitheses[0].probability;
    let updated = ThesisAntithesisGenerator::update_validation(output, 0, true);

    assert!(updated.antitheses[0].probability < original_prob);
  }

  #[test]
  fn test_update_validation_refutes_thesis() {
    let thesis = create_test_thesis();
    let output = ThesisAntithesisGenerator::generate(thesis).expect("Should succeed");

    let original_prob = output.antitheses[0].probability;
    let updated = ThesisAntithesisGenerator::update_validation(output, 0, false);

    assert!(updated.antitheses[0].probability > original_prob);
  }

  #[test]
  fn test_key_concerns_extraction() {
    let thesis = Thesis::new("Test".to_string(), "Rationale".to_string()).with_confidence(0.2);

    let output = ThesisAntithesisGenerator::generate(thesis).expect("Should succeed");

    // Low confidence should result in more key concerns
    assert!(!output.key_concerns.is_empty());
  }

  #[test]
  fn test_validation_status_determination() {
    // Test falsified status (high probability antitheses)
    let high_risk = vec![
      Antithesis::new(
        "Test".to_string(),
        "Reason".to_string(),
        FailureCategory::NoAdoption,
      )
      .with_probability(0.8),
      Antithesis::new(
        "Test".to_string(),
        "Reason".to_string(),
        FailureCategory::BusinessModelFailure,
      )
      .with_probability(0.9),
    ];
    let status = ThesisAntithesisGenerator::determine_validation_status(&high_risk);
    assert_eq!(status, ValidationStatus::Falsified);

    // Test validated status (low probability antitheses)
    let low_risk = vec![
      Antithesis::new(
        "Test".to_string(),
        "Reason".to_string(),
        FailureCategory::NoAdoption,
      )
      .with_probability(0.1),
      Antithesis::new(
        "Test".to_string(),
        "Reason".to_string(),
        FailureCategory::BusinessModelFailure,
      )
      .with_probability(0.15),
    ];
    let status = ThesisAntithesisGenerator::determine_validation_status(&low_risk);
    assert_eq!(status, ValidationStatus::Validated);
  }

  #[test]
  fn test_antithesis_falsifiability() {
    let with_validation = Antithesis::new(
      "Test".to_string(),
      "Reason".to_string(),
      FailureCategory::NoAdoption,
    )
    .with_validation("Run test".to_string());
    assert!(with_validation.is_falsifiable());

    let without_validation = Antithesis::new(
      "Test".to_string(),
      "Reason".to_string(),
      FailureCategory::NoAdoption,
    );
    assert!(!without_validation.is_falsifiable());
  }

  #[test]
  fn test_confidence_clamping() {
    let thesis = Thesis::new("Test".to_string(), "Rationale".to_string()).with_confidence(1.5); // Invalid, should be clamped

    assert!((thesis.confidence - 1.0).abs() < 0.01);
  }

  #[test]
  fn test_probability_clamping() {
    let antithesis = Antithesis::new(
      "Test".to_string(),
      "Reason".to_string(),
      FailureCategory::NoAdoption,
    )
    .with_probability(-0.5); // Invalid, should be clamped

    assert!((antithesis.probability - 0.0).abs() < 0.01);
  }
}
