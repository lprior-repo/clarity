//! Quality Dimensions Module - EQI (Engineering Quality Index) Framework
//!
//! Defines quality dimensions and assessment methods for evaluating
//! engineering artifacts through systematic quality analysis.

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
// QUALITY DIMENSION TYPES
// ============================================================================

/// Quality dimensions in the EQI Framework
///
/// These dimensions represent the key aspects of engineering quality
/// that should be assessed for any artifact or deliverable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityDimension {
  /// All requirements and features are fully implemented
  Completeness,
  /// Uniform design patterns, naming conventions, and behaviors
  Consistency,
  /// Ability to verify correctness through automated tests
  Testability,
  /// Clear documentation, naming, and understandability
  Clarity,
  /// Protection against vulnerabilities and threats
  Security,
  /// Response time, throughput, and resource efficiency
  Performance,
  /// Ease of modification, extension, and debugging
  Maintainability,
}

impl fmt::Display for QualityDimension {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Completeness => write!(f, "Completeness"),
      Self::Consistency => write!(f, "Consistency"),
      Self::Testability => write!(f, "Testability"),
      Self::Clarity => write!(f, "Clarity"),
      Self::Security => write!(f, "Security"),
      Self::Performance => write!(f, "Performance"),
      Self::Maintainability => write!(f, "Maintainability"),
    }
  }
}

// ============================================================================
// QUALITY METRIC
// ============================================================================

/// A measurement of quality for a specific dimension
///
/// Represents an assessment of how well an artifact scores on a
/// particular quality dimension, with supporting evidence.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QualityMetric {
  /// The dimension being measured
  pub dimension: QualityDimension,
  /// Score from 0.0 (poor) to 1.0 (excellent)
  pub score: f32,
  /// Evidence supporting this score
  pub evidence: String,
  /// When the assessment was made
  pub assessed_at: DateTime<Utc>,
  /// Unique identifier
  pub id: Uuid,
}

impl QualityMetric {
  /// Threshold below which a dimension is considered weak
  const WEAK_THRESHOLD: f32 = 0.5;
  /// Threshold above which a dimension is considered strong
  const STRONG_THRESHOLD: f32 = 0.8;

  /// Create a new quality metric
  ///
  /// # Errors
  /// Returns `QualityDimensionError::InvalidScore` if score is not in [0.0, 1.0]
  /// Returns `QualityDimensionError::EmptyEvidence` if evidence is empty
  pub fn new(
    dimension: QualityDimension,
    score: f32,
    evidence: String,
  ) -> Result<Self, QualityDimensionError> {
    if !(0.0..=1.0).contains(&score) {
      return Err(QualityDimensionError::InvalidScore { score });
    }
    if evidence.trim().is_empty() {
      return Err(QualityDimensionError::EmptyEvidence);
    }

    Ok(Self {
      id: Uuid::new_v4(),
      dimension,
      score,
      evidence,
      assessed_at: Utc::now(),
    })
  }

  /// Check if this metric indicates a weak dimension
  #[must_use]
  pub fn is_weak(&self) -> bool {
    self.score < Self::WEAK_THRESHOLD
  }

  /// Check if this metric indicates a strong dimension
  #[must_use]
  pub fn is_strong(&self) -> bool {
    self.score >= Self::STRONG_THRESHOLD
  }
}

// ============================================================================
// IMPROVEMENT ACTION
// ============================================================================

/// An actionable recommendation for improving a quality dimension
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImprovementAction {
  /// The dimension to improve
  pub dimension: QualityDimension,
  /// Description of the improvement action
  pub description: String,
  /// Priority (1 = highest)
  pub priority: usize,
}

impl ImprovementAction {
  /// Create a new improvement action
  #[must_use]
  pub fn new(dimension: QualityDimension, description: String, priority: usize) -> Self {
    Self {
      dimension,
      description,
      priority,
    }
  }
}

// ============================================================================
// EQI ASSESSMENT
// ============================================================================

/// Complete quality assessment for an artifact
///
/// Contains metrics for all assessed dimensions, an overall score,
/// and recommendations for improvement.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EQIAssessment {
  /// Unique identifier
  pub id: Uuid,
  /// Metrics for each assessed dimension
  pub metrics: Vec<QualityMetric>,
  /// Overall quality score (average of all metrics)
  pub overall_score: f32,
  /// Recommendations for improvement
  pub recommendations: Vec<String>,
  /// When the assessment was created
  pub created_at: DateTime<Utc>,
  /// When the assessment was last updated
  pub updated_at: DateTime<Utc>,
}

impl EQIAssessment {
  /// Create a new empty assessment
  #[must_use]
  pub fn new() -> Self {
    let now = Utc::now();
    Self {
      id: Uuid::new_v4(),
      metrics: Vec::new(),
      overall_score: 0.0,
      recommendations: Vec::new(),
      created_at: now,
      updated_at: now,
    }
  }

  /// Add a metric to the assessment
  #[must_use]
  pub fn with_metric(mut self, metric: QualityMetric) -> Self {
    self.metrics.push(metric);
    self.updated_at = Utc::now();
    self
  }

  /// Add a recommendation to the assessment
  #[must_use]
  pub fn with_recommendation(mut self, recommendation: String) -> Self {
    self.recommendations.push(recommendation);
    self.updated_at = Utc::now();
    self
  }

  /// Set the overall score
  #[must_use]
  pub fn with_overall_score(mut self, score: f32) -> Self {
    self.overall_score = score.clamp(0.0, 1.0);
    self.updated_at = Utc::now();
    self
  }

  /// Calculate the overall score from metrics
  ///
  /// Returns the average of all metric scores, or 0.0 if no metrics
  #[must_use]
  pub fn calculate_overall_score(&self) -> f32 {
    if self.metrics.is_empty() {
      return 0.0;
    }

    let total: f32 = self.metrics.iter().map(|m| m.score).sum();
    let average = total / self.metrics.len() as f32;
    average.clamp(0.0, 1.0)
  }

  /// Get all dimensions that score below the weak threshold
  #[must_use]
  pub fn get_weak_dimensions(&self) -> Vec<QualityDimension> {
    self
      .metrics
      .iter()
      .filter(|m| m.is_weak())
      .map(|m| m.dimension)
      .collect()
  }

  /// Check if there are any weak dimensions
  #[must_use]
  pub fn has_weak_dimensions(&self) -> bool {
    self.metrics.iter().any(|m| m.is_weak())
  }

  /// Get the metric for a specific dimension, if present
  #[must_use]
  pub fn get_metric_for_dimension(&self, dimension: QualityDimension) -> Option<&QualityMetric> {
    self.metrics.iter().find(|m| m.dimension == dimension)
  }

  /// Generate an improvement plan based on weak dimensions
  ///
  /// Returns prioritized actions sorted by urgency (lowest scores first)
  #[must_use]
  pub fn generate_improvement_plan(&self) -> Vec<ImprovementAction> {
    let weak_metrics: Vec<&QualityMetric> = self.metrics.iter().filter(|m| m.is_weak()).collect();

    // Sort by score ascending (weakest first for priority)
    let mut sorted_weak: Vec<&QualityMetric> = weak_metrics;
    sorted_weak.sort_by(|a, b| {
      a.score
        .partial_cmp(&b.score)
        .unwrap_or(std::cmp::Ordering::Equal)
    });

    sorted_weak
      .iter()
      .enumerate()
      .map(|(index, metric)| {
        let description = Self::generate_improvement_description(metric.dimension, metric.score);
        ImprovementAction::new(metric.dimension, description, index + 1)
      })
      .collect()
  }

  /// Generate a description for improving a dimension
  fn generate_improvement_description(dimension: QualityDimension, score: f32) -> String {
    let urgency = if score < 0.3 {
      "Critical"
    } else if score < 0.4 {
      "High"
    } else {
      "Medium"
    };

    let action = match dimension {
      QualityDimension::Completeness => "Address missing requirements and incomplete features",
      QualityDimension::Consistency => "Standardize patterns, conventions, and behaviors",
      QualityDimension::Testability => "Increase test coverage and improve test infrastructure",
      QualityDimension::Clarity => "Improve documentation, naming, and code organization",
      QualityDimension::Security => "Address security vulnerabilities and strengthen defenses",
      QualityDimension::Performance => "Optimize bottlenecks and improve resource efficiency",
      QualityDimension::Maintainability => "Reduce complexity and improve code maintainability",
    };

    format!("[{urgency}] {action}")
  }

  /// Perform a complete assessment from a list of metrics
  ///
  /// # Errors
  /// This function does not fail, but returns an empty assessment for empty input
  pub fn assess(metrics: Vec<QualityMetric>) -> Result<Self, QualityDimensionError> {
    let mut assessment = Self::new();

    for metric in metrics {
      assessment = assessment.with_metric(metric);
    }

    let overall = assessment.calculate_overall_score();
    assessment = assessment.with_overall_score(overall);

    // Generate automatic recommendations for weak dimensions
    let weak_dims = assessment.get_weak_dimensions();
    for dim in weak_dims {
      let recommendation = Self::generate_recommendation(dim);
      assessment = assessment.with_recommendation(recommendation);
    }

    Ok(assessment)
  }

  /// Generate a recommendation for a weak dimension
  fn generate_recommendation(dimension: QualityDimension) -> String {
    match dimension {
      QualityDimension::Completeness => {
        "Review and address any incomplete requirements or missing features".to_string()
      }
      QualityDimension::Consistency => {
        "Establish and enforce consistent patterns across the codebase".to_string()
      }
      QualityDimension::Testability => {
        "Increase automated test coverage and improve test quality".to_string()
      }
      QualityDimension::Clarity => "Improve documentation clarity and code readability".to_string(),
      QualityDimension::Security => {
        "Conduct security review and address identified vulnerabilities".to_string()
      }
      QualityDimension::Performance => "Profile and optimize performance bottlenecks".to_string(),
      QualityDimension::Maintainability => {
        "Refactor complex code and improve architectural organization".to_string()
      }
    }
  }
}

impl Default for EQIAssessment {
  fn default() -> Self {
    Self::new()
  }
}

// ============================================================================
// ERRORS
// ============================================================================

/// Errors for the quality dimensions module
#[derive(Debug, Error, PartialEq)]
pub enum QualityDimensionError {
  /// Score was not in valid range [0.0, 1.0]
  #[error("invalid score: {score}. Must be between 0.0 and 1.0")]
  InvalidScore { score: f32 },

  /// Evidence field was empty
  #[error("evidence cannot be empty")]
  EmptyEvidence,

  /// Validation failed
  #[error("validation failed: {0}")]
  ValidationFailed(String),
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn all_quality_dimensions_have_display() {
    let dimensions = [
      QualityDimension::Completeness,
      QualityDimension::Consistency,
      QualityDimension::Testability,
      QualityDimension::Clarity,
      QualityDimension::Security,
      QualityDimension::Performance,
      QualityDimension::Maintainability,
    ];

    for dimension in dimensions {
      let display = dimension.to_string();
      assert!(!display.is_empty());
    }
  }
}
