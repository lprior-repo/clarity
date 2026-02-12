//! EQI (External Quality Internal) framework.
//!
//! Evaluates product and system quality along five dimensions:
//! - Completeness
//! - Consistency
//! - Testability
//! - Clarity
//! - Security

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

pub const MIN_SCORE: f32 = 0.0;
pub const MAX_SCORE: f32 = 1.0;
pub const WEAK_DIMENSION_THRESHOLD: f32 = 0.6;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityDimension {
  Completeness,
  Consistency,
  Testability,
  Clarity,
  Security,
}

impl QualityDimension {
  #[must_use]
  pub fn all() -> &'static [Self] {
    &[
      Self::Completeness,
      Self::Consistency,
      Self::Testability,
      Self::Clarity,
      Self::Security,
    ]
  }

  #[must_use]
  pub const fn improvement_hint(&self) -> &'static str {
    match self {
      Self::Completeness => "Cover missing use-cases, edge-cases, and failure paths",
      Self::Consistency => "Unify rules and behavior across similar workflows",
      Self::Testability => "Make behavior observable and deterministic in tests",
      Self::Clarity => "Reduce ambiguity in naming, docs, and user-facing text",
      Self::Security => "Add validation, hardening, and abuse-case handling",
    }
  }
}

impl fmt::Display for QualityDimension {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Completeness => write!(f, "Completeness"),
      Self::Consistency => write!(f, "Consistency"),
      Self::Testability => write!(f, "Testability"),
      Self::Clarity => write!(f, "Clarity"),
      Self::Security => write!(f, "Security"),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DimensionScore {
  pub dimension: QualityDimension,
  pub score: f32,
  pub rationale: String,
}

impl DimensionScore {
  pub fn new(
    dimension: QualityDimension,
    score: f32,
    rationale: String,
  ) -> Result<Self, QualityDimensionsError> {
    if rationale.trim().is_empty() {
      return Err(QualityDimensionsError::EmptyField("rationale".to_string()));
    }
    if !(MIN_SCORE..=MAX_SCORE).contains(&score) {
      return Err(QualityDimensionsError::InvalidScore { score });
    }

    Ok(Self {
      dimension,
      score,
      rationale: rationale.trim().to_string(),
    })
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EQIAssessment {
  pub id: Uuid,
  pub subject: String,
  pub scores: Vec<DimensionScore>,
  pub overall_score: f32,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl EQIAssessment {
  pub fn new(subject: String) -> Result<Self, QualityDimensionsError> {
    if subject.trim().is_empty() {
      return Err(QualityDimensionsError::EmptyField("subject".to_string()));
    }

    let now = Utc::now();
    Ok(Self {
      id: Uuid::new_v4(),
      subject: subject.trim().to_string(),
      scores: Vec::new(),
      overall_score: 0.0,
      created_at: now,
      updated_at: now,
    })
  }

  #[must_use]
  pub fn with_score(self, score: DimensionScore) -> Self {
    let score_dimension = score.dimension;
    let scores: Vec<DimensionScore> = self
      .scores
      .into_iter()
      .filter(|entry| entry.dimension != score_dimension)
      .chain(std::iter::once(score.clone()))
      .collect();
    let overall_score = compute_overall_score(&scores);

    Self {
      scores,
      overall_score,
      updated_at: Utc::now(),
      ..self
    }
  }

  #[must_use]
  pub fn score_for(&self, dimension: QualityDimension) -> Option<f32> {
    self
      .scores
      .iter()
      .find(|entry| entry.dimension == dimension)
      .map(|entry| entry.score)
  }

  #[must_use]
  pub fn weak_dimensions(&self) -> Vec<QualityDimension> {
    QualityDimension::all()
      .iter()
      .copied()
      .filter(|dimension| {
        self
          .score_for(*dimension)
          .is_none_or(|score| score < WEAK_DIMENSION_THRESHOLD)
      })
      .collect()
  }

  #[must_use]
  pub fn recommendations(&self) -> Vec<String> {
    self
      .weak_dimensions()
      .into_iter()
      .map(|dimension| format!("{}: {}", dimension, dimension.improvement_hint()))
      .collect()
  }

  #[must_use]
  pub fn is_complete(&self) -> bool {
    self.scores.len() == QualityDimension::all().len()
  }
}

fn compute_overall_score(scores: &[DimensionScore]) -> f32 {
  if scores.is_empty() {
    return 0.0;
  }

  let sum: f32 = scores.iter().map(|entry| entry.score).sum();
  sum / scores.len() as f32
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum QualityDimensionsError {
  #[error("field cannot be empty: {0}")]
  EmptyField(String),

  #[error("score must be in range [0.0, 1.0], got {score}")]
  InvalidScore { score: f32 },
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dimension_score_rejects_invalid_score() {
    let score = DimensionScore::new(
      QualityDimension::Security,
      1.5,
      "missing input validation".to_string(),
    );
    assert!(matches!(
      score,
      Err(QualityDimensionsError::InvalidScore { score }) if score == 1.5
    ));
  }

  #[test]
  fn assessment_flags_missing_dimensions_as_weak() {
    let assessment_result = EQIAssessment::new("checkout flow".to_string());
    assert!(assessment_result.is_ok());
    let assessment = match assessment_result {
      Ok(assessment) => assessment,
      Err(_) => return,
    };
    let score_result = DimensionScore::new(
      QualityDimension::Completeness,
      0.8,
      "covers happy and error paths".to_string(),
    );
    assert!(score_result.is_ok());
    let assessment = assessment.with_score(match score_result {
      Ok(score) => score,
      Err(_) => return,
    });

    let weak = assessment.weak_dimensions();
    assert!(weak.contains(&QualityDimension::Security));
    assert!(weak.contains(&QualityDimension::Clarity));
  }

  #[test]
  fn complete_assessment_tracks_overall_score() {
    let assessment_result = EQIAssessment::new("checkout flow".to_string());
    assert!(assessment_result.is_ok());

    let assessment = match assessment_result {
      Ok(assessment) => assessment,
      Err(_) => return,
    };

    let assessment_result =
      QualityDimension::all()
        .iter()
        .copied()
        .try_fold(assessment, |current, dimension| {
          let score_result =
            DimensionScore::new(dimension, 0.9, format!("{dimension} looks strong"));
          assert!(score_result.is_ok());
          score_result.map(|score| current.with_score(score))
        });
    assert!(assessment_result.is_ok());
    let assessment = match assessment_result {
      Ok(assessment) => assessment,
      Err(_) => return,
    };

    assert!(assessment.is_complete());
    assert!(assessment.overall_score >= 0.9);
  }
}
