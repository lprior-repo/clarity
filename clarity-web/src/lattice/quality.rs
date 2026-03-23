#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro
)]
#![warn(clippy::nursery)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::use_self)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::unnecessary_lazy_evaluations)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::single_char_pattern)]
#![allow(clippy::needless_collect)]
#![forbid(unsafe_code)]

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

// Re-export Answer from types to use as the canonical Answer type
pub use crate::types::Answer;

/// Domain errors for quality scoring
#[derive(Debug, Error, PartialEq, Clone)]
pub enum QualityError {
  #[error("empty answers provided")]
  EmptyAnswers,

  #[error("invalid score value: {0}")]
  InvalidScore(String),

  #[error("dimension calculation failed: {0}")]
  DimensionFailed(String),
}

/// Quality dimensions evaluated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum QualityDimension {
  /// Percentage of required fields filled
  Completeness,

  /// Detection of contradictory requirements
  Consistency,

  /// Presence of acceptance criteria in EARS requirements
  Testability,

  /// Sentence complexity and jargon density
  Clarity,

  /// Security considerations (auth, encryption, validation)
  Security,
}

impl QualityDimension {
  /// All dimensions
  pub fn all() -> &'static [QualityDimension] {
    &[
      QualityDimension::Completeness,
      QualityDimension::Consistency,
      QualityDimension::Testability,
      QualityDimension::Clarity,
      QualityDimension::Security,
    ]
  }

  /// Display label
  pub fn label(self) -> &'static str {
    match self {
      QualityDimension::Completeness => "Completeness",
      QualityDimension::Consistency => "Consistency",
      QualityDimension::Testability => "Testability",
      QualityDimension::Clarity => "Clarity",
      QualityDimension::Security => "Security",
    }
  }

  /// Description of what this dimension measures
  pub fn description(self) -> &'static str {
    match self {
      QualityDimension::Completeness => "Percentage of required fields filled",
      QualityDimension::Consistency => "Absence of contradictory requirements",
      QualityDimension::Testability => "Presence of acceptance criteria",
      QualityDimension::Clarity => "Readability and minimal jargon",
      QualityDimension::Security => "Security considerations present",
    }
  }
}

/// Score for a single dimension (0-100)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DimensionScore {
  pub dimension: QualityDimension,
  pub score: u8,
}

impl DimensionScore {
  /// Create a new dimension score, validating the range
  pub fn new(dimension: QualityDimension, score: u8) -> Result<Self, QualityError> {
    match score {
      0..=100 => Ok(DimensionScore { dimension, score }),
      invalid => Err(QualityError::InvalidScore(invalid.to_string())),
    }
  }

  /// Check if score passes threshold
  pub fn passes(self, threshold: u8) -> bool {
    self.score >= threshold
  }
}

/// Issue explaining a low score
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityIssue {
  pub dimension: QualityDimension,
  pub severity: IssueSeverity,
  pub message: String,
}

impl QualityIssue {
  pub fn new(dimension: QualityDimension, severity: IssueSeverity, message: String) -> Self {
    Self {
      dimension,
      severity,
      message,
    }
  }
}

/// Severity of a quality issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
  Warning,
  Error,
  Critical,
}

/// Overall quality assessment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityScore {
  /// Overall score 0-100 (average of dimensions)
  pub overall: u8,
  /// Individual dimension scores
  pub dimensions: Vec<DimensionScore>,
  /// Issues explaining low scores
  pub issues: Vec<QualityIssue>,
}

impl QualityScore {
  /// Create a new quality score
  pub fn new(
    overall: u8,
    dimensions: Vec<DimensionScore>,
    issues: Vec<QualityIssue>,
  ) -> Result<Self, QualityError> {
    match overall {
      0..=100 => Ok(QualityScore {
        overall,
        dimensions,
        issues,
      }),
      invalid => Err(QualityError::InvalidScore(invalid.to_string())),
    }
  }

  /// Check if overall score passes threshold
  pub fn passes(&self, threshold: u8) -> bool {
    self.overall >= threshold
  }

  /// Get score for a specific dimension
  pub fn get_dimension(&self, dimension: QualityDimension) -> Option<&DimensionScore> {
    self.dimensions.iter().find(|d| d.dimension == dimension)
  }

  /// Get issues for a specific dimension
  pub fn get_issues(&self, dimension: QualityDimension) -> Vec<&QualityIssue> {
    self
      .issues
      .iter()
      .filter(|i| i.dimension == dimension)
      .collect()
  }
}

/// EARS requirement reference for quality scoring
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EarsRequirementRef {
  pub id: String,
  pub text: String,
  pub has_acceptance_criteria: bool,
}

/// Inversion control (requirement inversion for testing)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InversionControl {
  pub has_inversion_tests: bool,
  pub inverted_count: usize,
}

/// Calculate quality score from requirements data
///
/// # Arguments
/// * `answers` - User answers to prompt steps
/// * `ears` - EARS-formatted requirements
/// * `inversion` - Inversion control data
///
/// # Returns
/// Quality score with all dimensions evaluated
pub fn calculate_quality(
  answers: &[Answer],
  ears: &[EarsRequirementRef],
  _inversion: &InversionControl,
) -> Result<QualityScore, QualityError> {
  if answers.is_empty() {
    return Err(QualityError::EmptyAnswers);
  }

  let mut all_issues = Vec::new();

  // Calculate each dimension
  let completeness = calculate_completeness(answers, &mut all_issues);
  let consistency = calculate_consistency(answers, &mut all_issues);
  let testability = calculate_testability(ears, &mut all_issues);
  let clarity = calculate_clarity(answers, &mut all_issues);
  let security = calculate_security(answers, &mut all_issues);

  let dimensions = vec![completeness, consistency, testability, clarity, security];

  // Overall = average of 5 dimensions
  let overall =
    dimensions.iter().map(|d| u32::from(d.score)).sum::<u32>() / dimensions.len() as u32;

  let overall = u8::try_from(overall)
    .map_err(|_| QualityError::InvalidScore("overall calculation overflow".to_string()))?;

  QualityScore::new(overall, dimensions, all_issues)
}

/// Calculate completeness: % of required fields filled
fn calculate_completeness(answers: &[Answer], issues: &mut Vec<QualityIssue>) -> DimensionScore {
  let required_patterns = [
    "user_goal",
    "actors",
    "precondition",
    "outcome",
    "acceptance_criteria",
  ];

  let total_required = required_patterns.len();

  let filled_count = required_patterns
    .iter()
    .filter(|pattern| {
      answers
        .iter()
        .any(|a| a.step_id.contains(*pattern) && !a.value.trim().is_empty())
    })
    .count();

  // Check for empty required fields
  issues.extend(
    required_patterns
      .iter()
      .filter(|pattern| {
        !answers
          .iter()
          .any(|a| a.step_id.contains(*pattern) && !a.value.trim().is_empty())
      })
      .map(|pattern| {
        QualityIssue::new(
          QualityDimension::Completeness,
          IssueSeverity::Error,
          format!("Missing required field: {pattern}"),
        )
      }),
  );

  let score = if total_required > 0 {
    u8::try_from(
      filled_count
        .saturating_mul(100)
        .checked_div(total_required)
        .map_or(0, |v| v),
    )
    .map_or(100, |v| v)
  } else {
    100
  };

  DimensionScore::new(QualityDimension::Completeness, score).map_or_else(|_| DimensionScore {
    dimension: QualityDimension::Completeness,
    score: 0,
  }, |v| v)
}

/// Calculate consistency: detect contradictions
fn calculate_consistency(answers: &[Answer], issues: &mut Vec<QualityIssue>) -> DimensionScore {
  let total_pairs = answers.len().saturating_sub(1);

  // Simple contradiction detection: look for negations of similar concepts
  let values: Vec<_> = answers.iter().map(|a| a.value.to_lowercase()).collect();

  let contradictions = values
    .iter()
    .enumerate()
    .flat_map(|(i, val1)| values.iter().skip(i + 1).map(move |val2| (val1, val2)))
    .filter(|(val1, val2)| has_contradiction(val1, val2))
    .count();

  // Score based on contradiction ratio
  let score = if total_pairs > 0 {
    let contradiction_ratio = contradictions
      .saturating_mul(100)
      .checked_div(total_pairs)
      .map_or(0, |v| v);
    u8::try_from(100_u32.saturating_sub(contradiction_ratio as u32)).map_or(0, |v| v)
  } else {
    100
  };

  if contradictions > 0 {
    issues.push(QualityIssue::new(
      QualityDimension::Consistency,
      IssueSeverity::Warning,
      format!("Found {contradictions} potential contradictions in requirements"),
    ));
  }

  DimensionScore::new(QualityDimension::Consistency, score).map_or_else(|_| DimensionScore {
    dimension: QualityDimension::Consistency,
    score: 0,
  }, |v| v)
}

/// Check if two statements contradict each other
fn has_contradiction(val1: &str, val2: &str) -> bool {
  let contradictions = [
    ("must", "must not"),
    ("required", "optional"),
    ("always", "never"),
    ("enabled", "disabled"),
    ("allow", "deny"),
    ("include", "exclude"),
  ];

  contradictions.iter().any(|(pos, neg)| {
    (val1.contains(pos) && val2.contains(neg)) || (val1.contains(neg) && val2.contains(pos))
  })
}

/// Calculate testability: % of EARS with acceptance criteria
fn calculate_testability(
  ears: &[EarsRequirementRef],
  issues: &mut Vec<QualityIssue>,
) -> DimensionScore {
  if ears.is_empty() {
    issues.push(QualityIssue::new(
      QualityDimension::Testability,
      IssueSeverity::Error,
      "No EARS requirements defined".to_string(),
    ));
    return DimensionScore::new(QualityDimension::Testability, 0).map_or_else(|_| {
      DimensionScore {
        dimension: QualityDimension::Testability,
        score: 0,
      }
    }, |v| v);
  }

  let with_criteria = ears.iter().filter(|e| e.has_acceptance_criteria).count();

  let score = u8::try_from((with_criteria * 100) / ears.len()).map_or(100, |v| v);

  let without = ears.len() - with_criteria;
  if without > 0 {
    issues.push(QualityIssue::new(
      QualityDimension::Testability,
      IssueSeverity::Warning,
      format!("{without} requirement(s) missing acceptance criteria"),
    ));
  }

  DimensionScore::new(QualityDimension::Testability, score).map_or_else(|_| DimensionScore {
    dimension: QualityDimension::Testability,
    score: 0,
  }, |v| v)
}

/// Calculate clarity: sentence complexity and jargon density
fn calculate_clarity(answers: &[Answer], issues: &mut Vec<QualityIssue>) -> DimensionScore {
  let jargon_terms = [
    "microservice",
    "kubernetes",
    "orchestration",
    "containerization",
    "blockchain",
    "ai/ml",
    "serverless",
    "event-driven",
  ];

  let (total_sentences, complex_sentences, jargon_count) = answers.iter().fold(
    (0usize, 0usize, 0usize),
    |(total_sentences, complex_sentences, jargon_count), answer| {
      let text = &answer.value;

      // Count sentences (rough heuristic by period/exclamation count)
      let sentence_count = text.matches(&['.', '!', '?'][..]).count().max(1);

      // Complex sentence: more than 3 commas or 30 words
      let comma_count = text.matches(',').count();
      let word_count = text.split_whitespace().count();

      // Count jargon terms
      let lower = text.to_lowercase();
      let jargon_hits = jargon_terms
        .iter()
        .filter(|term| lower.contains(*term))
        .count();

      (
        total_sentences + sentence_count,
        complex_sentences
          + if comma_count > 3 || word_count > 30 {
            1
          } else {
            0
          },
        jargon_count + jargon_hits,
      )
    },
  );

  // Score = 100 - (complex_sentence_ratio + jargon_penalty)
  let complex_ratio = if total_sentences > 0 {
    complex_sentences
      .saturating_mul(100)
      .checked_div(total_sentences)
      .map_or(0, |v| v)
  } else {
    0
  };

  let jargon_penalty = (jargon_count * 5).min(50);

  let score = u8::try_from(
    100_u32
      .saturating_sub(complex_ratio as u32)
      .saturating_sub(jargon_penalty as u32),
  )
  .map_or(0, |v| v);

  if complex_sentences > 0 {
    issues.push(QualityIssue::new(
      QualityDimension::Clarity,
      IssueSeverity::Warning,
      format!("{complex_sentences} complex sentence(s) detected (consider simplifying)"),
    ));
  }

  if jargon_count > 2 {
    issues.push(QualityIssue::new(
      QualityDimension::Clarity,
      IssueSeverity::Warning,
      format!("High jargon density ({jargon_count} terms) - consider explaining terminology"),
    ));
  }

  DimensionScore::new(QualityDimension::Clarity, score).map_or_else(|_| DimensionScore {
    dimension: QualityDimension::Clarity,
    score: 0,
  }, |v| v)
}

/// Calculate security: auth/encryption/validation mentions
fn calculate_security(answers: &[Answer], issues: &mut Vec<QualityIssue>) -> DimensionScore {
  let security_keywords = [
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

  let matching_keywords: Vec<&str> = answers
    .iter()
    .flat_map(|answer| {
      let lower = answer.value.to_lowercase();
      security_keywords
        .iter()
        .filter_map(move |keyword| lower.contains(*keyword).then_some(*keyword))
    })
    .collect();

  let mentions = matching_keywords.len();
  let covered_areas: HashSet<&'static str> = matching_keywords
    .iter()
    .flat_map(|keyword| {
      let mut areas = Vec::new();
      if keyword.contains("auth") || keyword.contains("login") || keyword.contains("password") {
        areas.push("authentication");
      }
      if keyword.contains("encrypt") || keyword.contains("tls") || keyword.contains("ssl") {
        areas.push("encryption");
      }
      if keyword.contains("validat") || keyword.contains("sanitiz") || keyword.contains("escape") {
        areas.push("validation");
      }
      areas
    })
    .collect();

  // Score based on coverage of security areas
  let coverage_score = covered_areas.len() * 30; // max 90
  let mention_bonus = mentions.min(5) * 2; // max 10
  let total = coverage_score + mention_bonus;

  let score = u8::try_from(total.min(100)).map_or(100, |v| v);

  if covered_areas.is_empty() {
    issues.push(QualityIssue::new(
      QualityDimension::Security,
      IssueSeverity::Error,
      "No security considerations mentioned".to_string(),
    ));
  } else if covered_areas.len() < 3 {
    let missing = ["authentication", "encryption", "validation"]
      .iter()
      .filter(|area| !covered_areas.contains(*area))
      .join(", ");

    issues.push(QualityIssue::new(
      QualityDimension::Security,
      IssueSeverity::Warning,
      format!("Security considerations incomplete: missing {missing}"),
    ));
  }

  DimensionScore::new(QualityDimension::Security, score).map_or_else(|_| DimensionScore {
    dimension: QualityDimension::Security,
    score: 0,
  }, |v| v)
}

#[cfg(test)]
#[allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::unnecessary_debug_formatting,
  clippy::match_same_arms,
  clippy::option_if_let_else,
  clippy::suspicious_else_formatting
)]
mod tests {

  use super::*;

  fn create_answer(step_id: &str, value: &str) -> Answer {
    Answer {
      step_id: step_id.to_string(),
      value: value.to_string(),
      timestamp: "2024-01-01T00:00:00Z".to_string(),
    }
  }

  fn create_ears(id: &str, text: &str, has_criteria: bool) -> EarsRequirementRef {
    EarsRequirementRef {
      id: id.to_string(),
      text: text.to_string(),
      has_acceptance_criteria: has_criteria,
    }
  }

  #[test]
  fn test_dimension_score_valid_range() {
    let score = DimensionScore::new(QualityDimension::Completeness, 75);
    assert!(score.is_ok());
    if let Ok(s) = score {
      assert_eq!(s.score, 75);
    } else {
      panic!("Expected Ok");
    }
  }

  #[test]
  fn test_dimension_score_invalid_too_high() {
    let score = DimensionScore::new(QualityDimension::Completeness, 101);
    assert!(matches!(score, Err(QualityError::InvalidScore(_))));
  }

  #[test]
  fn test_dimension_score_passes_threshold() {
    let score = DimensionScore::new(QualityDimension::Completeness, 80);
    let score = match score {
      Ok(s) => s,
      Err(_) => panic!("Expected valid score"),
    };
    assert!(score.passes(70));
    assert!(!score.passes(90));
  }

  #[test]
  fn test_quality_score_passes_threshold() {
    let dimensions = vec![
      match DimensionScore::new(QualityDimension::Completeness, 80) {
        Ok(s) => s,
        Err(_) => panic!("Expected valid score"),
      },
      match DimensionScore::new(QualityDimension::Consistency, 75) {
        Ok(s) => s,
        Err(_) => panic!("Expected valid score"),
      },
      match DimensionScore::new(QualityDimension::Testability, 70) {
        Ok(s) => s,
        Err(_) => panic!("Expected valid score"),
      },
      match DimensionScore::new(QualityDimension::Clarity, 85) {
        Ok(s) => s,
        Err(_) => panic!("Expected valid score"),
      },
      match DimensionScore::new(QualityDimension::Security, 90) {
        Ok(s) => s,
        Err(_) => panic!("Expected valid score"),
      },
    ];

    let score = match QualityScore::new(80, dimensions, vec![]) {
      Ok(s) => s,
      Err(_) => panic!("Expected valid score"),
    };
    assert!(score.passes(70));
    assert!(!score.passes(90));
  }

  #[test]
  fn test_quality_score_get_dimension() {
    let dimensions = vec![
      match DimensionScore::new(QualityDimension::Completeness, 80) {
        Ok(s) => s,
        Err(_) => panic!("Expected valid score"),
      },
      match DimensionScore::new(QualityDimension::Consistency, 75) {
        Ok(s) => s,
        Err(_) => panic!("Expected valid score"),
      },
    ];

    let score = match QualityScore::new(77, dimensions, vec![]) {
      Ok(s) => s,
      Err(_) => panic!("Expected valid score"),
    };

    let completeness = score.get_dimension(QualityDimension::Completeness);
    assert!(completeness.is_some());
    if let Some(c) = completeness {
      assert_eq!(c.score, 80);
    }

    let security = score.get_dimension(QualityDimension::Security);
    assert!(security.is_none());
  }

  #[test]
  fn test_quality_score_get_issues() {
    let issues = vec![
      QualityIssue::new(
        QualityDimension::Completeness,
        IssueSeverity::Error,
        "Missing field".to_string(),
      ),
      QualityIssue::new(
        QualityDimension::Consistency,
        IssueSeverity::Warning,
        "Contradiction".to_string(),
      ),
    ];

    let score = match QualityScore::new(50, vec![], issues.clone()) {
      Ok(s) => s,
      Err(_) => panic!("Expected valid score"),
    };

    let completeness_issues = score.get_issues(QualityDimension::Completeness);
    assert_eq!(completeness_issues.len(), 1);
    assert_eq!(completeness_issues[0].severity, IssueSeverity::Error);

    let consistency_issues = score.get_issues(QualityDimension::Consistency);
    assert_eq!(consistency_issues.len(), 1);

    let security_issues = score.get_issues(QualityDimension::Security);
    assert!(security_issues.is_empty());
  }

  #[test]
  fn test_calculate_quality_empty_answers() {
    let ears = vec![];
    let inversion = InversionControl {
      has_inversion_tests: false,
      inverted_count: 0,
    };

    let result = calculate_quality(&[], &ears, &inversion);
    assert!(matches!(result, Err(QualityError::EmptyAnswers)));
  }

  #[test]
  fn test_calculate_quality_perfect_scores() {
    let answers = vec![
      create_answer("user_goal", "User must authenticate"),
      create_answer("actors", "System admin"),
      create_answer("precondition", "User exists"),
      create_answer("outcome", "Access granted"),
      create_answer("acceptance_criteria", "Login within 2 seconds"),
      create_answer(
        "security",
        "System must use TLS encryption and validate all inputs",
      ),
    ];

    let ears = vec![
      create_ears("1", "User shall authenticate", true),
      create_ears("2", "System shall encrypt data", true),
    ];

    let inversion = InversionControl {
      has_inversion_tests: true,
      inverted_count: 2,
    };

    let result = calculate_quality(&answers, &ears, &inversion);
    assert!(result.is_ok());

    let score = match result {
      Ok(s) => s,
      Err(_) => panic!("Expected Ok result"),
    };
    assert_eq!(score.overall, 100); // All perfect scores

    // Check no critical issues
    let critical = score
      .issues
      .iter()
      .filter(|i| i.severity == IssueSeverity::Critical);
    assert_eq!(critical.count(), 0);
  }

  #[test]
  fn test_calculate_completeness_missing_fields() {
    let answers = vec![
      create_answer("user_goal", "Goal"),
      // Missing actors, precondition, outcome, acceptance_criteria
    ];

    let mut issues = vec![];
    let score = calculate_completeness(&answers, &mut issues);

    // Should be 20% (1 out of 5 required fields)
    assert_eq!(score.score, 20);

    // Should have 4 issues (missing 4 fields)
    assert_eq!(issues.len(), 4);
    assert!(issues
      .iter()
      .all(|i| i.dimension == QualityDimension::Completeness));
  }

  #[test]
  fn test_calculate_completeness_all_fields() {
    let answers = vec![
      create_answer("user_goal", "Goal"),
      create_answer("actors", "Admin"),
      create_answer("precondition", "Precondition"),
      create_answer("outcome", "Success"),
      create_answer("acceptance_criteria", "Criteria"),
    ];

    let mut issues = vec![];
    let score = calculate_completeness(&answers, &mut issues);

    assert_eq!(score.score, 100);
    assert!(issues.is_empty());
  }

  #[test]
  fn test_calculate_consistency_contradictions() {
    let answers = vec![
      create_answer("req1", "User must authenticate"),
      create_answer("req2", "User must not authenticate"),
      create_answer("req3", "Data is required"),
    ];

    let mut issues = vec![];
    let score = calculate_consistency(&answers, &mut issues);

    // Should detect contradiction between "must" and "must not"
    assert!(score.score < 100);
    assert!(!issues.is_empty());
    assert!(issues[0].message.contains("contradiction"));
  }

  #[test]
  fn test_calculate_consistency_no_contradictions() {
    let answers = vec![
      create_answer("req1", "User must authenticate"),
      create_answer("req2", "Admin must authorize"),
    ];

    let mut issues = vec![];
    let score = calculate_consistency(&answers, &mut issues);

    assert_eq!(score.score, 100);
    assert!(issues.is_empty());
  }

  #[test]
  fn test_has_contradiction() {
    assert!(has_contradiction("must allow access", "must deny access"));
    assert!(has_contradiction("always enabled", "never enabled"));
    assert!(has_contradiction("required field", "optional field"));

    assert!(!has_contradiction(
      "must allow access",
      "should allow access"
    ));
    assert!(!has_contradiction("enabled feature", "enabled setting"));
  }

  #[test]
  fn test_calculate_testability_with_criteria() {
    let ears = vec![
      create_ears("1", "Req 1", true),
      create_ears("2", "Req 2", true),
      create_ears("3", "Req 3", false), // One without
    ];

    let mut issues = vec![];
    let score = calculate_testability(&ears, &mut issues);

    // 2 out of 3 = 66%
    assert_eq!(score.score, 66);

    // Should have 1 issue
    assert_eq!(issues.len(), 1);
    assert!(issues[0].message.contains("1"));
  }

  #[test]
  fn test_calculate_testability_no_ears() {
    let ears = vec![];
    let mut issues = vec![];
    let score = calculate_testability(&ears, &mut issues);

    assert_eq!(score.score, 0);
    assert_eq!(issues.len(), 1);
    assert!(issues[0].message.contains("No EARS"));
  }

  #[test]
  fn test_calculate_testability_all_with_criteria() {
    let ears = vec![
      create_ears("1", "Req 1", true),
      create_ears("2", "Req 2", true),
    ];

    let mut issues = vec![];
    let score = calculate_testability(&ears, &mut issues);

    assert_eq!(score.score, 100);
    assert!(issues.is_empty());
  }

  #[test]
  fn test_calculate_clarity_complex_sentences() {
    let answers = vec![
            create_answer(
                "req1",
                "The system shall, under normal operating conditions, provided that all prerequisites are met, and assuming no external interference, process the data.",
            ),
        ];

    let mut issues = vec![];
    let score = calculate_clarity(&answers, &mut issues);

    // Complex sentence should reduce score
    assert!(score.score < 100);
    assert!(!issues.is_empty());
    assert!(issues[0].message.contains("complex"));
  }

  #[test]
  fn test_calculate_clarity_jargon() {
    let answers = vec![
            create_answer(
                "req1",
                "Implement microservice architecture with Kubernetes orchestration and serverless event-driven blockchain integration.",
            ),
        ];

    let mut issues = vec![];
    let score = calculate_clarity(&answers, &mut issues);

    // High jargon should reduce score
    assert!(score.score < 100);

    // Should have jargon issue
    let jargon_issues: Vec<_> = issues
      .iter()
      .filter(|i| i.message.contains("jargon"))
      .collect();
    assert!(!jargon_issues.is_empty());
  }

  #[test]
  fn test_calculate_clarity_perfect() {
    let answers = vec![
      create_answer("req1", "Users must log in."),
      create_answer("req2", "Data is saved securely."),
    ];

    let mut issues = vec![];
    let score = calculate_clarity(&answers, &mut issues);

    assert_eq!(score.score, 100);
    assert!(issues.is_empty());
  }

  #[test]
  fn test_calculate_security_no_mentions() {
    let answers = vec![
      create_answer("req1", "Process data"),
      create_answer("req2", "Save results"),
    ];

    let mut issues = vec![];
    let score = calculate_security(&answers, &mut issues);

    assert_eq!(score.score, 0);
    assert_eq!(issues.len(), 1);
    assert!(issues[0].message.contains("No security"));
  }

  #[test]
  fn test_calculate_security_partial_coverage() {
    let answers = vec![create_answer(
      "req1",
      "Users must authenticate with password",
    )];

    let mut issues = vec![];
    let score = calculate_security(&answers, &mut issues);

    // Should have some coverage (authentication)
    assert!(score.score > 0);
    assert!(score.score < 100);

    // Should have warning about missing areas
    assert!(!issues.is_empty());
  }

  #[test]
  fn test_calculate_security_full_coverage() {
    let answers = vec![create_answer(
      "req1",
      "Users authenticate with password. Data encrypted with TLS. Inputs validated and sanitized.",
    )];

    let mut issues = vec![];
    let score = calculate_security(&answers, &mut issues);

    // Should have high coverage
    assert!(score.score >= 90);
    assert!(issues.is_empty());
  }

  #[test]
  fn test_quality_dimension_labels() {
    assert_eq!(QualityDimension::Completeness.label(), "Completeness");
    assert_eq!(QualityDimension::Consistency.label(), "Consistency");
    assert_eq!(QualityDimension::Testability.label(), "Testability");
    assert_eq!(QualityDimension::Clarity.label(), "Clarity");
    assert_eq!(QualityDimension::Security.label(), "Security");
  }

  #[test]
  fn test_quality_dimension_descriptions() {
    for dim in QualityDimension::all() {
      let desc = dim.description();
      assert!(!desc.is_empty());
    }
  }

  #[test]
  fn test_quality_dimension_all() {
    let all = QualityDimension::all();
    assert_eq!(all.len(), 5);
    assert!(all.contains(&QualityDimension::Completeness));
    assert!(all.contains(&QualityDimension::Consistency));
    assert!(all.contains(&QualityDimension::Testability));
    assert!(all.contains(&QualityDimension::Clarity));
    assert!(all.contains(&QualityDimension::Security));
  }

  #[test]
  fn test_overall_score_calculation() {
    let answers = vec![
      create_answer("user_goal", "Goal"),
      create_answer("actors", "Admin"),
      create_answer("precondition", "Precondition"),
      create_answer("outcome", "Success"),
      create_answer("acceptance_criteria", "Criteria"),
    ];

    let ears = vec![create_ears("1", "Req 1", true)];

    let inversion = InversionControl {
      has_inversion_tests: false,
      inverted_count: 0,
    };

    let result = calculate_quality(&answers, &ears, &inversion);
    assert!(result.is_ok());

    let score = match result {
      Ok(s) => s,
      Err(_) => panic!("Expected Ok result"),
    };

    // With 100% completeness, 100% consistency (single answer), 100% testability,
    // 100% clarity, and 0% security = 80% average
    assert_eq!(score.overall, 80);

    // Should have security issue
    let security_issues = score.get_issues(QualityDimension::Security);
    assert!(!security_issues.is_empty());
  }

  #[test]
  fn test_issue_severity_variants() {
    let error = QualityIssue::new(
      QualityDimension::Completeness,
      IssueSeverity::Error,
      "Error message".to_string(),
    );

    let warning = QualityIssue::new(
      QualityDimension::Consistency,
      IssueSeverity::Warning,
      "Warning message".to_string(),
    );

    let critical = QualityIssue::new(
      QualityDimension::Security,
      IssueSeverity::Critical,
      "Critical message".to_string(),
    );

    assert_eq!(error.severity, IssueSeverity::Error);
    assert_eq!(warning.severity, IssueSeverity::Warning);
    assert_eq!(critical.severity, IssueSeverity::Critical);
  }

  // =========================================================================
  // MUTANT CATCHING TESTS
  // These tests specifically target mutations that cargo-mutants found missed
  // =========================================================================

  #[test]
  fn test_quality_dimension_description_exact_content() {
    // Catches mutation: description() returning "xyzzy" (line 74)
    // Each dimension must return its exact expected description
    assert_eq!(
      QualityDimension::Completeness.description(),
      "Percentage of required fields filled"
    );
    assert_eq!(
      QualityDimension::Consistency.description(),
      "Absence of contradictory requirements"
    );
    assert_eq!(
      QualityDimension::Testability.description(),
      "Presence of acceptance criteria"
    );
    assert_eq!(
      QualityDimension::Clarity.description(),
      "Readability and minimal jargon"
    );
    assert_eq!(
      QualityDimension::Security.description(),
      "Security considerations present"
    );
  }

  #[test]
  fn test_completeness_exact_boundary_all_five_fields() {
    // Catches mutation: `>` to `>=` at line 270 in calculate_completeness
    // With exactly 5 required fields, all 5 filled should equal 100%
    let answers = vec![
      create_answer("user_goal", "User logs in"),
      create_answer("actors", "Admin"),
      create_answer("precondition", "User exists"),
      create_answer("outcome", "Access granted"),
      create_answer("acceptance_criteria", "Login works"),
    ];

    let mut issues = vec![];
    let score = calculate_completeness(&answers, &mut issues);

    // 5/5 = 100%, but the `>` mutation would make it 0%
    assert_eq!(score.score, 100, "All 5 fields filled should equal 100%");
    assert!(issues.is_empty(), "No issues when all fields present");
  }

  #[test]
  fn test_completeness_partial_calculation() {
    // Catches mutations in calculation math at line 270
    // 3/5 fields should equal 60%
    let answers = vec![
      create_answer("user_goal", "Goal"),
      create_answer("actors", "Actor"),
      create_answer("precondition", "Precond"),
      // Missing outcome and acceptance_criteria
    ];

    let mut issues = vec![];
    let score = calculate_completeness(&answers, &mut issues);

    // (3 * 100) / 5 = 60
    // If `+` mutates to `*`, this would be (3 + 100) * 5 = 515 (clamped to 100)
    // If `/` mutates to `*`, this would be (3 * 100) * 5 = 1500 (clamped to 100)
    assert_eq!(score.score, 60, "3 of 5 fields should equal 60%");
    assert_eq!(issues.len(), 2, "Should have 2 missing field issues");
  }

  #[test]
  fn test_consistency_exact_contradiction_count() {
    // Catches mutations: `+` to `*` at line 291, `/` to `*` at line 301
    // Test with exactly 1 contradiction among 3 answers (3 pairs)
    let answers = vec![
      create_answer("req1", "must allow"),
      create_answer("req2", "must not allow"),
      create_answer("req3", "optional field"),
    ];

    let mut issues = vec![];
    let score = calculate_consistency(&answers, &mut issues);

    // 1 contradiction in 2 pairs = (1 * 100) / 2 = 50% penalty, so score = 50
    // If `+` mutates to `*`, contradiction count becomes product (wrong)
    // If `/` mutates to `*`, ratio becomes product (wrong)
    assert_eq!(
      score.score, 50,
      "1 contradiction in 2 pairs should give 50%"
    );
    assert_eq!(issues.len(), 1);
    assert!(issues[0].message.contains("1"));
  }

  #[test]
  fn test_consistency_multiple_contradictions() {
    // Catches math mutations in contradiction counting/scoring
    let answers = vec![
      create_answer("req1", "must allow"),
      create_answer("req2", "must not allow"),
      create_answer("req3", "always enabled"),
      create_answer("req4", "never enabled"),
    ];

    let mut issues = vec![];
    let score = calculate_consistency(&answers, &mut issues);

    // 4 answers = 3 total_pairs (len - 1)
    // 2 contradictions found: (must/must not) and (always/never)
    // ratio = (2 * 100) / 3 = 66
    // score = 100 - 66 = 34
    // If `+` mutates to `*`, contradictions become product (wrong!)
    // If `/` mutates to `*`, ratio becomes product (wrong!)
    assert_eq!(
      score.score, 34,
      "2 contradictions in 3 pairs should give score 34"
    );
    assert_eq!(issues.len(), 1);
  }

  #[test]
  fn test_clarity_exact_boundary_complex_sentence() {
    // Catches mutations: `>` to `>=` at line 403
    // Exactly 4 commas should trigger complexity (threshold is > 3)
    let answers = vec![create_answer(
      "req1",
      "The system shall, process data, validate input, save results, exit",
    )];

    let mut issues = vec![];
    let score = calculate_clarity(&answers, &mut issues);

    // 4 commas > 3, so should be complex
    // If `>` mutates to `>=`, then 4 commas >= 3 would still trigger (same behavior)
    // But we want to catch the boundary case
    assert!(score.score < 100, "4 commas should reduce clarity score");
    assert!(!issues.is_empty(), "Should have complexity issue");
  }

  #[test]
  fn test_clarity_boundary_exactly_three_commas() {
    // Catches mutation: `>` to `>=` at line 403
    // Exactly 3 commas should NOT trigger complexity (threshold is > 3)
    let answers = vec![create_answer(
      "req1",
      "The system shall, process data, validate input, save results",
    )];

    let mut issues = vec![];
    let score = calculate_clarity(&answers, &mut issues);

    // 3 commas is not > 3, so should NOT be complex
    // If `>` mutates to `>=`, then 3 commas >= 3 would trigger (wrong!)
    assert_eq!(
      score.score, 100,
      "Exactly 3 commas should NOT reduce score (threshold is > 3)"
    );

    // Check no complexity issue
    let complex_issues: Vec<_> = issues
      .iter()
      .filter(|i| i.message.contains("complex"))
      .collect();
    assert!(
      complex_issues.is_empty(),
      "3 commas should not create complexity issue"
    );
  }

  #[test]
  fn test_clarity_boundary_exactly_thirty_words() {
    // Catches mutation: `>` to `>=` at line 403
    // Exactly 30 words should NOT trigger complexity (threshold is > 30)
    let text = "word ".repeat(15); // 15 words
    let answers = vec![create_answer("req1", &format!("{} {}", text, text.trim()))]; // 30 words

    let mut issues = vec![];
    let score = calculate_clarity(&answers, &mut issues);

    // 30 words is not > 30, so should NOT be complex
    // If `>` mutates to `>=`, then 30 words >= 30 would trigger (wrong!)
    assert_eq!(
      score.score, 100,
      "Exactly 30 words should NOT reduce score (threshold is > 30)"
    );
  }

  #[test]
  fn test_clarity_complex_ratio_calculation() {
    // Catches mutations: `/` to `*` at line 416, `*` to `/` at line 417
    // Test specific sentence/complex ratio to catch math mutations
    let answers = vec![
      create_answer("req1", "Simple."), // 1 sentence, 0 complex
      create_answer("req2", "Simple."), // 1 sentence, 0 complex
      create_answer(
        "req3",
        "The system shall, process data, validate input, save results, and exit.",
      ), // 1 sentence, 1 complex (5 commas > 3)
    ];

    let mut issues = vec![];
    let score = calculate_clarity(&answers, &mut issues);

    // 3 total sentences, 1 complex = (1 * 100) / 3 = 33
    // Score = 100 - 33 = 67
    // If `/` mutates to `*`, ratio becomes (1 * 100) * 3 = 300 (wrong!)
    // If `*` mutates to `/`, ratio becomes (1 / 100) / 3 = 0 (wrong!)
    assert_eq!(score.score, 67, "1 complex in 3 sentences should give ~67%");
  }

  #[test]
  fn test_clarity_jargon_penalty_calculation() {
    // Catches mutation: `*` to `/` at line 422
    // Test specific jargon count to catch math mutation
    let answers = vec![create_answer(
      "req1",
      "microservice kubernetes blockchain serverless",
    )];

    let mut issues = vec![];
    let score = calculate_clarity(&answers, &mut issues);

    // 4 jargon terms = 4 * 5 = 20 penalty
    // Score = 100 - 20 = 80
    // If `*` mutates to `/`, penalty becomes 4 / 5 = 0 (wrong!)
    assert_eq!(score.score, 80, "4 jargon terms should give 20 penalty");
  }

  #[test]
  fn test_clarity_jargon_threshold() {
    // Catches mutation: `>` to `>=` at line 439
    // Exactly 2 jargon terms should NOT trigger warning (threshold is > 2)
    let answers = vec![create_answer("req1", "microservice kubernetes")];

    let mut issues = vec![];
    calculate_clarity(&answers, &mut issues);

    // 2 jargon terms is not > 2, so should NOT warn
    // If `>` mutates to `>=`, then 2 >= 2 would trigger (wrong!)
    let jargon_issues: Vec<_> = issues
      .iter()
      .filter(|i| i.message.contains("jargon"))
      .collect();
    assert!(
      jargon_issues.is_empty(),
      "Exactly 2 jargon terms should NOT trigger warning (threshold is > 2)"
    );
  }

  #[test]
  fn test_clarity_three_jargon_terms_triggers_warning() {
    // Catches mutation: `>` to `>=` at line 439
    // Exactly 3 jargon terms SHOULD trigger warning (threshold is > 2)
    let answers = vec![create_answer("req1", "microservice kubernetes blockchain")];

    let mut issues = vec![];
    calculate_clarity(&answers, &mut issues);

    // 3 jargon terms is > 2, so should warn
    // If `>` mutates to `>=`, then 3 >= 2 would still trigger (same)
    let jargon_issues: Vec<_> = issues
      .iter()
      .filter(|i| i.message.contains("jargon"))
      .collect();
    assert_eq!(
      jargon_issues.len(),
      1,
      "3 jargon terms should trigger warning"
    );
  }

  #[test]
  fn test_security_auth_keyword_detection() {
    // Catches mutation: `||` to `&&` at line 487
    // "auth" keyword should be caught by contains("auth")
    let answers = vec![create_answer("req1", "Use auth for access")];

    let mut issues = vec![];
    let score = calculate_security(&answers, &mut issues);

    // "auth" contains "auth", should trigger authentication area
    // If `||` mutates to `&&`, all three conditions must be true (wrong!)
    assert!(score.score > 0, "auth keyword should give positive score");

    // Should not have "missing authentication" issue
    let missing_auth: Vec<_> = issues
      .iter()
      .filter(|i| i.message.contains("missing") && i.message.contains("authentication"))
      .collect();
    assert!(
      missing_auth.is_empty(),
      "auth keyword should cover authentication"
    );
  }

  #[test]
  fn test_security_login_keyword_detection() {
    // Catches mutation: `||` to `&&` at line 487
    // "login" keyword should be caught independently
    let answers = vec![create_answer("req1", "User login required")];

    let mut issues = vec![];
    let score = calculate_security(&answers, &mut issues);

    // "login" contains "login", should trigger authentication area
    // If `||` mutates to `&&`, keyword wouldn't match other conditions (wrong!)
    assert!(score.score > 0, "login keyword should give positive score");
  }

  #[test]
  fn test_security_password_keyword_detection() {
    // Catches mutation: `||` to `&&` at line 487
    // "password" keyword should be caught independently
    let answers = vec![create_answer("req1", "Enter password to continue")];

    let mut issues = vec![];
    let score = calculate_security(&answers, &mut issues);

    // "password" contains "password", should trigger authentication area
    assert!(
      score.score > 0,
      "password keyword should give positive score"
    );
  }

  #[test]
  fn test_security_encrypt_keyword_detection() {
    // Catches mutation: `||` to `&&` at line 490
    // "encrypt" keyword should trigger encryption area
    let answers = vec![create_answer("req1", "Data must encrypt")];

    let mut issues = vec![];
    let score = calculate_security(&answers, &mut issues);

    // "encrypt" contains "encrypt", should trigger encryption area
    // If `||` mutates to `&&`, all three conditions must be true (wrong!)
    assert!(
      score.score > 0,
      "encrypt keyword should give positive score"
    );
  }

  #[test]
  fn test_security_tls_keyword_detection() {
    // Catches mutation: `||` to `&&` at line 490
    // "tls" keyword should be caught independently
    let answers = vec![create_answer("req1", "Use tls for transport")];

    let mut issues = vec![];
    let score = calculate_security(&answers, &mut issues);

    // "tls" contains "tls", should trigger encryption area
    assert!(score.score > 0, "tls keyword should give positive score");
  }

  #[test]
  fn test_security_validate_keyword_detection() {
    // Catches mutation: `||` to `&&` at line 493
    // "validate" contains "validat", should trigger validation area
    let answers = vec![create_answer("req1", "Always validate user input")];

    let mut issues = vec![];
    let score = calculate_security(&answers, &mut issues);

    // "validate" contains "validat", should trigger validation area
    // If `||` mutates to `&&`, all three conditions must be true (wrong!)
    assert!(
      score.score > 0,
      "validate keyword should give positive score"
    );
  }

  #[test]
  fn test_security_sanitize_keyword_detection() {
    // Catches mutation: `||` to `&&` at line 493
    // "sanitize" contains "sanitiz", should trigger validation area
    let answers = vec![create_answer("req1", "Sanitize all inputs")];

    let mut issues = vec![];
    let score = calculate_security(&answers, &mut issues);

    // "sanitize" contains "sanitiz", should trigger validation area
    assert!(
      score.score > 0,
      "sanitize keyword should give positive score"
    );
  }

  #[test]
  fn test_security_escape_keyword_detection() {
    // Catches mutation: `||` to `&&` at line 493
    // "escape" should be caught independently
    let answers = vec![create_answer("req1", "Escape special characters")];

    let mut issues = vec![];
    let score = calculate_security(&answers, &mut issues);

    // "escape" should trigger validation area
    assert!(score.score > 0, "escape keyword should give positive score");
  }

  #[test]
  fn test_security_keyword_present_but_area_uncovered() {
    // Catches mutation: delete `!` at line 517
    // Test that missing area detection actually checks for absence
    // With only "authentication" covered, should warn about missing "encryption" and "validation"
    let answers = vec![create_answer(
      "req1",
      "Users must authenticate with password",
    )];

    let mut issues = vec![];
    calculate_security(&answers, &mut issues);

    // Should have warning about missing areas
    let incomplete_issues: Vec<_> = issues
      .iter()
      .filter(|i| i.message.contains("incomplete"))
      .collect();
    assert_eq!(
      incomplete_issues.len(),
      1,
      "Should warn about incomplete coverage"
    );
    assert!(
      incomplete_issues[0].message.contains("encryption")
        && incomplete_issues[0].message.contains("validation"),
      "Should mention missing encryption and validation"
    );
  }

  #[test]
  fn test_security_only_encryption_covered() {
    // Tests area coverage calculation logic
    let answers = vec![create_answer("req1", "Use TLS for encryption")];

    let mut issues = vec![];
    let score = calculate_security(&answers, &mut issues);

    // Only encryption covered (1 area) = 1 * 30 = 30 points
    // "tls" and "encrypt" both match = 2 mentions * 2 = 4 bonus
    // Total = 30 + 4 = 34
    assert_eq!(
      score.score, 34,
      "Only encryption should give 34% (30 coverage + 4 bonus)"
    );

    // Should warn about missing areas
    assert!(!issues.is_empty());
    assert!(issues[0].message.contains("incomplete"));
  }

  #[test]
  fn test_security_two_areas_covered() {
    // Tests area coverage calculation with 2/3 areas
    let answers = vec![create_answer(
      "req1",
      "Users authenticate with password and data uses TLS encryption",
    )];

    let mut issues = vec![];
    let score = calculate_security(&answers, &mut issues);

    // Authentication + encryption = 2 areas = 2 * 30 = 60 + mention bonus
    assert!(
      (60..=80).contains(&score.score),
      "Two areas should give 60-70% score"
    );
  }
}
