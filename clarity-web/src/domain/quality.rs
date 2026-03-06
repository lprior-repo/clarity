//! Quality Algebra
//!
//! Unified quality reporting and evaluation traits.

use crate::domain::error::ClarityError;
use serde::{Deserialize, Serialize};

/// Mandatory gate threshold for quality scores.
pub const QUALITY_GATE_THRESHOLD: u8 = 70;

/// Fields required for a "complete" requirement capture.
pub const REQUIRED_FIELDS: &[&str] = &[
  "user_goal",
  "actors",
  "precondition",
  "outcome",
  "acceptance_criteria",
];

/// Jargon terms that penalize clarity if used without explanation.
pub const JARGON_TERMS: &[&str] = &[
  "microservice",
  "kubernetes",
  "orchestration",
  "containerization",
  "blockchain",
  "ai/ml",
  "serverless",
  "event-driven",
];

/// Keywords that indicate security considerations.
pub const SECURITY_KEYWORDS: &[&str] = &[
  "auth",
  "authentication",
  "authorization",
  "login",
  "password",
  "encrypt",
  "decrypt",
  "hash",
  "salt",
  "tls",
  "ssl",
  "https",
  "validat",
  "sanitiz",
  "escape",
  "csrf",
  "xss",
  "injection",
];

/// Phrases that indicate potential contradictions.
pub const CONTRADICTORY_PHRASES: &[(&str, &str)] = &[
  ("must", "must not"),
  ("required", "optional"),
  ("always", "never"),
  ("enabled", "disabled"),
  ("allow", "deny"),
  ("include", "exclude"),
];

/// Configuration for quality scoring weights and thresholds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QualityConfig {
  pub gate_threshold: u8,
  pub weights: QualityWeights,
}

impl Default for QualityConfig {
  fn default() -> Self {
    Self {
      gate_threshold: QUALITY_GATE_THRESHOLD,
      weights: QualityWeights::default(),
    }
  }
}

/// Weights for each quality dimension (must sum to 100).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QualityWeights {
  pub completeness: u8,
  pub consistency: u8,
  pub testability: u8,
  pub clarity: u8,
  pub security: u8,
}

impl Default for QualityWeights {
  fn default() -> Self {
    Self {
      completeness: 20,
      consistency: 20,
      testability: 20,
      clarity: 20,
      security: 20,
    }
  }
}

/// Quality dimensions evaluated by the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum QualityDimension {
  /// Percentage of required fields filled.
  Completeness,
  /// Detection of contradictory requirements.
  Consistency,
  /// Presence of acceptance criteria in requirements.
  Testability,
  /// Sentence complexity and jargon density.
  Clarity,
  /// Security considerations (auth, encryption, validation).
  Security,
}

impl QualityDimension {
  /// Returns all available dimensions.
  #[must_use]
  pub const fn all() -> &'static [Self] {
    &[
      Self::Completeness,
      Self::Consistency,
      Self::Testability,
      Self::Clarity,
      Self::Security,
    ]
  }

  /// Display label for the dimension.
  #[must_use]
  pub const fn label(self) -> &'static str {
    match self {
      Self::Completeness => "Completeness",
      Self::Consistency => "Consistency",
      Self::Testability => "Testability",
      Self::Clarity => "Clarity",
      Self::Security => "Security",
    }
  }

  /// Description of what this dimension measures.
  #[must_use]
  pub const fn description(self) -> &'static str {
    match self {
      Self::Completeness => "Percentage of required fields filled",
      Self::Consistency => "Absence of contradictory requirements",
      Self::Testability => "Presence of acceptance criteria",
      Self::Clarity => "Readability and minimal jargon",
      Self::Security => "Security considerations present",
    }
  }
}

/// Severity of a quality issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
  Warning,
  Error,
  Critical,
}

/// A specific issue detected during quality analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityIssue {
  pub dimension: QualityDimension,
  pub severity: IssueSeverity,
  pub message: String,
  pub suggestion: Option<String>,
}

/// Score for a single dimension (0-100).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DimensionScore {
  pub dimension: QualityDimension,
  pub score: u8,
}

impl DimensionScore {
  /// Check if the score passes a given threshold.
  #[must_use]
  pub const fn passes(self, threshold: u8) -> bool {
    self.score >= threshold
  }
}

/// Unified quality assessment report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityReport {
  /// Overall score 0-100 (weighted average).
  pub overall_score: u8,
  /// Individual dimension scores.
  pub dimensions: Vec<DimensionScore>,
  /// Collection of identified issues.
  pub issues: Vec<QualityIssue>,
}

impl QualityReport {
  /// Check if the overall score passes the mandatory gate threshold.
  #[must_use]
  pub const fn gate_passes(&self) -> bool {
    self.overall_score >= QUALITY_GATE_THRESHOLD
  }

  /// Get the score for a specific dimension.
  #[must_use]
  pub fn get_score(&self, dimension: QualityDimension) -> u8 {
    self.get_dimension(dimension).map_or(0, |d| d.score)
  }

  /// Get the score object for a specific dimension.
  #[must_use]
  pub fn get_dimension(&self, dimension: QualityDimension) -> Option<&DimensionScore> {
    self.dimensions.iter().find(|d| d.dimension == dimension)
  }
}

/// Trait for types that can be evaluated for quality.
pub trait QualityEvaluator<T> {
  /// Performs quality analysis on the input.
  ///
  /// # Errors
  /// Returns `ClarityError` if the quality analysis fails.
  fn evaluate(&self, input: &T) -> Result<QualityReport, ClarityError>;
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
  use super::*;

  #[test]
  fn test_quality_report_serialization() {
    let report = QualityReport {
      overall_score: 85,
      dimensions: vec![DimensionScore {
        dimension: QualityDimension::Security,
        score: 90,
      }],
      issues: vec![QualityIssue {
        dimension: QualityDimension::Clarity,
        severity: IssueSeverity::Warning,
        message: "Too much jargon".to_string(),
        suggestion: Some("Explain terms".to_string()),
      }],
    };
    let json = serde_json::to_string(&report).unwrap();
    let decoded: QualityReport = serde_json::from_str(&json).unwrap();
    assert_eq!(report, decoded);
  }

  #[test]
  fn test_gate_passes() {
    let mut report = QualityReport {
      overall_score: 70,
      dimensions: vec![],
      issues: vec![],
    };
    assert!(report.gate_passes());
    report.overall_score = 69;
    assert!(!report.gate_passes());
  }
}
