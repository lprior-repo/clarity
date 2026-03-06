//! Quality Analyzer (WP28) - Quality scoring and analysis for specs
//!
//! This module provides comprehensive quality analysis for specifications,
//! calculating scores for coverage, clarity, testability, and AI readiness.

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::cast_precision_loss, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro)]

use serde::{Deserialize, Serialize};
use std::fmt::Write;

use crate::intent::types::{Behavior, Feature, Spec, Verification};

/// Quality issue detected during analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityIssue {
  /// Missing tests for error conditions
  MissingErrorTests,
  /// Missing authentication/authorization tests
  MissingAuthenticationTest,
  /// Missing edge case coverage
  MissingEdgeCases,
  /// Rules are too vague or ambiguous
  VagueRules,
  /// No examples provided for behaviors
  NoExamples,
  /// Missing explanations for behaviors
  MissingExplanations,
  /// Invariants defined but not tested
  UntestedInvariants,
  /// Missing AI hints for code generation
  MissingAiHints,
  /// Missing preconditions for behaviors
  MissingPreconditions,
  /// Missing postconditions for behaviors
  MissingPostconditions,
}

impl QualityIssue {
  /// Get a human-readable description of the issue
  #[must_use]
  pub const fn description(&self) -> &'static str {
    match self {
      Self::MissingErrorTests => "No error condition tests defined",
      Self::MissingAuthenticationTest => "No authentication/authorization tests defined",
      Self::MissingEdgeCases => "Edge cases not covered in specifications",
      Self::VagueRules => "Some rules are too vague or ambiguous",
      Self::NoExamples => "No examples provided for verification",
      Self::MissingExplanations => "Missing descriptions or explanations",
      Self::UntestedInvariants => "Invariants defined but no tests verify them",
      Self::MissingAiHints => "Missing AI hints for better code generation",
      Self::MissingPreconditions => "Behaviors missing preconditions",
      Self::MissingPostconditions => "Behaviors missing postconditions",
    }
  }

  /// Get a suggestion for fixing the issue
  #[must_use]
  pub const fn suggestion(&self) -> &'static str {
    match self {
      Self::MissingErrorTests => "Add verification tests for error conditions and failure cases",
      Self::MissingAuthenticationTest => "Add tests for authentication and authorization scenarios",
      Self::MissingEdgeCases => {
        "Define edge cases: empty inputs, boundary values, concurrent access"
      }
      Self::VagueRules => "Make rules more specific with concrete examples and constraints",
      Self::NoExamples => "Add example test cases to verification definitions",
      Self::MissingExplanations => "Add detailed descriptions to behaviors and features",
      Self::UntestedInvariants => "Add verification tests that validate invariant constraints",
      Self::MissingAiHints => "Add AI hints section with implementation guidance",
      Self::MissingPreconditions => "Define preconditions: what must be true before execution",
      Self::MissingPostconditions => "Define postconditions: what must be true after execution",
    }
  }
}

/// Quality metrics for a spec
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityReport {
  /// Coverage score (0-100): error tests, auth tests, edge cases, invariants
  pub coverage_score: u8,
  /// Clarity score (0-100): intent ratio, notes ratio, vague penalties
  pub clarity_score: u8,
  /// Testability score (0-100): dependencies, pre/postconditions, examples
  pub testability_score: u8,
  /// AI readiness score (0-100): AI hints, verification, examples
  pub ai_readiness_score: u8,
  /// Overall score (0-100): weighted average
  pub overall_score: u8,
  /// Issues detected during analysis
  pub issues: Vec<QualityIssue>,
  /// Suggestions for improvement
  pub suggestions: Vec<String>,
}

impl QualityReport {
  /// Create a new quality report with the given scores
  #[must_use]
  pub fn new(
    coverage_score: u8,
    clarity_score: u8,
    testability_score: u8,
    ai_readiness_score: u8,
  ) -> Self {
    let overall_score = calculate_overall_score_from_values(
      coverage_score,
      clarity_score,
      testability_score,
      ai_readiness_score,
    );

    Self {
      coverage_score,
      clarity_score,
      testability_score,
      ai_readiness_score,
      overall_score,
      issues: Vec::new(),
      suggestions: Vec::new(),
    }
  }

  /// Check if the report has any issues
  #[must_use]
  pub const fn has_issues(&self) -> bool {
    !self.issues.is_empty()
  }

  /// Get the number of issues
  #[must_use]
  pub const fn issue_count(&self) -> usize {
    self.issues.len()
  }

  /// Add an issue and its corresponding suggestion
  pub fn add_issue(&mut self, issue: QualityIssue) {
    if !self.issues.contains(&issue) {
      self.issues.push(issue);
      self.suggestions.push(issue.suggestion().to_string());
    }
  }

  /// Merge another report's issues into this one
  pub fn merge_issues(&mut self, other: &Self) {
    for issue in &other.issues {
      self.add_issue(*issue);
    }
  }
}

/// Analyze a spec and produce a quality report
///
/// This is the main entry point for quality analysis, calculating:
/// - Coverage score: based on error tests, auth tests, edge cases, invariants
/// - Clarity score: based on intent ratio, notes ratio, vague penalties
/// - Testability score: based on dependencies, pre/postconditions, examples
/// - AI readiness score: based on AI hints, verification, examples
#[must_use]
pub fn analyze_spec(spec: &Spec) -> QualityReport {
  let coverage_score = calculate_coverage_score(spec);
  let clarity_score = calculate_clarity_score(spec);
  let testability_score = calculate_testability_score(spec);
  let ai_readiness_score = calculate_ai_readiness_score(spec);

  let mut report = QualityReport::new(
    coverage_score,
    clarity_score,
    testability_score,
    ai_readiness_score,
  );

  // Collect issues from each dimension
  collect_coverage_issues(spec, &mut report);
  collect_clarity_issues(spec, &mut report);
  collect_testability_issues(spec, &mut report);
  collect_ai_readiness_issues(spec, &mut report);

  report
}

/// Format a quality report for human-readable output
#[must_use]
pub fn format_report(report: &QualityReport) -> String {
  let mut output = String::new();

  output.push_str("=== Quality Report ===\n\n");
  let _ = write!(
    output,
    "Coverage Score:     {:3}/100\n\
         Clarity Score:      {:3}/100\n\
         Testability Score:  {:3}/100\n\
         AI Readiness Score: {:3}/100\n\
         ------------------------\n\
         Overall Score:      {:3}/100\n\n",
    report.coverage_score,
    report.clarity_score,
    report.testability_score,
    report.ai_readiness_score,
    report.overall_score
  );

  if report.has_issues() {
    let _ = writeln!(output, "Issues Found ({}):", report.issue_count());
    for (idx, issue) in report.issues.iter().enumerate() {
      let _ = writeln!(output, "  {}. {}", idx + 1, issue.description());
    }
    output.push('\n');
  } else {
    output.push_str("No issues found. Spec quality is excellent!\n\n");
  }

  if !report.suggestions.is_empty() {
    output.push_str("Suggestions for Improvement:\n");
    for suggestion in &report.suggestions {
      let _ = writeln!(output, "  - {suggestion}");
    }
  }

  output
}

/// Calculate coverage score (0-100)
///
/// Checks for:
/// - Error condition tests
/// - Authentication/authorization tests
/// - Edge case coverage
/// - Invariant testing
#[must_use]
pub fn calculate_coverage_score(spec: &Spec) -> u8 {
  let mut score: u16 = 100;

  let has_error_tests = check_has_error_tests(spec);
  let has_auth_tests = check_has_auth_tests(spec);
  let has_edge_cases = check_has_edge_cases(spec);
  let invariants_tested = check_invariants_tested(spec);

  // Penalize for missing coverage areas
  if !has_error_tests {
    score = score.saturating_sub(20);
  }
  if !has_auth_tests {
    score = score.saturating_sub(15);
  }
  if !has_edge_cases {
    score = score.saturating_sub(15);
  }
  if !invariants_tested {
    score = score.saturating_sub(20);
  }

  // Bonus for comprehensive verification
  let verification_ratio = calculate_verification_ratio(spec);
  if verification_ratio > 0.8 {
    score = score.saturating_add(10);
  }

  // Bonus for having invariants defined
  if !spec.invariants.is_empty() {
    score = score.saturating_add(5);
  }

  u8::try_from(score.min(100)).unwrap_or(100)
}

/// Calculate clarity score (0-100)
///
/// Checks for:
/// - Intent/description ratio
/// - Notes/explanation ratio
/// - Vague language penalties
#[must_use]
pub fn calculate_clarity_score(spec: &Spec) -> u8 {
  let mut score: u16 = 100;

  // Check description coverage
  let desc_ratio = calculate_description_ratio(spec);
  if desc_ratio < 0.5 {
    score = score.saturating_sub(20);
  } else if desc_ratio < 0.8 {
    score = score.saturating_sub(10);
  }

  // Check for vague language
  let vague_count = count_vague_language(spec);
  score = score.saturating_sub(u16::from(vague_count.saturating_mul(5)));

  // Bonus for good documentation
  if desc_ratio >= 0.9 {
    score = score.saturating_add(5);
  }

  // Check spec-level description
  if spec.description.is_empty() {
    score = score.saturating_sub(10);
  }

  u8::try_from(score.min(100)).unwrap_or(100)
}

/// Calculate testability score (0-100)
///
/// Checks for:
/// - Dependencies documented
/// - Preconditions defined
/// - Postconditions defined
/// - Examples provided
#[must_use]
pub fn calculate_testability_score(spec: &Spec) -> u8 {
  let mut score: u16 = 100;

  let precond_ratio = calculate_precondition_ratio(spec);
  let postcond_ratio = calculate_postcondition_ratio(spec);
  let example_ratio = calculate_example_ratio(spec);
  let deps_ratio = calculate_dependency_documentation_ratio(spec);

  // Penalize for missing testability elements
  if precond_ratio < 0.5 {
    score = score.saturating_sub(15);
  }
  if postcond_ratio < 0.5 {
    score = score.saturating_sub(15);
  }
  if example_ratio < 0.5 {
    score = score.saturating_sub(15);
  }
  if deps_ratio < 0.5 {
    score = score.saturating_sub(10);
  }

  // Bonus for comprehensive testability
  if precond_ratio >= 0.8 && postcond_ratio >= 0.8 {
    score = score.saturating_add(10);
  }
  if example_ratio >= 0.8 {
    score = score.saturating_add(5);
  }

  u8::try_from(score.min(100)).unwrap_or(100)
}

/// Calculate AI readiness score (0-100)
///
/// Checks for:
/// - AI hints present
/// - Verification examples
/// - Implementation guidance
#[must_use]
pub fn calculate_ai_readiness_score(spec: &Spec) -> u8 {
  let mut score: u16 = 100;

  let ai_hints = &spec.ai_hints;

  // Check for AI hints sections
  let has_impl_hints = !ai_hints.implementation.architecture.is_empty()
    || !ai_hints.implementation.performance_notes.is_empty()
    || !ai_hints.implementation.error_handling.is_empty();

  let has_security_hints = !ai_hints.security.authentication.is_empty()
    || !ai_hints.security.authorization.is_empty()
    || !ai_hints.security.data_sensitivity.is_empty()
    || !ai_hints.security.concerns.is_empty();

  let has_entity_hints = !ai_hints.entities.is_empty();
  let has_lib_hints = !ai_hints.preferred_libraries.is_empty();
  let has_style_hints = !ai_hints.style_hints.is_empty();

  // Penalize for missing AI hints
  if !has_impl_hints {
    score = score.saturating_sub(20);
  }
  if !has_security_hints {
    score = score.saturating_sub(15);
  }
  if !has_entity_hints {
    score = score.saturating_sub(10);
  }
  if !has_lib_hints {
    score = score.saturating_sub(5);
  }
  if !has_style_hints {
    score = score.saturating_sub(5);
  }

  // Check for verification examples (helps AI understand expected behavior)
  let example_ratio = calculate_example_ratio(spec);
  if example_ratio < 0.5 {
    score = score.saturating_sub(15);
  }

  // Bonus for comprehensive AI hints
  if has_impl_hints && has_security_hints && has_entity_hints {
    score = score.saturating_add(10);
  }

  u8::try_from(score.min(100)).unwrap_or(100)
}

/// Calculate overall score as weighted average
///
/// Weights:
/// - Coverage: 30%
/// - Clarity: 25%
/// - Testability: 25%
/// - AI Readiness: 20%
#[must_use]
pub fn calculate_overall_score(report: &QualityReport) -> u8 {
  calculate_overall_score_from_values(
    report.coverage_score,
    report.clarity_score,
    report.testability_score,
    report.ai_readiness_score,
  )
}

/// Calculate overall score from individual scores
fn calculate_overall_score_from_values(
  coverage: u8,
  clarity: u8,
  testability: u8,
  ai_readiness: u8,
) -> u8 {
  // Weighted average: 30% coverage, 25% clarity, 25% testability, 20% AI readiness
  let weighted_sum = u16::from(coverage)
    .saturating_mul(30)
    .saturating_add(u16::from(clarity).saturating_mul(25))
    .saturating_add(u16::from(testability).saturating_mul(25))
    .saturating_add(u16::from(ai_readiness).saturating_mul(20));

  u8::try_from(weighted_sum / 100).unwrap_or(100)
}

// =============================================================================
// Helper functions for coverage analysis
// =============================================================================

/// Check if any behaviors have error condition tests
fn check_has_error_tests(spec: &Spec) -> bool {
  spec.features.iter().any(|f| {
    f.behaviors.iter().any(|b| {
      b.verification.as_ref().is_some_and(|v| {
        let desc_lower = v.description.to_lowercase();
        let example_lower = v.example.to_lowercase();
        desc_lower.contains("error")
          || desc_lower.contains("fail")
          || desc_lower.contains("invalid")
          || example_lower.contains("error")
          || example_lower.contains("fail")
          || example_lower.contains("invalid")
      })
    })
  })
}

/// Check if any behaviors have authentication/authorization tests
fn check_has_auth_tests(spec: &Spec) -> bool {
  let has_auth_behavior = spec.features.iter().any(|f| {
    f.name.to_lowercase().contains("auth")
      || f.behaviors.iter().any(|b| {
        b.name.to_lowercase().contains("auth")
          || b.name.to_lowercase().contains("login")
          || b.name.to_lowercase().contains("permission")
      })
  });

  let has_auth_verification = spec.features.iter().any(|f| {
    f.behaviors.iter().any(|b| {
      b.verification.as_ref().is_some_and(|v| {
        let desc_lower = v.description.to_lowercase();
        desc_lower.contains("auth")
          || desc_lower.contains("unauthorized")
          || desc_lower.contains("forbidden")
          || desc_lower.contains("permission")
      })
    })
  });

  let has_security_hints = !spec.ai_hints.security.authentication.is_empty()
    || !spec.ai_hints.security.authorization.is_empty();

  has_auth_behavior || has_auth_verification || has_security_hints
}

/// Check if edge cases are documented
fn check_has_edge_cases(spec: &Spec) -> bool {
  spec.features.iter().any(|f: &Feature| {
    f.behaviors.iter().any(|b: &Behavior| {
      // Check preconditions for edge case hints
      let precond_has_edge = b.preconditions.iter().any(|p: &String| {
        let p_lower = p.to_lowercase();
        p_lower.contains("empty")
          || p_lower.contains("null")
          || p_lower.contains("boundary")
          || p_lower.contains("limit")
          || p_lower.contains("max")
          || p_lower.contains("min")
      });

      // Check verification for edge case testing
      let verif_has_edge = b.verification.as_ref().is_some_and(|v: &Verification| {
        let desc_lower = v.description.to_lowercase();
        let example_lower = v.example.to_lowercase();
        desc_lower.contains("edge")
          || desc_lower.contains("boundary")
          || desc_lower.contains("empty")
          || example_lower.contains("edge")
          || example_lower.contains("empty")
          || example_lower.contains("null")
      });

      precond_has_edge || verif_has_edge
    })
  })
}

/// Check if invariants have corresponding tests
fn check_invariants_tested(spec: &Spec) -> bool {
  if spec.invariants.is_empty() {
    return true; // No invariants = trivially tested
  }

  // Check if any verification mentions invariants
  let verif_mentions_invariant = spec.features.iter().any(|f| {
    f.behaviors.iter().any(|b| {
      b.verification.as_ref().is_some_and(|v| {
        v.description.to_lowercase().contains("invariant")
          || v.example.to_lowercase().contains("invariant")
      })
    })
  });

  // Check if invariants are referenced in postconditions
  let postcond_mentions_invariant = spec.features.iter().any(|f| {
    f.behaviors.iter().any(|b| {
      b.postconditions
        .iter()
        .any(|p| p.to_lowercase().contains("invariant"))
    })
  });

  verif_mentions_invariant || postcond_mentions_invariant
}

/// Calculate ratio of behaviors with verification
fn calculate_verification_ratio(spec: &Spec) -> f64 {
  let total = total_behavior_count(spec);
  if total == 0 {
    return 1.0;
  }

  let with_verification = spec
    .features
    .iter()
    .map(|f| {
      f.behaviors
        .iter()
        .filter(|b| b.verification.is_some())
        .count()
    })
    .sum::<usize>();

  (with_verification as f64) / (total as f64)
}

// =============================================================================
// Helper functions for clarity analysis
// =============================================================================

/// Calculate ratio of behaviors with descriptions
fn calculate_description_ratio(spec: &Spec) -> f64 {
  let total = total_behavior_count(spec);
  if total == 0 {
    return 1.0;
  }

  let with_description = spec
    .features
    .iter()
    .map(|f| {
      f.behaviors
        .iter()
        .filter(|b| !b.description.is_empty())
        .count()
    })
    .sum::<usize>();

  (with_description as f64) / (total as f64)
}

/// Count instances of vague language
fn count_vague_language(spec: &Spec) -> u8 {
  let vague_words = [
    "maybe",
    "perhaps",
    "probably",
    "might",
    "could",
    "somehow",
    "something",
    "stuff",
    "things",
    "etc",
    "and so on",
    "roughly",
    "approximately",
    "usually",
    "typically",
    "generally",
    "often",
    "sometimes",
  ];

  let mut count: u8 = 0;

  // Check spec description
  let spec_desc_lower = spec.description.to_lowercase();
  for word in &vague_words {
    if spec_desc_lower.contains(word) {
      count = count.saturating_add(1);
    }
  }

  // Check feature descriptions
  for feature in &spec.features {
    let feat_desc_lower = feature.description.to_lowercase();
    for word in &vague_words {
      if feat_desc_lower.contains(word) {
        count = count.saturating_add(1);
      }
    }

    // Check behavior descriptions
    for behavior in &feature.behaviors {
      let beh_desc_lower = behavior.description.to_lowercase();
      for word in &vague_words {
        if beh_desc_lower.contains(word) {
          count = count.saturating_add(1);
        }
      }
    }
  }

  count.min(20) // Cap at 20 to avoid over-penalizing
}

// =============================================================================
// Helper functions for testability analysis
// =============================================================================

/// Calculate ratio of behaviors with preconditions
fn calculate_precondition_ratio(spec: &Spec) -> f64 {
  let total = total_behavior_count(spec);
  if total == 0 {
    return 1.0;
  }

  let with_precond = spec
    .features
    .iter()
    .map(|f| {
      f.behaviors
        .iter()
        .filter(|b| !b.preconditions.is_empty())
        .count()
    })
    .sum::<usize>();

  (with_precond as f64) / (total as f64)
}

/// Calculate ratio of behaviors with postconditions
fn calculate_postcondition_ratio(spec: &Spec) -> f64 {
  let total = total_behavior_count(spec);
  if total == 0 {
    return 1.0;
  }

  let with_postcond = spec
    .features
    .iter()
    .map(|f| {
      f.behaviors
        .iter()
        .filter(|b| !b.postconditions.is_empty())
        .count()
    })
    .sum::<usize>();

  (with_postcond as f64) / (total as f64)
}

/// Calculate ratio of behaviors with examples in verification
fn calculate_example_ratio(spec: &Spec) -> f64 {
  let total = total_behavior_count(spec);
  if total == 0 {
    return 1.0;
  }

  let with_example = spec
    .features
    .iter()
    .map(|f| {
      f.behaviors
        .iter()
        .filter(|b| {
          b.verification
            .as_ref()
            .is_some_and(|v| !v.example.is_empty())
        })
        .count()
    })
    .sum::<usize>();

  (with_example as f64) / (total as f64)
}

/// Calculate ratio of features with documented dependencies
fn calculate_dependency_documentation_ratio(spec: &Spec) -> f64 {
  if spec.features.is_empty() {
    return 1.0;
  }

  // Features with explicit dependencies or no dependencies needed
  let documented = spec
    .features
    .iter()
    .filter(|f| !f.depends_on.is_empty() || f.behaviors.is_empty())
    .count();

  (documented as f64) / (spec.features.len() as f64)
}

// =============================================================================
// Helper functions for counting
// =============================================================================

/// Get total behavior count across all features
fn total_behavior_count(spec: &Spec) -> usize {
  spec.features.iter().map(|f| f.behaviors.len()).sum()
}

// =============================================================================
// Issue collection functions
// =============================================================================

/// Collect coverage-related issues
fn collect_coverage_issues(spec: &Spec, report: &mut QualityReport) {
  if !check_has_error_tests(spec) {
    report.add_issue(QualityIssue::MissingErrorTests);
  }
  if !check_has_auth_tests(spec) {
    report.add_issue(QualityIssue::MissingAuthenticationTest);
  }
  if !check_has_edge_cases(spec) {
    report.add_issue(QualityIssue::MissingEdgeCases);
  }
  if !spec.invariants.is_empty() && !check_invariants_tested(spec) {
    report.add_issue(QualityIssue::UntestedInvariants);
  }
}

/// Collect clarity-related issues
fn collect_clarity_issues(spec: &Spec, report: &mut QualityReport) {
  if calculate_description_ratio(spec) < 0.5 {
    report.add_issue(QualityIssue::MissingExplanations);
  }
  if count_vague_language(spec) > 3 {
    report.add_issue(QualityIssue::VagueRules);
  }
}

/// Collect testability-related issues
fn collect_testability_issues(spec: &Spec, report: &mut QualityReport) {
  if calculate_precondition_ratio(spec) < 0.5 {
    report.add_issue(QualityIssue::MissingPreconditions);
  }
  if calculate_postcondition_ratio(spec) < 0.5 {
    report.add_issue(QualityIssue::MissingPostconditions);
  }
  if calculate_example_ratio(spec) < 0.5 {
    report.add_issue(QualityIssue::NoExamples);
  }
}

/// Collect AI readiness-related issues
fn collect_ai_readiness_issues(spec: &Spec, report: &mut QualityReport) {
  let ai_hints = &spec.ai_hints;

  let has_any_hints = !ai_hints.implementation.architecture.is_empty()
    || !ai_hints.implementation.performance_notes.is_empty()
    || !ai_hints.implementation.error_handling.is_empty()
    || !ai_hints.entities.is_empty()
    || !ai_hints.preferred_libraries.is_empty()
    || !ai_hints.style_hints.is_empty()
    || !ai_hints.security.authentication.is_empty()
    || !ai_hints.security.authorization.is_empty();

  if !has_any_hints {
    report.add_issue(QualityIssue::MissingAiHints);
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {
  use super::*;
  use crate::intent::types::{AIHints, Behavior, Feature, Invariant, Verification};

  fn create_minimal_spec() -> Spec {
    Spec::new("test-spec".to_string()).expect("valid spec name")
  }

  fn create_behavior_with_verification(name: &str, verification: Verification) -> Behavior {
    Behavior::new(name.to_string())
      .expect("valid behavior name")
      .with_verification(verification)
  }

  fn create_behavior_with_preconditions(name: &str, preconditions: Vec<&str>) -> Behavior {
    let mut behavior = Behavior::new(name.to_string()).expect("valid behavior name");
    for precond in preconditions {
      behavior.add_precondition(precond.to_string());
    }
    behavior
  }

  fn create_behavior_with_postconditions(name: &str, postconditions: Vec<&str>) -> Behavior {
    let mut behavior = Behavior::new(name.to_string()).expect("valid behavior name");
    for postcond in postconditions {
      behavior.add_postcondition(postcond.to_string());
    }
    behavior
  }

  fn create_feature_with_behaviors(name: &str, behaviors: Vec<Behavior>) -> Feature {
    let mut feature = Feature::new(name.to_string()).expect("valid feature name");
    for behavior in behaviors {
      feature.add_behavior(behavior).expect("should add behavior");
    }
    feature
  }

  #[test]
  fn test_quality_issue_descriptions() {
    assert!(!QualityIssue::MissingErrorTests.description().is_empty());
    assert!(!QualityIssue::VagueRules.description().is_empty());
    assert!(!QualityIssue::MissingAiHints.description().is_empty());
  }

  #[test]
  fn test_quality_issue_suggestions() {
    assert!(!QualityIssue::MissingErrorTests.suggestion().is_empty());
    assert!(!QualityIssue::NoExamples.suggestion().is_empty());
  }

  #[test]
  fn test_quality_report_new() {
    let report = QualityReport::new(80, 70, 90, 60);
    assert_eq!(report.coverage_score, 80);
    assert_eq!(report.clarity_score, 70);
    assert_eq!(report.testability_score, 90);
    assert_eq!(report.ai_readiness_score, 60);
    assert!(report.issues.is_empty());
    assert!(report.suggestions.is_empty());
    assert!(!report.has_issues());
  }

  #[test]
  fn test_quality_report_overall_score_calculation() {
    let report = QualityReport::new(100, 100, 100, 100);
    assert_eq!(report.overall_score, 100);

    let report = QualityReport::new(0, 0, 0, 0);
    assert_eq!(report.overall_score, 0);

    // 30*80 + 25*70 + 25*90 + 20*60 = 2400 + 1750 + 2250 + 1200 = 7600 / 100 = 76
    let report = QualityReport::new(80, 70, 90, 60);
    assert_eq!(report.overall_score, 76);
  }

  #[test]
  fn test_quality_report_add_issue() {
    let mut report = QualityReport::new(100, 100, 100, 100);
    assert!(!report.has_issues());

    report.add_issue(QualityIssue::MissingErrorTests);
    assert!(report.has_issues());
    assert_eq!(report.issue_count(), 1);
    assert_eq!(report.suggestions.len(), 1);

    // Adding same issue again should not duplicate
    report.add_issue(QualityIssue::MissingErrorTests);
    assert_eq!(report.issue_count(), 1);
  }

  #[test]
  fn test_analyze_spec_empty() {
    let spec = create_minimal_spec();
    let report = analyze_spec(&spec);

    // Empty spec should have issues
    assert!(report.has_issues());
    assert!(report.coverage_score < 100);
  }

  #[test]
  fn test_analyze_spec_with_comprehensive_content() {
    let mut spec = create_minimal_spec();

    // Add feature with complete behavior
    let verification =
      Verification::new("unit_test".to_string(), "Test with error cases".to_string())
        .with_example("assert!(result.is_ok())".to_string());

    let mut behavior = create_behavior_with_verification("create_user", verification);
    behavior = behavior.with_description("Create a new user account".to_string());
    behavior.add_precondition("User is authenticated".to_string());
    behavior.add_postcondition("User exists in database".to_string());

    let feature = create_feature_with_behaviors("user_management", vec![behavior]);
    spec.add_feature(feature).expect("should add feature");

    // Add invariant
    spec.add_invariant(Invariant::new(
      "unique_email".to_string(),
      "Email must be unique".to_string(),
    ));

    // Add AI hints
    let mut ai_hints = AIHints::default();
    ai_hints.implementation.architecture = "Clean Architecture".to_string();
    ai_hints.security.authentication = "JWT tokens".to_string();
    spec = spec.with_ai_hints(ai_hints);

    let report = analyze_spec(&spec);

    // Should have better scores than empty spec
    assert!(report.clarity_score > 50);
    assert!(report.testability_score > 50);
  }

  #[test]
  fn test_calculate_coverage_score_empty() {
    let spec = create_minimal_spec();
    let score = calculate_coverage_score(&spec);
    assert!(score < 100);
  }

  #[test]
  fn test_calculate_coverage_score_with_error_tests() {
    let mut spec = create_minimal_spec();

    let verification = Verification::new(
      "unit_test".to_string(),
      "Test error handling for invalid input".to_string(),
    );
    let behavior = create_behavior_with_verification("process_data", verification);
    let feature = create_feature_with_behaviors("data", vec![behavior]);
    spec.add_feature(feature).expect("should add feature");

    let score = calculate_coverage_score(&spec);
    // Should be higher than without error tests
    assert!(score > 40);
  }

  #[test]
  fn test_calculate_clarity_score_with_descriptions() {
    let mut spec = create_minimal_spec();
    spec = spec.with_description("A well-documented spec".to_string());

    let behavior = Behavior::new("do_something".to_string())
      .expect("valid behavior name")
      .with_description("This behavior does something specific".to_string());

    let feature = create_feature_with_behaviors("feature1", vec![behavior]);
    spec.add_feature(feature).expect("should add feature");

    let score = calculate_clarity_score(&spec);
    assert!(score > 70);
  }

  #[test]
  fn test_calculate_clarity_score_with_vague_language() {
    let mut spec = create_minimal_spec();
    spec = spec.with_description("This spec maybe does something probably".to_string());

    let behavior = Behavior::new("do_something".to_string())
      .expect("valid behavior name")
      .with_description("Sometimes it might work, stuff happens".to_string());

    let feature = create_feature_with_behaviors("feature1", vec![behavior]);
    spec.add_feature(feature).expect("should add feature");

    let score = calculate_clarity_score(&spec);
    // Should be penalized for vague language
    assert!(score < 90);
  }

  #[test]
  fn test_calculate_testability_score_with_preconditions() {
    let behavior = create_behavior_with_preconditions(
      "create_user",
      vec!["User is authenticated", "Email is valid"],
    );
    let feature = create_feature_with_behaviors("users", vec![behavior]);

    let mut spec = create_minimal_spec();
    spec.add_feature(feature).expect("should add feature");

    let score = calculate_testability_score(&spec);
    // Having preconditions should improve the score, but not massively
    // since we're still missing postconditions and examples
    assert!(score > 50);
  }

  #[test]
  fn test_calculate_testability_score_with_postconditions() {
    let behavior = create_behavior_with_postconditions(
      "create_user",
      vec!["User exists in database", "ID is assigned"],
    );
    let feature = create_feature_with_behaviors("users", vec![behavior]);

    let mut spec = create_minimal_spec();
    spec.add_feature(feature).expect("should add feature");

    let score = calculate_testability_score(&spec);
    // Having postconditions should improve the score, but not massively
    // since we're still missing preconditions and examples
    assert!(score > 50);
  }

  #[test]
  fn test_calculate_ai_readiness_score_with_hints() {
    let mut ai_hints = AIHints::default();
    ai_hints.implementation.architecture = "Hexagonal Architecture".to_string();
    ai_hints.implementation.error_handling = "Result types".to_string();
    ai_hints.security.authentication = "OAuth2".to_string();
    ai_hints.preferred_libraries = vec!["tokio".to_string(), "serde".to_string()];

    let spec = create_minimal_spec().with_ai_hints(ai_hints);
    let score = calculate_ai_readiness_score(&spec);

    assert!(score > 50);
  }

  #[test]
  fn test_calculate_ai_readiness_score_empty() {
    let spec = create_minimal_spec();
    let score = calculate_ai_readiness_score(&spec);
    // Empty AI hints should result in low score
    assert!(score < 70);
  }

  #[test]
  fn test_format_report_empty() {
    let report = QualityReport::new(80, 70, 90, 60);
    let formatted = format_report(&report);

    assert!(formatted.contains("Coverage Score:"));
    assert!(formatted.contains("80/100"));
    assert!(formatted.contains("Clarity Score:"));
    assert!(formatted.contains("70/100"));
    assert!(formatted.contains("Testability Score:"));
    assert!(formatted.contains("90/100"));
    assert!(formatted.contains("AI Readiness Score:"));
    assert!(formatted.contains("60/100"));
    assert!(formatted.contains("Overall Score:"));
    assert!(formatted.contains("76/100"));
  }

  #[test]
  fn test_format_report_with_issues() {
    let mut report = QualityReport::new(50, 50, 50, 50);
    report.add_issue(QualityIssue::MissingErrorTests);
    report.add_issue(QualityIssue::NoExamples);

    let formatted = format_report(&report);

    assert!(formatted.contains("Issues Found (2)"));
    assert!(formatted.contains("No error condition tests defined"));
    assert!(formatted.contains("No examples provided for verification"));
    assert!(formatted.contains("Suggestions for Improvement"));
  }

  #[test]
  fn test_check_has_error_tests_positive() {
    let mut spec = create_minimal_spec();

    let verification =
      Verification::new("unit_test".to_string(), "Test error handling".to_string());
    let behavior = create_behavior_with_verification("process", verification);
    let feature = create_feature_with_behaviors("api", vec![behavior]);
    spec.add_feature(feature).expect("should add feature");

    assert!(check_has_error_tests(&spec));
  }

  #[test]
  fn test_check_has_error_tests_negative() {
    let mut spec = create_minimal_spec();

    let verification = Verification::new("unit_test".to_string(), "Test success path".to_string());
    let behavior = create_behavior_with_verification("process", verification);
    let feature = create_feature_with_behaviors("api", vec![behavior]);
    spec.add_feature(feature).expect("should add feature");

    assert!(!check_has_error_tests(&spec));
  }

  #[test]
  fn test_check_has_auth_tests_positive() {
    let mut spec = create_minimal_spec();

    let behavior = Behavior::new("login_user".to_string()).expect("valid behavior name");
    let feature = create_feature_with_behaviors("authentication", vec![behavior]);
    spec.add_feature(feature).expect("should add feature");

    assert!(check_has_auth_tests(&spec));
  }

  #[test]
  fn test_check_has_edge_cases_positive() {
    let mut spec = create_minimal_spec();

    let mut behavior = Behavior::new("process".to_string()).expect("valid behavior name");
    behavior.add_precondition("Input is not empty".to_string());

    let feature = create_feature_with_behaviors("data", vec![behavior]);
    spec.add_feature(feature).expect("should add feature");

    assert!(check_has_edge_cases(&spec));
  }

  #[test]
  fn test_check_invariants_tested_when_empty() {
    let spec = create_minimal_spec();
    // No invariants = trivially tested
    assert!(check_invariants_tested(&spec));
  }

  #[test]
  fn test_check_invariants_tested_with_verification() {
    let mut spec = create_minimal_spec();
    spec.add_invariant(Invariant::new(
      "test".to_string(),
      "Test invariant".to_string(),
    ));

    let verification = Verification::new(
      "unit_test".to_string(),
      "Verify invariant holds".to_string(),
    );
    let behavior = create_behavior_with_verification("check", verification);
    let feature = create_feature_with_behaviors("core", vec![behavior]);
    spec.add_feature(feature).expect("should add feature");

    assert!(check_invariants_tested(&spec));
  }

  #[test]
  fn test_calculate_verification_ratio() {
    let mut spec = create_minimal_spec();

    // Add two behaviors, one with verification
    let behavior1 = Behavior::new("no_verif".to_string()).expect("valid behavior name");
    let behavior2 = create_behavior_with_verification(
      "with_verif",
      Verification::new("test".to_string(), "Test".to_string()),
    );

    let feature = create_feature_with_behaviors("test", vec![behavior1, behavior2]);
    spec.add_feature(feature).expect("should add feature");

    let ratio = calculate_verification_ratio(&spec);
    assert!((ratio - 0.5).abs() < 0.01);
  }

  #[test]
  fn test_calculate_description_ratio() {
    let mut spec = create_minimal_spec();

    let behavior1 = Behavior::new("no_desc".to_string()).expect("valid behavior name");
    let behavior2 = Behavior::new("with_desc".to_string())
      .expect("valid behavior name")
      .with_description("Has description".to_string());

    let feature = create_feature_with_behaviors("test", vec![behavior1, behavior2]);
    spec.add_feature(feature).expect("should add feature");

    let ratio = calculate_description_ratio(&spec);
    assert!((ratio - 0.5).abs() < 0.01);
  }

  #[test]
  fn test_count_vague_language() {
    let spec = create_minimal_spec().with_description("This maybe works sometimes".to_string());

    let count = count_vague_language(&spec);
    assert!(count >= 2); // "maybe" and "sometimes"
  }

  #[test]
  fn test_collect_coverage_issues() {
    let spec = create_minimal_spec();
    let mut report = QualityReport::new(100, 100, 100, 100);

    collect_coverage_issues(&spec, &mut report);

    // Empty spec should have several coverage issues
    assert!(report.issues.contains(&QualityIssue::MissingErrorTests));
  }

  #[test]
  fn test_collect_testability_issues() {
    // Create a spec with behaviors but no pre/postconditions or examples
    let mut spec = create_minimal_spec();

    let behavior = Behavior::new("create_user".to_string()).expect("valid behavior name");
    let feature = create_feature_with_behaviors("users", vec![behavior]);
    spec.add_feature(feature).expect("should add feature");

    let mut report = QualityReport::new(100, 100, 100, 100);

    collect_testability_issues(&spec, &mut report);

    // Spec with behaviors but no pre/postconditions should have issues
    assert!(
      report.issues.contains(&QualityIssue::MissingPreconditions)
        || report.issues.contains(&QualityIssue::MissingPostconditions)
        || report.issues.contains(&QualityIssue::NoExamples)
    );
  }

  #[test]
  fn test_full_analysis_workflow() {
    let mut spec = create_minimal_spec();
    spec = spec.with_description("Test specification".to_string());

    // Add behavior with all quality elements
    let verification = Verification::new(
      "integration_test".to_string(),
      "Test with error cases and edge conditions".to_string(),
    )
    .with_example("assert!(result.is_ok())".to_string());

    let mut behavior = Behavior::new("create_resource".to_string())
      .expect("valid behavior name")
      .with_description("Create a new resource".to_string())
      .with_verification(verification);

    behavior.add_precondition("User is authenticated".to_string());
    behavior.add_postcondition("Resource exists".to_string());

    let feature = create_feature_with_behaviors("resources", vec![behavior]);
    spec.add_feature(feature).expect("should add feature");

    // Add AI hints
    let mut ai_hints = AIHints::default();
    ai_hints.implementation.architecture = "Modular".to_string();
    spec = spec.with_ai_hints(ai_hints);

    let report = analyze_spec(&spec);

    // Should produce a reasonable report
    assert!(report.overall_score > 0);
    assert!(report.overall_score <= 100);

    // Format should work
    let formatted = format_report(&report);
    assert!(formatted.contains("Quality Report"));
  }
}
