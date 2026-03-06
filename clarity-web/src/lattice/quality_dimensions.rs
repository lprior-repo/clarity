#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Quality Dimensions module for extended quality analysis.
//!
//! This module provides a comprehensive framework for evaluating requirements
//! across multiple quality dimensions beyond the basic 5-dimension model.

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Domain errors for quality dimensions
#[derive(Debug, Error, PartialEq, Clone)]
pub enum QualityDimensionError {
  #[error("dimension name is empty")]
  EmptyName,

  #[error("invalid score value: {0}")]
  InvalidScore(u8),

  #[error("dimension not found: {0}")]
  NotFound(String),

  #[error("weight must be between 0.0 and 1.0")]
  InvalidWeight,
}

/// Core quality dimensions (ISO 25010 based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreDimension {
  /// Functional completeness
  Completeness,
  /// Functional correctness
  Correctness,
  /// Functional appropriateness
  Appropriateness,
  /// Performance efficiency
  Performance,
  /// Compatibility with other systems
  Compatibility,
  /// Usability for end users
  Usability,
  /// Reliability and availability
  Reliability,
  /// Security and access control
  Security,
  /// Maintainability
  Maintainability,
  /// Transferability/portability
  Portability,
}

impl CoreDimension {
  /// Get all core dimensions
  #[must_use]
  pub const fn all() -> [Self; 10] {
    [
      Self::Completeness,
      Self::Correctness,
      Self::Appropriateness,
      Self::Performance,
      Self::Compatibility,
      Self::Usability,
      Self::Reliability,
      Self::Security,
      Self::Maintainability,
      Self::Portability,
    ]
  }

  /// Get human-readable label
  #[must_use]
  pub const fn label(&self) -> &'static str {
    match self {
      Self::Completeness => "Completeness",
      Self::Correctness => "Correctness",
      Self::Appropriateness => "Appropriateness",
      Self::Performance => "Performance",
      Self::Compatibility => "Compatibility",
      Self::Usability => "Usability",
      Self::Reliability => "Reliability",
      Self::Security => "Security",
      Self::Maintainability => "Maintainability",
      Self::Portability => "Portability",
    }
  }

  /// Get category group
  #[must_use]
  pub const fn category(&self) -> DimensionCategory {
    match self {
      Self::Completeness | Self::Correctness | Self::Appropriateness => {
        DimensionCategory::Functional
      }
      Self::Performance => DimensionCategory::Efficiency,
      Self::Compatibility => DimensionCategory::Compatibility,
      Self::Usability => DimensionCategory::Usability,
      Self::Reliability => DimensionCategory::Reliability,
      Self::Security => DimensionCategory::Security,
      Self::Maintainability | Self::Portability => DimensionCategory::Maintainability,
    }
  }

  /// Get default weight for scoring
  #[must_use]
  pub const fn default_weight(&self) -> f32 {
    match self {
      Self::Security | Self::Correctness => 1.5,
      Self::Reliability | Self::Performance => 1.2,
      Self::Completeness => 1.0,
      _ => 0.8,
    }
  }
}

/// Dimension categories for grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DimensionCategory {
  Functional,
  Efficiency,
  Compatibility,
  Usability,
  Reliability,
  Security,
  Maintainability,
}

impl DimensionCategory {
  /// Get all categories
  #[must_use]
  pub const fn all() -> [Self; 7] {
    [
      Self::Functional,
      Self::Efficiency,
      Self::Compatibility,
      Self::Usability,
      Self::Reliability,
      Self::Security,
      Self::Maintainability,
    ]
  }

  /// Get label
  #[must_use]
  pub const fn label(&self) -> &'static str {
    match self {
      Self::Functional => "Functional",
      Self::Efficiency => "Efficiency",
      Self::Compatibility => "Compatibility",
      Self::Usability => "Usability",
      Self::Reliability => "Reliability",
      Self::Security => "Security",
      Self::Maintainability => "Maintainability",
    }
  }
}

/// Score for a single dimension
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DimensionScore {
  /// The dimension scored
  pub dimension: CoreDimension,
  /// Score value (0-100)
  pub score: u8,
  /// Weight applied (0.0-2.0)
  pub weight: f32,
}

impl DimensionScore {
  /// Create a new dimension score
  ///
  /// # Errors
  ///
  /// Returns `QualityDimensionError::InvalidScore` if score > 100
  pub fn new(
    dimension: CoreDimension,
    score: u8,
    weight: f32,
  ) -> Result<Self, QualityDimensionError> {
    if score > 100 {
      return Err(QualityDimensionError::InvalidScore(score));
    }
    if weight < 0.0 || weight > 2.0 {
      return Err(QualityDimensionError::InvalidWeight);
    }

    Ok(Self {
      dimension,
      score,
      weight,
    })
  }

  /// Create with default weight
  ///
  /// # Errors
  ///
  /// Returns `QualityDimensionError::InvalidScore` if score > 100
  pub fn with_default_weight(
    dimension: CoreDimension,
    score: u8,
  ) -> Result<Self, QualityDimensionError> {
    Self::new(dimension, score, dimension.default_weight())
  }

  /// Calculate weighted score
  #[must_use]
  pub fn weighted_score(&self) -> f32 {
    f32::from(self.score) * self.weight
  }

  /// Check if score passes threshold
  #[must_use]
  pub fn passes(&self, threshold: u8) -> bool {
    self.score >= threshold
  }
}

/// Issue detected during dimension analysis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DimensionIssue {
  /// Related dimension
  pub dimension: CoreDimension,
  /// Issue severity
  pub severity: IssueSeverity,
  /// Issue description
  pub message: String,
  /// Location/element causing the issue
  pub location: Option<String>,
}

impl DimensionIssue {
  /// Create a new dimension issue
  #[must_use]
  pub fn new(dimension: CoreDimension, severity: IssueSeverity, message: String) -> Self {
    Self {
      dimension,
      severity,
      message,
      location: None,
    }
  }

  /// Add location using builder pattern
  #[must_use]
  pub fn with_location(mut self, location: String) -> Self {
    self.location = Some(location);
    self
  }
}

/// Issue severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum IssueSeverity {
  /// Minor suggestion
  Info,
  /// Potential issue
  Warning,
  /// Significant problem
  Error,
  /// Blocking issue
  Critical,
}

impl IssueSeverity {
  /// Convert to numeric impact
  #[must_use]
  pub const fn impact(&self) -> u8 {
    match self {
      Self::Info => 5,
      Self::Warning => 15,
      Self::Error => 30,
      Self::Critical => 50,
    }
  }
}

/// Complete quality dimension analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionAnalysis {
  /// Individual dimension scores
  pub scores: Vec<DimensionScore>,
  /// Detected issues
  pub issues: Vec<DimensionIssue>,
  /// Overall weighted score
  pub overall_score: u8,
  /// Scores grouped by category
  pub category_scores: HashMap<DimensionCategory, u8>,
  /// Summary message
  pub summary: String,
}

impl DimensionAnalysis {
  /// Create new dimension analysis
  #[must_use]
  pub fn new(scores: Vec<DimensionScore>, issues: Vec<DimensionIssue>) -> Self {
    let category_scores = calculate_category_scores(&scores);
    let overall_score = calculate_overall_score(&scores);
    let summary = generate_analysis_summary(&scores, &issues);

    Self {
      scores,
      issues,
      overall_score,
      category_scores,
      summary,
    }
  }

  /// Get score for a specific dimension
  #[must_use]
  pub fn get_score(&self, dimension: CoreDimension) -> Option<&DimensionScore> {
    self.scores.iter().find(|s| s.dimension == dimension)
  }

  /// Get issues for a specific dimension
  #[must_use]
  pub fn get_issues(&self, dimension: CoreDimension) -> Vec<&DimensionIssue> {
    self
      .issues
      .iter()
      .filter(|i| i.dimension == dimension)
      .collect()
  }

  /// Get issues by severity
  #[must_use]
  pub fn issues_by_severity(&self, severity: IssueSeverity) -> Vec<&DimensionIssue> {
    self
      .issues
      .iter()
      .filter(|i| i.severity == severity)
      .collect()
  }

  /// Get weakest dimensions (below threshold)
  #[must_use]
  pub fn weakest_dimensions(&self, threshold: u8) -> Vec<&DimensionScore> {
    self
      .scores
      .iter()
      .filter(|s| s.score < threshold)
      .sorted_by_key(|s| s.score)
      .collect()
  }

  /// Check if passes minimum quality gate
  #[must_use]
  pub fn passes_gate(&self, min_score: u8, max_critical: usize) -> bool {
    let critical_count = self
      .issues
      .iter()
      .filter(|i| i.severity == IssueSeverity::Critical)
      .count();

    self.overall_score >= min_score && critical_count <= max_critical
  }
}

/// Calculate category scores
fn calculate_category_scores(scores: &[DimensionScore]) -> HashMap<DimensionCategory, u8> {
  let mut result = HashMap::new();

  for category in DimensionCategory::all() {
    let category_scores: Vec<&DimensionScore> = scores
      .iter()
      .filter(|s| s.dimension.category() == category)
      .collect();

    if !category_scores.is_empty() {
      let total_weight: f32 = category_scores.iter().map(|s| s.weight).sum();
      let weighted_sum: f32 = category_scores.iter().map(|s| s.weighted_score()).sum();

      let avg = if total_weight > 0.0 {
        weighted_sum / total_weight
      } else {
        0.0
      };

      result.insert(category, avg.min(100.0) as u8);
    }
  }

  result
}

/// Calculate overall weighted score
fn calculate_overall_score(scores: &[DimensionScore]) -> u8 {
  if scores.is_empty() {
    return 0;
  }

  let total_weight: f32 = scores.iter().map(|s| s.weight).sum();
  let weighted_sum: f32 = scores.iter().map(|s| s.weighted_score()).sum();

  if total_weight > 0.0 {
    ((weighted_sum / total_weight).min(100.0)) as u8
  } else {
    0
  }
}

/// Generate analysis summary
fn generate_analysis_summary(scores: &[DimensionScore], issues: &[DimensionIssue]) -> String {
  let high_scores = scores.iter().filter(|s| s.score >= 80).count();
  let low_scores = scores.iter().filter(|s| s.score < 50).count();

  let critical_issues = issues
    .iter()
    .filter(|i| i.severity == IssueSeverity::Critical)
    .count();
  let errors = issues
    .iter()
    .filter(|i| i.severity == IssueSeverity::Error)
    .count();

  format!(
    "Quality Analysis: {high_scores} dimensions scored 80+, {low_scores} below 50. \
         Issues: {critical_issues} critical, {errors} errors."
  )
}

/// Analyze requirements across all quality dimensions
///
/// # Arguments
/// * `requirements` - List of requirement texts to analyze
///
/// # Returns
/// Complete dimension analysis with scores and issues
#[must_use]
pub fn analyze_dimensions(requirements: &[&str]) -> DimensionAnalysis {
  let mut scores = Vec::new();
  let mut issues = Vec::new();

  // Analyze each dimension
  for dimension in CoreDimension::all() {
    let (score, dimension_issues) = analyze_single_dimension(dimension, requirements);

    if let Ok(dim_score) = DimensionScore::with_default_weight(dimension, score) {
      scores.push(dim_score);
    }

    issues.extend(dimension_issues);
  }

  DimensionAnalysis::new(scores, issues)
}

/// Analyze a single dimension
fn analyze_single_dimension(
  dimension: CoreDimension,
  requirements: &[&str],
) -> (u8, Vec<DimensionIssue>) {
  let mut issues = Vec::new();

  let score = match dimension {
    CoreDimension::Completeness => analyze_completeness(requirements, &mut issues),
    CoreDimension::Correctness => analyze_correctness(requirements, &mut issues),
    CoreDimension::Appropriateness => analyze_appropriateness(requirements, &mut issues),
    CoreDimension::Performance => analyze_performance(requirements, &mut issues),
    CoreDimension::Compatibility => analyze_compatibility(requirements, &mut issues),
    CoreDimension::Usability => analyze_usability(requirements, &mut issues),
    CoreDimension::Reliability => analyze_reliability(requirements, &mut issues),
    CoreDimension::Security => analyze_security(requirements, &mut issues),
    CoreDimension::Maintainability => analyze_maintainability(requirements, &mut issues),
    CoreDimension::Portability => analyze_portability(requirements, &mut issues),
  };

  (score, issues)
}

/// Analyze completeness dimension
fn analyze_completeness(requirements: &[&str], issues: &mut Vec<DimensionIssue>) -> u8 {
  if requirements.is_empty() {
    issues.push(DimensionIssue::new(
      CoreDimension::Completeness,
      IssueSeverity::Critical,
      "No requirements provided".to_string(),
    ));
    return 0;
  }

  let required_elements: [(&str, [&str; 3]); 4] = [
    ("actor", ["user", "system", "actor"]),
    ("action", ["shall", "must", "will"]),
    ("condition", ["when", "if", "where"]),
    ("result", ["then", "output", "response"]),
  ];

  let mut filled_count = 0;
  let total = required_elements.len();

  for (name, keywords) in &required_elements {
    let has_element = requirements.iter().any(|r| {
      let lower = r.to_lowercase();
      keywords.iter().any(|k| lower.contains(k))
    });

    if has_element {
      filled_count += 1;
    } else {
      issues.push(DimensionIssue::new(
        CoreDimension::Completeness,
        IssueSeverity::Warning,
        format!("Missing {name} element in requirements"),
      ));
    }
  }

  ((filled_count * 100) / total) as u8
}

/// Analyze correctness dimension
fn analyze_correctness(requirements: &[&str], issues: &mut Vec<DimensionIssue>) -> u8 {
  let mut score: u8 = 100;

  // Check for vague language
  let vague_terms = ["etc", "and so on", "tbd", "todo", "sometime", "eventually"];

  for req in requirements {
    let lower = req.to_lowercase();
    for term in &vague_terms {
      if lower.contains(term) {
        score = score.saturating_sub(15);
        issues.push(
          DimensionIssue::new(
            CoreDimension::Correctness,
            IssueSeverity::Warning,
            format!("Vague term found: '{term}'"),
          )
          .with_location(req.to_string()),
        );
      }
    }
  }

  // Check for contradictory statements
  for (i, req1) in requirements.iter().enumerate() {
    for req2 in requirements.iter().skip(i + 1) {
      if has_contradiction(req1, req2) {
        score = score.saturating_sub(25);
        issues.push(DimensionIssue::new(
          CoreDimension::Correctness,
          IssueSeverity::Error,
          "Potential contradiction detected".to_string(),
        ));
      }
    }
  }

  score
}

/// Check for contradiction between two requirements
fn has_contradiction(req1: &str, req2: &str) -> bool {
  let lower1 = req1.to_lowercase();
  let lower2 = req2.to_lowercase();

  let contradictions = [
    ("must", "must not"),
    ("always", "never"),
    ("enabled", "disabled"),
    ("required", "optional"),
  ];

  contradictions.iter().any(|(pos, neg)| {
    (lower1.contains(pos) && lower2.contains(neg)) || (lower1.contains(neg) && lower2.contains(pos))
  })
}

/// Analyze appropriateness dimension
fn analyze_appropriateness(requirements: &[&str], issues: &mut Vec<DimensionIssue>) -> u8 {
  let mut score: u8 = 100;

  // Check for technical jargon without context
  let jargon = [
    "microservice",
    "kubernetes",
    "blockchain",
    "ai/ml",
    "serverless",
  ];

  for req in requirements {
    let lower = req.to_lowercase();
    for term in &jargon {
      if lower.contains(term) && !lower.contains("for example") && !lower.contains("such as") {
        score = score.saturating_sub(10);
        issues.push(
          DimensionIssue::new(
            CoreDimension::Appropriateness,
            IssueSeverity::Info,
            format!("Technical term '{term}' may need context"),
          )
          .with_location(req.to_string()),
        );
      }
    }
  }

  score
}

/// Analyze performance dimension
fn analyze_performance(requirements: &[&str], issues: &mut Vec<DimensionIssue>) -> u8 {
  let mut score: u8 = 100;

  // Check for performance requirements
  let perf_keywords = [
    "response time",
    "latency",
    "throughput",
    "performance",
    "within",
    "seconds",
    "milliseconds",
  ];

  let has_perf_req = requirements.iter().any(|r| {
    let lower = r.to_lowercase();
    perf_keywords.iter().any(|k| lower.contains(k))
  });

  if !has_perf_req {
    score = 50;
    issues.push(DimensionIssue::new(
      CoreDimension::Performance,
      IssueSeverity::Warning,
      "No explicit performance requirements found".to_string(),
    ));
  }

  score
}

/// Analyze compatibility dimension
fn analyze_compatibility(requirements: &[&str], issues: &mut Vec<DimensionIssue>) -> u8 {
  let mut score: u8 = 100;

  let compat_keywords = [
    "compatible",
    "integration",
    "api",
    "interface",
    "protocol",
    "version",
  ];

  let has_compat_req = requirements.iter().any(|r| {
    let lower = r.to_lowercase();
    compat_keywords.iter().any(|k| lower.contains(k))
  });

  if !has_compat_req {
    score = 60;
    issues.push(DimensionIssue::new(
      CoreDimension::Compatibility,
      IssueSeverity::Info,
      "No compatibility requirements specified".to_string(),
    ));
  }

  score
}

/// Analyze usability dimension
fn analyze_usability(requirements: &[&str], issues: &mut Vec<DimensionIssue>) -> u8 {
  let mut score: u8 = 100;

  let usability_keywords = [
    "user interface",
    "ux",
    "accessibility",
    "usability",
    "user experience",
    "ui",
  ];

  let has_usability_req = requirements.iter().any(|r| {
    let lower = r.to_lowercase();
    usability_keywords.iter().any(|k| lower.contains(k))
  });

  if !has_usability_req {
    score = 70;
    issues.push(DimensionIssue::new(
      CoreDimension::Usability,
      IssueSeverity::Info,
      "No usability requirements specified".to_string(),
    ));
  }

  score
}

/// Analyze reliability dimension
fn analyze_reliability(requirements: &[&str], issues: &mut Vec<DimensionIssue>) -> u8 {
  let mut score: u8 = 100;

  let reliability_keywords = [
    "availability",
    "uptime",
    "reliability",
    "fault",
    "recovery",
    "backup",
    "redundancy",
  ];

  let has_reliability_req = requirements.iter().any(|r| {
    let lower = r.to_lowercase();
    reliability_keywords.iter().any(|k| lower.contains(k))
  });

  if !has_reliability_req {
    score = 50;
    issues.push(DimensionIssue::new(
      CoreDimension::Reliability,
      IssueSeverity::Warning,
      "No reliability requirements specified".to_string(),
    ));
  }

  score
}

/// Analyze security dimension
fn analyze_security(requirements: &[&str], issues: &mut Vec<DimensionIssue>) -> u8 {
  let mut score: u8 = 100;

  let security_keywords = [
    "authentication",
    "authorization",
    "encryption",
    "security",
    "access control",
    "validate",
    "sanitize",
  ];

  let found_keywords: Vec<&str> = security_keywords
    .iter()
    .filter(|k| requirements.iter().any(|r| r.to_lowercase().contains(*k)))
    .copied()
    .collect();

  if found_keywords.is_empty() {
    score = 0;
    issues.push(DimensionIssue::new(
      CoreDimension::Security,
      IssueSeverity::Critical,
      "No security requirements specified".to_string(),
    ));
  } else if found_keywords.len() < 3 {
    score = 60;
    issues.push(DimensionIssue::new(
      CoreDimension::Security,
      IssueSeverity::Error,
      "Security requirements incomplete".to_string(),
    ));
  }

  score
}

/// Analyze maintainability dimension
fn analyze_maintainability(requirements: &[&str], issues: &mut Vec<DimensionIssue>) -> u8 {
  let mut score: u8 = 100;

  // Check for complexity
  for req in requirements {
    let words = req.split_whitespace().count();
    if words > 50 {
      score = score.saturating_sub(20);
      issues.push(
        DimensionIssue::new(
          CoreDimension::Maintainability,
          IssueSeverity::Warning,
          format!("Requirement too complex ({words} words)"),
        )
        .with_location(req.to_string()),
      );
    }
  }

  score
}

/// Analyze portability dimension
fn analyze_portability(requirements: &[&str], issues: &mut Vec<DimensionIssue>) -> u8 {
  let mut score: u8 = 100;

  let portability_keywords = [
    "platform",
    "portable",
    "cross-platform",
    "environment",
    "deployment",
  ];

  let has_portability_req = requirements.iter().any(|r| {
    let lower = r.to_lowercase();
    portability_keywords.iter().any(|k| lower.contains(k))
  });

  if !has_portability_req {
    score = 70;
    issues.push(DimensionIssue::new(
      CoreDimension::Portability,
      IssueSeverity::Info,
      "No portability requirements specified".to_string(),
    ));
  }

  score
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {
  use super::*;

  #[test]
  fn test_core_dimension_labels() {
    assert_eq!(CoreDimension::Completeness.label(), "Completeness");
    assert_eq!(CoreDimension::Security.label(), "Security");
  }

  #[test]
  fn test_core_dimension_categories() {
    assert_eq!(
      CoreDimension::Completeness.category(),
      DimensionCategory::Functional
    );
    assert_eq!(
      CoreDimension::Security.category(),
      DimensionCategory::Security
    );
  }

  #[test]
  fn test_core_dimension_default_weights() {
    assert!(CoreDimension::Security.default_weight() > 1.0);
    assert!(CoreDimension::Completeness.default_weight() >= 1.0);
  }

  #[test]
  fn test_dimension_score_new_valid() {
    let score = DimensionScore::new(CoreDimension::Completeness, 75, 1.0);
    assert!(score.is_ok());
  }

  #[test]
  fn test_dimension_score_invalid_score() {
    let result = DimensionScore::new(CoreDimension::Completeness, 101, 1.0);
    assert!(matches!(
      result,
      Err(QualityDimensionError::InvalidScore(101))
    ));
  }

  #[test]
  fn test_dimension_score_invalid_weight() {
    let result = DimensionScore::new(CoreDimension::Completeness, 50, 3.0);
    assert!(matches!(result, Err(QualityDimensionError::InvalidWeight)));
  }

  #[test]
  fn test_dimension_score_weighted_calculation() {
    let score = DimensionScore::new(CoreDimension::Completeness, 80, 1.5).unwrap();
    assert!((score.weighted_score() - 120.0).abs() < f32::EPSILON);
  }

  #[test]
  fn test_dimension_score_passes_threshold() {
    let score = DimensionScore::new(CoreDimension::Completeness, 75, 1.0).unwrap();
    assert!(score.passes(70));
    assert!(!score.passes(80));
  }

  #[test]
  fn test_dimension_issue_builder() {
    let issue = DimensionIssue::new(
      CoreDimension::Security,
      IssueSeverity::Critical,
      "Test issue".to_string(),
    )
    .with_location("req-001".to_string());

    assert_eq!(issue.location, Some("req-001".to_string()));
  }

  #[test]
  fn test_issue_severity_ordering() {
    assert!(IssueSeverity::Critical > IssueSeverity::Error);
    assert!(IssueSeverity::Error > IssueSeverity::Warning);
    assert!(IssueSeverity::Warning > IssueSeverity::Info);
  }

  #[test]
  fn test_issue_severity_impact() {
    assert!(IssueSeverity::Critical.impact() > IssueSeverity::Error.impact());
  }

  #[test]
  fn test_analyze_dimensions_empty() {
    let analysis = analyze_dimensions(&[]);

    assert!(analysis
      .issues
      .iter()
      .any(|i| i.severity == IssueSeverity::Critical));
  }

  #[test]
  fn test_analyze_dimensions_basic() {
    let requirements = vec![
      "The system shall authenticate users before access.",
      "User must be authorized to view data.",
      "Response time shall be within 2 seconds.",
    ];

    let analysis = analyze_dimensions(&requirements);

    assert!(!analysis.scores.is_empty());
    assert!(analysis.overall_score > 0);
  }

  #[test]
  fn test_analyze_completeness_with_all_elements() {
    let requirements =
      vec!["When the user requests data, the system shall return the response within 2 seconds."];

    let mut issues = Vec::new();
    let score = analyze_completeness(&requirements, &mut issues);

    assert!(score > 50);
  }

  #[test]
  fn test_analyze_completeness_missing_elements() {
    let requirements = vec!["Some vague text."];

    let mut issues = Vec::new();
    let score = analyze_completeness(&requirements, &mut issues);

    assert!(score < 100);
    assert!(!issues.is_empty());
  }

  #[test]
  fn test_analyze_correctness_vague_terms() {
    let requirements = vec!["The system shall do etc and so on."];

    let mut issues = Vec::new();
    let score = analyze_correctness(&requirements, &mut issues);

    assert!(score < 100);
    assert!(!issues.is_empty());
  }

  #[test]
  fn test_analyze_correctness_contradiction() {
    let requirements = vec![
      "The system must always be available.",
      "The system must never be available.",
    ];

    let mut issues = Vec::new();
    let score = analyze_correctness(&requirements, &mut issues);

    assert!(score < 100);
  }

  #[test]
  fn test_analyze_security_no_requirements() {
    let requirements = vec!["The system shall process data."];

    let mut issues = Vec::new();
    let score = analyze_security(&requirements, &mut issues);

    assert_eq!(score, 0);
    assert!(issues.iter().any(|i| i.severity == IssueSeverity::Critical));
  }

  #[test]
  fn test_analyze_security_with_requirements() {
    let requirements = vec![
      "The system shall authenticate users.",
      "Data shall be encrypted.",
      "Input shall be validated.",
    ];

    let mut issues = Vec::new();
    let score = analyze_security(&requirements, &mut issues);

    assert!(score >= 60);
  }

  #[test]
  fn test_dimension_analysis_get_score() {
    let scores =
      vec![DimensionScore::with_default_weight(CoreDimension::Completeness, 80).unwrap()];

    let analysis = DimensionAnalysis::new(scores, vec![]);

    let score = analysis.get_score(CoreDimension::Completeness);
    assert!(score.is_some());

    let no_score = analysis.get_score(CoreDimension::Security);
    assert!(no_score.is_none());
  }

  #[test]
  fn test_dimension_analysis_weakest_dimensions() {
    let scores = vec![
      DimensionScore::with_default_weight(CoreDimension::Completeness, 90).unwrap(),
      DimensionScore::with_default_weight(CoreDimension::Security, 40).unwrap(),
      DimensionScore::with_default_weight(CoreDimension::Performance, 30).unwrap(),
    ];

    let analysis = DimensionAnalysis::new(scores, vec![]);

    let weakest = analysis.weakest_dimensions(50);
    assert_eq!(weakest.len(), 2);
    assert_eq!(weakest[0].dimension, CoreDimension::Performance); // Lowest first
  }

  #[test]
  fn test_dimension_analysis_passes_gate() {
    let scores =
      vec![DimensionScore::with_default_weight(CoreDimension::Completeness, 80).unwrap()];

    let analysis = DimensionAnalysis::new(scores, vec![]);

    assert!(analysis.passes_gate(70, 0));

    let critical_issues = vec![DimensionIssue::new(
      CoreDimension::Security,
      IssueSeverity::Critical,
      "Critical issue".to_string(),
    )];

    let analysis_with_issues = DimensionAnalysis::new(vec![], critical_issues);
    assert!(!analysis_with_issues.passes_gate(70, 0));
  }

  #[test]
  fn test_category_scores_calculation() {
    let scores = vec![
      DimensionScore::with_default_weight(CoreDimension::Completeness, 80).unwrap(),
      DimensionScore::with_default_weight(CoreDimension::Correctness, 90).unwrap(),
    ];

    let analysis = DimensionAnalysis::new(scores, vec![]);

    let functional_score = analysis.category_scores.get(&DimensionCategory::Functional);
    assert!(functional_score.is_some());
  }

  #[test]
  fn test_overall_score_weighted() {
    let scores = vec![
      DimensionScore::new(CoreDimension::Security, 100, 1.5).unwrap(),
      DimensionScore::new(CoreDimension::Usability, 50, 0.5).unwrap(),
    ];

    let analysis = DimensionAnalysis::new(scores, vec![]);

    // Weighted: (100 * 1.5 + 50 * 0.5) / (1.5 + 0.5) = 175 / 2 = 87.5
    assert!(analysis.overall_score >= 85 && analysis.overall_score <= 90);
  }

  #[test]
  fn test_all_dimensions_covered() {
    let requirements = vec!["The system shall process user requests."];
    let analysis = analyze_dimensions(&requirements);

    // All 10 dimensions should have scores
    assert_eq!(analysis.scores.len(), 10);
  }
}
