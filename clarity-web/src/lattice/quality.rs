//! Quality scoring for the Lattice analysis engine.
//!
//! This module implements the discovery-phase quality analysis using
//! the unified domain quality algebra.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::unused_self)]

use crate::domain::error::ClarityError;
use crate::domain::quality::{
  DimensionScore, IssueSeverity, QualityDimension, QualityEvaluator, QualityIssue, QualityReport,
  CONTRADICTORY_PHRASES, JARGON_TERMS, REQUIRED_FIELDS, SECURITY_KEYWORDS,
};
use crate::domain::types::Answer;
use std::collections::HashSet;

/// Evaluator for discovery-phase requirements (Answers).
pub struct LatticeQualityEvaluator;

impl QualityEvaluator<Vec<Answer>> for LatticeQualityEvaluator {
  fn evaluate(&self, answers: &Vec<Answer>) -> Result<QualityReport, ClarityError> {
    if answers.is_empty() {
      return Err(ClarityError::analysis("empty answers provided"));
    }

    let mut all_issues = Vec::new();

    // Calculate each dimension
    let completeness = self.calculate_completeness(answers, &mut all_issues);
    let consistency = self.calculate_consistency(answers, &mut all_issues);
    let testability = self.calculate_testability(answers, &mut all_issues);
    let clarity = self.calculate_clarity(answers, &mut all_issues);
    let security = self.calculate_security(answers, &mut all_issues);

    let dimensions = vec![completeness, consistency, testability, clarity, security];

    // Overall = simple average of 5 dimensions (can be updated to use weights)
    let overall = dimensions.iter().map(|d| u32::from(d.score)).sum::<u32>() / 5;

    Ok(QualityReport {
      overall_score: u8::try_from(overall).unwrap_or(0),
      dimensions,
      issues: all_issues,
    })
  }
}

impl LatticeQualityEvaluator {
  /// Calculate completeness: % of required fields filled.
  fn calculate_completeness(
    &self,
    answers: &[Answer],
    issues: &mut Vec<QualityIssue>,
  ) -> DimensionScore {
    let filled_count = REQUIRED_FIELDS
      .iter()
      .filter(|pattern| {
        answers
          .iter()
          .any(|a| a.step_id.contains(*pattern) && !a.value.trim().is_empty())
      })
      .count();

    for pattern in REQUIRED_FIELDS {
      let has_answer = answers
        .iter()
        .any(|a| a.step_id.contains(pattern) && !a.value.trim().is_empty());

      if !has_answer {
        issues.push(QualityIssue {
          dimension: QualityDimension::Completeness,
          severity: IssueSeverity::Error,
          message: format!("Missing required field: {pattern}"),
          suggestion: Some(format!("Please provide details for {pattern}")),
        });
      }
    }

    let score = (filled_count * 100) / REQUIRED_FIELDS.len();
    DimensionScore {
      dimension: QualityDimension::Completeness,
      score: u8::try_from(score).unwrap_or(0),
    }
  }

  /// Calculate consistency: detect contradictions.
  fn calculate_consistency(
    &self,
    answers: &[Answer],
    issues: &mut Vec<QualityIssue>,
  ) -> DimensionScore {
    let mut contradictions = 0_usize;
    let total_pairs = answers.len().saturating_sub(1);

    let values: Vec<_> = answers.iter().map(|a| a.value.to_lowercase()).collect();

    for (i, val1) in values.iter().enumerate() {
      for val2 in values.iter().skip(i + 1) {
        if CONTRADICTORY_PHRASES.iter().any(|(pos, neg)| {
          (val1.contains(pos) && val2.contains(neg)) || (val1.contains(neg) && val2.contains(pos))
        }) {
          contradictions += 1;
        }
      }
    }

    let score = if total_pairs > 0 {
      let contradiction_ratio = contradictions
        .saturating_mul(100)
        .checked_div(total_pairs)
        .unwrap_or(0);
      100_u32.saturating_sub(u32::try_from(contradiction_ratio).unwrap_or(0))
    } else {
      100
    };

    if contradictions > 0 {
      issues.push(QualityIssue {
        dimension: QualityDimension::Consistency,
        severity: IssueSeverity::Warning,
        message: format!("Found {contradictions} potential contradictions"),
        suggestion: Some(
          "Review requirements for conflicting 'must' vs 'must not' statements".to_string(),
        ),
      });
    }

    DimensionScore {
      dimension: QualityDimension::Consistency,
      score: u8::try_from(score).unwrap_or(0),
    }
  }

  /// Calculate testability: Presence of acceptance criteria.
  fn calculate_testability(
    &self,
    answers: &[Answer],
    issues: &mut Vec<QualityIssue>,
  ) -> DimensionScore {
    let has_criteria = answers
      .iter()
      .any(|a| a.step_id.contains("acceptance_criteria") && !a.value.trim().is_empty());

    let score = if has_criteria { 100 } else { 0 };

    if !has_criteria {
      issues.push(QualityIssue {
        dimension: QualityDimension::Testability,
        severity: IssueSeverity::Error,
        message: "No acceptance criteria defined".to_string(),
        suggestion: Some("Add specific, measurable criteria for success".to_string()),
      });
    }

    DimensionScore {
      dimension: QualityDimension::Testability,
      score,
    }
  }

  /// Calculate clarity: complexity and jargon.
  fn calculate_clarity(
    &self,
    answers: &[Answer],
    issues: &mut Vec<QualityIssue>,
  ) -> DimensionScore {
    let mut complex_count = 0;
    let mut jargon_count = 0;

    for answer in answers {
      let text = &answer.value;
      let word_count = text.split_whitespace().count();
      let comma_count = text.matches(',').count();

      if word_count > 30 || comma_count > 3 {
        complex_count += 1;
      }

      let lower = text.to_lowercase();
      jargon_count += JARGON_TERMS
        .iter()
        .filter(|term| lower.contains(*term))
        .count();
    }

    let jargon_penalty = (jargon_count * 5).min(50);
    let complex_penalty = (complex_count * 10).min(50);

    let score = 100_u32
      .saturating_sub(u32::try_from(jargon_penalty).unwrap_or(0))
      .saturating_sub(u32::try_from(complex_penalty).unwrap_or(0));

    if complex_count > 0 {
      issues.push(QualityIssue {
        dimension: QualityDimension::Clarity,
        severity: IssueSeverity::Warning,
        message: format!("{complex_count} complex sentences detected"),
        suggestion: Some("Break long sentences into shorter, clearer requirements".to_string()),
      });
    }

    DimensionScore {
      dimension: QualityDimension::Clarity,
      score: u8::try_from(score).unwrap_or(0),
    }
  }

  /// Calculate security: Coverage of security areas.
  fn calculate_security(
    &self,
    answers: &[Answer],
    issues: &mut Vec<QualityIssue>,
  ) -> DimensionScore {
    let mut covered_areas = HashSet::new();

    for answer in answers {
      let lower = answer.value.to_lowercase();
      for keyword in SECURITY_KEYWORDS {
        if lower.contains(keyword) {
          if keyword.contains("auth") || keyword.contains("login") {
            covered_areas.insert("authentication");
          }
          if keyword.contains("encrypt") || keyword.contains("tls") {
            covered_areas.insert("encryption");
          }
          if keyword.contains("validat") || keyword.contains("sanitiz") {
            covered_areas.insert("validation");
          }
        }
      }
    }

    let score = (covered_areas.len() * 33).min(100);

    if covered_areas.is_empty() {
      issues.push(QualityIssue {
        dimension: QualityDimension::Security,
        severity: IssueSeverity::Warning,
        message: "No security considerations mentioned".to_string(),
        suggestion: Some(
          "Consider adding requirements for authentication, encryption, or input validation"
            .to_string(),
        ),
      });
    }

    DimensionScore {
      dimension: QualityDimension::Security,
      score: u8::try_from(score).unwrap_or(0),
    }
  }
}
