//! Quality Improver (WP30) - Spec improvement suggestions
//!
//! Generates specific improvement suggestions based on quality analysis.
//! Provides actionable recommendations for:
//! - Missing tests (error tests, auth tests, edge cases)
//! - Vague rule clarifications with examples
//! - Missing examples for behaviors
//!
//! ## Design Principles
//!
//! - Zero unwrap/expect: All fallible operations return Result
//! - Pure functions: No side effects, deterministic output
//! - Type-safe: Uses domain types from quality module

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// =============================================================================
// Error Types
// =============================================================================

/// Error type for quality improvement operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ImproverError {
  /// Report is empty or invalid
  #[error("quality report is empty")]
  EmptyReport,

  /// Invalid priority value
  #[error("invalid priority value: {0}")]
  InvalidPriority(u8),

  /// Category not found
  #[error("category not found: {0}")]
  CategoryNotFound(String),

  /// Field reference invalid
  #[error("invalid field reference: {0}")]
  InvalidFieldReference(String),
}

// =============================================================================
// Core Types
// =============================================================================

/// Improvement suggestion for spec quality
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImprovementSuggestion {
  /// Category of improvement (e.g., "testing", "clarity", "completeness")
  pub category: String,
  /// Human-readable description of the improvement
  pub description: String,
  /// Priority level (1-10, where 10 is highest)
  pub priority: u8,
  /// The field or area affected by this improvement
  pub affected_field: String,
  /// Specific action to take
  pub suggested_action: String,
}

impl ImprovementSuggestion {
  /// Create a new improvement suggestion
  ///
  /// # Errors
  /// Returns `ImproverError::InvalidPriority` if priority is not in range 1-10
  pub fn new(
    category: impl Into<String>,
    description: impl Into<String>,
    priority: u8,
    affected_field: impl Into<String>,
    suggested_action: impl Into<String>,
  ) -> Result<Self, ImproverError> {
    match priority {
      1..=10 => Ok(Self {
        category: category.into(),
        description: description.into(),
        priority,
        affected_field: affected_field.into(),
        suggested_action: suggested_action.into(),
      }),
      invalid => Err(ImproverError::InvalidPriority(invalid)),
    }
  }

  /// Check if this is a high-priority suggestion (priority >= 8)
  #[must_use]
  pub const fn is_high_priority(&self) -> bool {
    self.priority >= 8
  }

  /// Check if this is a medium-priority suggestion (priority 4-7)
  #[must_use]
  pub fn is_medium_priority(&self) -> bool {
    (4..=7).contains(&self.priority)
  }

  /// Check if this is a low-priority suggestion (priority 1-3)
  #[must_use]
  pub const fn is_low_priority(&self) -> bool {
    self.priority <= 3
  }
}

// =============================================================================
// Quality Report Types (for integration with quality module)
// =============================================================================

/// Quality issue category for reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueCategory {
  /// Missing test coverage
  MissingTests,
  /// Vague or ambiguous rules
  VagueRules,
  /// Missing examples
  MissingExamples,
  /// Missing error handling
  MissingErrors,
  /// Missing authentication/authorization
  MissingAuth,
  /// Missing edge cases
  MissingEdgeCases,
  /// Low completeness score
  LowCompleteness,
  /// Low clarity score
  LowClarity,
  /// Low security score
  LowSecurity,
  /// Low testability score
  LowTestability,
  /// Low consistency score
  LowConsistency,
}

impl IssueCategory {
  /// Get display label for this category
  #[must_use]
  pub const fn label(self) -> &'static str {
    match self {
      Self::MissingTests => "Missing Tests",
      Self::VagueRules => "Vague Rules",
      Self::MissingExamples => "Missing Examples",
      Self::MissingErrors => "Missing Error Handling",
      Self::MissingAuth => "Missing Authentication",
      Self::MissingEdgeCases => "Missing Edge Cases",
      Self::LowCompleteness => "Low Completeness",
      Self::LowClarity => "Low Clarity",
      Self::LowSecurity => "Low Security",
      Self::LowTestability => "Low Testability",
      Self::LowConsistency => "Low Consistency",
    }
  }

  /// Get all categories
  #[must_use]
  pub const fn all() -> &'static [Self] {
    &[
      Self::MissingTests,
      Self::VagueRules,
      Self::MissingExamples,
      Self::MissingErrors,
      Self::MissingAuth,
      Self::MissingEdgeCases,
      Self::LowCompleteness,
      Self::LowClarity,
      Self::LowSecurity,
      Self::LowTestability,
      Self::LowConsistency,
    ]
  }
}

/// Quality issue found in spec analysis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityIssueReport {
  /// Category of the issue
  pub category: IssueCategory,
  /// Severity level (1-10)
  pub severity: u8,
  /// The field or area where the issue was found
  pub field: String,
  /// Description of the issue
  pub description: String,
  /// Optional context or example
  pub context: Option<String>,
}

impl QualityIssueReport {
  /// Create a new quality issue report
  #[must_use]
  pub fn new(
    category: IssueCategory,
    severity: u8,
    field: impl Into<String>,
    description: impl Into<String>,
  ) -> Self {
    Self {
      category,
      severity,
      field: field.into(),
      description: description.into(),
      context: None,
    }
  }

  /// Add context to the issue report
  #[must_use]
  pub fn with_context(mut self, context: impl Into<String>) -> Self {
    self.context = Some(context.into());
    self
  }
}

/// Quality report containing all issues found in spec analysis
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityReport {
  /// Overall quality score (0-100)
  pub overall_score: u8,
  /// All issues found during analysis
  pub issues: Vec<QualityIssueReport>,
  /// Number of behaviors analyzed
  pub behavior_count: usize,
  /// Number of features analyzed
  pub feature_count: usize,
  /// Behaviors without verification
  pub unverified_behaviors: Vec<String>,
  /// Behaviors without examples
  pub behaviors_without_examples: Vec<String>,
  /// Vague rule descriptions detected
  pub vague_rules: Vec<String>,
  /// Missing error test areas
  pub missing_error_tests: Vec<String>,
  /// Missing auth test areas
  pub missing_auth_tests: Vec<String>,
  /// Missing edge cases
  pub missing_edge_cases: Vec<String>,
}

impl QualityReport {
  /// Create a new empty quality report
  #[must_use]
  pub const fn new() -> Self {
    Self {
      overall_score: 0,
      issues: Vec::new(),
      behavior_count: 0,
      feature_count: 0,
      unverified_behaviors: Vec::new(),
      behaviors_without_examples: Vec::new(),
      vague_rules: Vec::new(),
      missing_error_tests: Vec::new(),
      missing_auth_tests: Vec::new(),
      missing_edge_cases: Vec::new(),
    }
  }

  /// Create a quality report with specified values
  #[must_use]
  pub const fn with_scores(overall_score: u8, behavior_count: usize, feature_count: usize) -> Self {
    Self {
      overall_score,
      issues: Vec::new(),
      behavior_count,
      feature_count,
      unverified_behaviors: Vec::new(),
      behaviors_without_examples: Vec::new(),
      vague_rules: Vec::new(),
      missing_error_tests: Vec::new(),
      missing_auth_tests: Vec::new(),
      missing_edge_cases: Vec::new(),
    }
  }

  /// Add an issue to the report
  pub fn add_issue(&mut self, issue: QualityIssueReport) {
    self.issues.push(issue);
  }

  /// Get issues by category
  #[must_use]
  pub fn issues_by_category(&self, category: IssueCategory) -> Vec<&QualityIssueReport> {
    self
      .issues
      .iter()
      .filter(|i| i.category == category)
      .collect()
  }

  /// Check if report has any critical issues (severity >= 8)
  #[must_use]
  pub fn has_critical_issues(&self) -> bool {
    self.issues.iter().any(|i| i.severity >= 8)
  }

  /// Get count of issues by category
  #[must_use]
  pub fn count_by_category(&self, category: IssueCategory) -> usize {
    self
      .issues
      .iter()
      .filter(|i| i.category == category)
      .count()
  }
}

impl Default for QualityReport {
  fn default() -> Self {
    Self::new()
  }
}

// =============================================================================
// Main Improvement Functions
// =============================================================================

/// Generate all improvement suggestions based on quality report
///
/// Analyzes the quality report and generates comprehensive improvement
/// suggestions across all categories.
///
/// # Arguments
/// * `report` - The quality report to analyze
///
/// # Returns
/// A vector of improvement suggestions sorted by priority (highest first)
#[must_use]
pub fn suggest_improvements(report: &QualityReport) -> Vec<ImprovementSuggestion> {
  let mut suggestions = Vec::new();

  // Collect suggestions from all categories
  suggestions.extend(suggest_missing_tests(report));
  suggestions.extend(suggest_vague_rules_improvements(report));
  suggestions.extend(suggest_examples_improvements(report));
  suggestions.extend(suggest_security_improvements(report));
  suggestions.extend(suggest_completeness_improvements(report));
  suggestions.extend(suggest_clarity_improvements(report));
  suggestions.extend(suggest_consistency_improvements(report));

  // Sort by priority (highest first), then by category
  suggestions
    .into_iter()
    .sorted_by(|a, b| {
      b.priority
        .cmp(&a.priority)
        .then(a.category.cmp(&b.category))
    })
    .collect()
}

/// Suggest improvements for missing tests
///
/// Analyzes the report for missing test coverage including:
/// - Error handling tests
/// - Authentication/authorization tests
/// - Edge case tests
///
/// # Arguments
/// * `report` - The quality report to analyze
///
/// # Returns
/// A vector of improvement suggestions for test coverage
#[must_use]
pub fn suggest_missing_tests(report: &QualityReport) -> Vec<ImprovementSuggestion> {
  let mut suggestions = Vec::new();

  // Suggest error tests
  for area in &report.missing_error_tests {
    if let Ok(s) = ImprovementSuggestion::new(
            "testing",
            format!("Add error handling tests for {area}"),
            9,
            area.clone(),
            format!("Create test cases that verify error conditions in {area}. Include tests for: invalid inputs, boundary conditions, resource exhaustion, and failure states."),
        ) { suggestions.push(s) }
  }

  // Suggest auth tests
  for area in &report.missing_auth_tests {
    if let Ok(s) = ImprovementSuggestion::new(
            "testing",
            format!("Add authentication/authorization tests for {area}"),
            10,
            area.clone(),
            format!("Create test cases that verify authentication and authorization in {area}. Include tests for: unauthenticated access, insufficient permissions, token expiration, and role-based access control."),
        ) { suggestions.push(s) }
  }

  // Suggest edge case tests
  for edge_case in &report.missing_edge_cases {
    if let Ok(s) = ImprovementSuggestion::new(
            "testing",
            format!("Add edge case tests for {edge_case}"),
            7,
            edge_case.clone(),
            format!("Create test cases for edge cases in {edge_case}. Consider: empty inputs, maximum values, null/nil handling, concurrent access, and timeout scenarios."),
        ) { suggestions.push(s) }
  }

  // Check for behaviors without verification
  for behavior in &report.unverified_behaviors {
    if let Ok(s) = ImprovementSuggestion::new(
            "testing",
            format!("Add verification for behavior: {behavior}"),
            8,
            behavior.clone(),
            format!("Define verification criteria for {behavior}. Specify: test type (unit/integration/manual), expected outcomes, and validation steps."),
        ) { suggestions.push(s) }
  }

  // Check for low testability score issues
  for issue in report.issues_by_category(IssueCategory::LowTestability) {
    if let Ok(s) = ImprovementSuggestion::new(
      "testing",
      format!("Improve testability: {}", issue.description),
      issue.severity,
      issue.field.clone(),
      format!(
        "Add acceptance criteria and verification steps. {}",
        issue.context.as_deref().unwrap_or("")
      ),
    ) {
      suggestions.push(s)
    }
  }

  suggestions
}

/// Suggest improvements for vague rules
///
/// Analyzes vague rule descriptions and suggests clarifications
/// with specific examples.
///
/// # Arguments
/// * `report` - The quality report to analyze
///
/// # Returns
/// A vector of improvement suggestions for rule clarity
#[must_use]
pub fn suggest_vague_rules_improvements(report: &QualityReport) -> Vec<ImprovementSuggestion> {
  let mut suggestions = Vec::new();

  for rule in &report.vague_rules {
    // Detect type of vagueness and suggest specific improvements
    let (description, action) = analyze_vague_rule(rule);

    if let Ok(s) = ImprovementSuggestion::new("clarity", description, 7, rule.clone(), action) {
      suggestions.push(s)
    }
  }

  // Check for low clarity score issues
  for issue in report.issues_by_category(IssueCategory::LowClarity) {
    if let Ok(s) = ImprovementSuggestion::new(
            "clarity",
            format!("Clarify: {}", issue.description),
            issue.severity,
            issue.field.clone(),
            format!("Rewrite with specific values and examples. Avoid ambiguous terms like 'fast', 'good', or 'appropriate'. {}", issue.context.as_deref().unwrap_or("Use measurable criteria.")),
        ) { suggestions.push(s) }
  }

  suggestions
}

/// Suggest improvements for missing examples
///
/// Identifies behaviors without examples and suggests adding them.
///
/// # Arguments
/// * `report` - The quality report to analyze
///
/// # Returns
/// A vector of improvement suggestions for adding examples
#[must_use]
pub fn suggest_examples_improvements(report: &QualityReport) -> Vec<ImprovementSuggestion> {
  let mut suggestions = Vec::new();

  for behavior in &report.behaviors_without_examples {
    if let Ok(s) = ImprovementSuggestion::new(
            "completeness",
            format!("Add example for behavior: {behavior}"),
            6,
            behavior.clone(),
            format!("Provide a concrete example demonstrating {behavior}. Include: input values, expected output, and any relevant preconditions or context."),
        ) { suggestions.push(s) }
  }

  // Check for low completeness issues
  for issue in report.issues_by_category(IssueCategory::LowCompleteness) {
    if let Ok(s) = ImprovementSuggestion::new(
      "completeness",
      format!("Add missing content: {}", issue.description),
      issue.severity,
      issue.field.clone(),
      format!(
        "Fill in the missing information. {}",
        issue
          .context
          .as_deref()
          .unwrap_or("Provide complete details for this field.")
      ),
    ) {
      suggestions.push(s)
    }
  }

  suggestions
}

// =============================================================================
// Additional Improvement Functions
// =============================================================================

/// Suggest security-related improvements
#[must_use]
fn suggest_security_improvements(report: &QualityReport) -> Vec<ImprovementSuggestion> {
  let mut suggestions = Vec::new();

  for issue in report.issues_by_category(IssueCategory::LowSecurity) {
    if let Ok(s) = ImprovementSuggestion::new(
      "security",
      format!("Address security concern: {}", issue.description),
      10,
      issue.field.clone(),
      format!(
        "Add security controls. {}",
        issue
          .context
          .as_deref()
          .unwrap_or("Consider authentication, authorization, encryption, and input validation.")
      ),
    ) {
      suggestions.push(s)
    }
  }

  // Check for missing auth tests as security issues
  if !report.missing_auth_tests.is_empty() {
    if let Ok(s) = ImprovementSuggestion::new(
            "security",
            "Add comprehensive security test coverage".to_string(),
            10,
            "security".to_string(),
            format!("The following areas need security tests: {}. Include tests for authentication, authorization, input validation, and injection prevention.", report.missing_auth_tests.join(", ")),
        ) { suggestions.push(s) }
  }

  suggestions
}

/// Suggest completeness improvements
#[must_use]
fn suggest_completeness_improvements(report: &QualityReport) -> Vec<ImprovementSuggestion> {
  let mut suggestions = Vec::new();

  // Check overall score
  if report.overall_score < 50 {
    if let Ok(s) = ImprovementSuggestion::new(
            "completeness",
            "Overall quality score is critically low".to_string(),
            10,
            "overall".to_string(),
            "Focus on filling in missing required fields and adding verification criteria before addressing other issues.".to_string(),
        ) { suggestions.push(s) }
  } else if report.overall_score < 70 {
    if let Ok(s) = ImprovementSuggestion::new(
            "completeness",
            "Overall quality score needs improvement".to_string(),
            8,
            "overall".to_string(),
            "Address the identified gaps to improve the overall quality score. Prioritize high-severity issues first.".to_string(),
        ) { suggestions.push(s) }
  }

  // Check for behaviors without verification
  if report.behavior_count > 0 && report.unverified_behaviors.len() > report.behavior_count / 2 {
    if let Ok(s) = ImprovementSuggestion::new(
            "completeness",
            "More than half of behaviors lack verification".to_string(),
            9,
            "verification".to_string(),
            format!("{} of {} behaviors need verification criteria. Define how each behavior will be tested and validated.", report.unverified_behaviors.len(), report.behavior_count),
        ) { suggestions.push(s) }
  }

  suggestions
}

/// Suggest clarity improvements
#[must_use]
fn suggest_clarity_improvements(report: &QualityReport) -> Vec<ImprovementSuggestion> {
  let mut suggestions = Vec::new();

  // Check for common vague patterns across all vague rules
  if report.vague_rules.len() > 3 {
    if let Ok(s) = ImprovementSuggestion::new(
            "clarity",
            "Multiple vague rules detected - consider glossary".to_string(),
            6,
            "documentation".to_string(),
            "Create a glossary defining common terms and acceptable value ranges to ensure consistent interpretation across all rules.".to_string(),
        ) { suggestions.push(s) }
  }

  suggestions
}

/// Suggest consistency improvements
#[must_use]
fn suggest_consistency_improvements(report: &QualityReport) -> Vec<ImprovementSuggestion> {
  let mut suggestions = Vec::new();

  for issue in report.issues_by_category(IssueCategory::LowConsistency) {
    if let Ok(s) = ImprovementSuggestion::new(
      "consistency",
      format!("Resolve inconsistency: {}", issue.description),
      8,
      issue.field.clone(),
      format!(
        "Review and resolve the contradiction. {}",
        issue
          .context
          .as_deref()
          .unwrap_or("Ensure all requirements align and do not conflict.")
      ),
    ) {
      suggestions.push(s)
    }
  }

  suggestions
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Analyze a vague rule and return description and suggested action
fn analyze_vague_rule(rule: &str) -> (String, String) {
  let lower = rule.to_lowercase();

  // Check for common vague patterns
  if lower.contains("fast") || lower.contains("quickly") || lower.contains("performant") {
    (
            format!("Clarify performance requirement: '{rule}'"),
            "Replace vague performance terms with specific measurable values. Example: 'Response time must be under 200ms for 95% of requests' instead of 'fast response'.".to_string(),
        )
  } else if lower.contains("good") || lower.contains("appropriate") || lower.contains("proper") {
    (
            format!("Clarify quality requirement: '{rule}'"),
            "Replace subjective terms with specific criteria. Example: 'Error rate must be below 0.1%' instead of 'good error handling'.".to_string(),
        )
  } else if lower.contains("some") || lower.contains("various") || lower.contains("multiple") {
    (
            format!("Clarify scope: '{rule}'"),
            "Specify exact items or range. Example: 'Support 3-5 concurrent users' instead of 'support multiple users'.".to_string(),
        )
  } else if lower.contains("should") || lower.contains("may") || lower.contains("might") {
    (
            format!("Clarify requirement strength: '{rule}'"),
            "Use 'must' for mandatory requirements or 'should' with explicit conditions. Avoid ambiguous modal verbs without context.".to_string(),
        )
  } else if lower.contains("etc") || lower.contains("and so on") || lower.contains("...") {
    (
            format!("Complete the list: '{rule}'"),
            "Replace 'etc.' with a complete list of items. If the list is too long, provide a comprehensive reference or pattern.".to_string(),
        )
  } else {
    (
            format!("Add specificity to: '{rule}'"),
            "Provide concrete examples, specific values, or measurable criteria. Avoid ambiguous language and ensure the requirement can be objectively verified.".to_string(),
        )
  }
}

// =============================================================================
// Tests
// =============================================================================

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
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro,
  clippy::needless_pass_by_value
)]
mod tests {

  use super::*;

  fn create_test_report() -> QualityReport {
    QualityReport::new()
  }

  fn create_report_with_issues() -> QualityReport {
    let mut report = QualityReport::with_scores(65, 10, 3);
    report.missing_error_tests = vec!["user_creation".to_string(), "data_validation".to_string()];
    report.missing_auth_tests = vec!["admin_panel".to_string()];
    report.missing_edge_cases = vec!["empty_input".to_string()];
    report.unverified_behaviors = vec!["process_payment".to_string()];
    report.behaviors_without_examples = vec!["calculate_total".to_string()];
    report.vague_rules = vec!["system should be fast".to_string()];
    report
  }

  // =========================================================================
  // ImprovementSuggestion Tests
  // =========================================================================

  #[test]
  fn test_improvement_suggestion_new_valid() {
    let suggestion = ImprovementSuggestion::new("testing", "Add test", 5, "field", "action");
    assert!(suggestion.is_ok());
    let s = suggestion.expect("valid suggestion");
    assert_eq!(s.category, "testing");
    assert_eq!(s.priority, 5);
  }

  #[test]
  fn test_improvement_suggestion_invalid_priority_zero() {
    let suggestion = ImprovementSuggestion::new("testing", "Add test", 0, "field", "action");
    assert!(matches!(suggestion, Err(ImproverError::InvalidPriority(0))));
  }

  #[test]
  fn test_improvement_suggestion_invalid_priority_eleven() {
    let suggestion = ImprovementSuggestion::new("testing", "Add test", 11, "field", "action");
    assert!(matches!(
      suggestion,
      Err(ImproverError::InvalidPriority(11))
    ));
  }

  #[test]
  fn test_improvement_suggestion_boundary_priority_one() {
    let suggestion = ImprovementSuggestion::new("testing", "Add test", 1, "field", "action");
    assert!(suggestion.is_ok());
  }

  #[test]
  fn test_improvement_suggestion_boundary_priority_ten() {
    let suggestion = ImprovementSuggestion::new("testing", "Add test", 10, "field", "action");
    assert!(suggestion.is_ok());
  }

  #[test]
  fn test_improvement_suggestion_priority_levels() {
    let high = ImprovementSuggestion::new("cat", "desc", 9, "field", "action").expect("valid");
    let medium = ImprovementSuggestion::new("cat", "desc", 5, "field", "action").expect("valid");
    let low = ImprovementSuggestion::new("cat", "desc", 2, "field", "action").expect("valid");

    assert!(high.is_high_priority());
    assert!(!high.is_medium_priority());
    assert!(!high.is_low_priority());

    assert!(!medium.is_high_priority());
    assert!(medium.is_medium_priority());
    assert!(!medium.is_low_priority());

    assert!(!low.is_high_priority());
    assert!(!low.is_medium_priority());
    assert!(low.is_low_priority());
  }

  // =========================================================================
  // QualityReport Tests
  // =========================================================================

  #[test]
  fn test_quality_report_new() {
    let report = create_test_report();
    assert_eq!(report.overall_score, 0);
    assert!(report.issues.is_empty());
    assert_eq!(report.behavior_count, 0);
    assert_eq!(report.feature_count, 0);
  }

  #[test]
  fn test_quality_report_with_scores() {
    let report = QualityReport::with_scores(75, 20, 5);
    assert_eq!(report.overall_score, 75);
    assert_eq!(report.behavior_count, 20);
    assert_eq!(report.feature_count, 5);
  }

  #[test]
  fn test_quality_report_add_issue() {
    let mut report = create_test_report();
    let issue =
      QualityIssueReport::new(IssueCategory::MissingTests, 8, "test_field", "Missing test");
    report.add_issue(issue);
    assert_eq!(report.issues.len(), 1);
  }

  #[test]
  fn test_quality_report_issues_by_category() {
    let mut report = create_test_report();
    report.add_issue(QualityIssueReport::new(
      IssueCategory::MissingTests,
      8,
      "field1",
      "issue1",
    ));
    report.add_issue(QualityIssueReport::new(
      IssueCategory::VagueRules,
      5,
      "field2",
      "issue2",
    ));
    report.add_issue(QualityIssueReport::new(
      IssueCategory::MissingTests,
      7,
      "field3",
      "issue3",
    ));

    let test_issues = report.issues_by_category(IssueCategory::MissingTests);
    assert_eq!(test_issues.len(), 2);
  }

  #[test]
  fn test_quality_report_has_critical_issues() {
    let mut report = create_test_report();
    assert!(!report.has_critical_issues());

    report.add_issue(QualityIssueReport::new(
      IssueCategory::MissingTests,
      7,
      "field",
      "issue",
    ));
    assert!(!report.has_critical_issues());

    report.add_issue(QualityIssueReport::new(
      IssueCategory::MissingTests,
      8,
      "field",
      "critical issue",
    ));
    assert!(report.has_critical_issues());
  }

  #[test]
  fn test_quality_report_count_by_category() {
    let mut report = create_test_report();
    report.add_issue(QualityIssueReport::new(
      IssueCategory::MissingTests,
      5,
      "field",
      "issue1",
    ));
    report.add_issue(QualityIssueReport::new(
      IssueCategory::MissingTests,
      5,
      "field",
      "issue2",
    ));
    report.add_issue(QualityIssueReport::new(
      IssueCategory::VagueRules,
      5,
      "field",
      "issue3",
    ));

    assert_eq!(report.count_by_category(IssueCategory::MissingTests), 2);
    assert_eq!(report.count_by_category(IssueCategory::VagueRules), 1);
    assert_eq!(report.count_by_category(IssueCategory::MissingExamples), 0);
  }

  // =========================================================================
  // IssueCategory Tests
  // =========================================================================

  #[test]
  fn test_issue_category_labels() {
    assert_eq!(IssueCategory::MissingTests.label(), "Missing Tests");
    assert_eq!(IssueCategory::VagueRules.label(), "Vague Rules");
    assert_eq!(IssueCategory::MissingExamples.label(), "Missing Examples");
    assert_eq!(
      IssueCategory::MissingErrors.label(),
      "Missing Error Handling"
    );
    assert_eq!(IssueCategory::MissingAuth.label(), "Missing Authentication");
    assert_eq!(
      IssueCategory::MissingEdgeCases.label(),
      "Missing Edge Cases"
    );
    assert_eq!(IssueCategory::LowCompleteness.label(), "Low Completeness");
    assert_eq!(IssueCategory::LowClarity.label(), "Low Clarity");
    assert_eq!(IssueCategory::LowSecurity.label(), "Low Security");
    assert_eq!(IssueCategory::LowTestability.label(), "Low Testability");
    assert_eq!(IssueCategory::LowConsistency.label(), "Low Consistency");
  }

  #[test]
  fn test_issue_category_all() {
    let all = IssueCategory::all();
    assert_eq!(all.len(), 11);
  }

  // =========================================================================
  // QualityIssueReport Tests
  // =========================================================================

  #[test]
  fn test_quality_issue_report_new() {
    let issue = QualityIssueReport::new(
      IssueCategory::MissingTests,
      8,
      "test_field",
      "Missing test coverage",
    );
    assert_eq!(issue.category, IssueCategory::MissingTests);
    assert_eq!(issue.severity, 8);
    assert_eq!(issue.field, "test_field");
    assert_eq!(issue.description, "Missing test coverage");
    assert!(issue.context.is_none());
  }

  #[test]
  fn test_quality_issue_report_with_context() {
    let issue = QualityIssueReport::new(IssueCategory::MissingTests, 8, "field", "issue")
      .with_context("Additional context");
    assert_eq!(issue.context, Some("Additional context".to_string()));
  }

  // =========================================================================
  // suggest_improvements Tests
  // =========================================================================

  #[test]
  fn test_suggest_improvements_empty_report() {
    // Use a report with score 70 to avoid the completeness threshold suggestions
    let report = QualityReport::with_scores(70, 0, 0);
    let suggestions = suggest_improvements(&report);
    // Empty report with good score should have no suggestions
    assert!(suggestions.is_empty());
  }

  #[test]
  fn test_suggest_improvements_with_issues() {
    let report = create_report_with_issues();
    let suggestions = suggest_improvements(&report);
    // Should have suggestions for various issues
    assert!(!suggestions.is_empty());
  }

  #[test]
  fn test_suggest_improvements_sorted_by_priority() {
    let mut report = create_test_report();
    report.missing_auth_tests = vec!["auth".to_string()]; // Priority 10
    report.missing_edge_cases = vec!["edge".to_string()]; // Priority 7
    report.behaviors_without_examples = vec!["example".to_string()]; // Priority 6

    let suggestions = suggest_improvements(&report);
    // Verify sorted by priority (highest first)
    for i in 1..suggestions.len() {
      assert!(suggestions[i - 1].priority >= suggestions[i].priority);
    }
  }

  // =========================================================================
  // suggest_missing_tests Tests
  // =========================================================================

  #[test]
  fn test_suggest_missing_tests_error_tests() {
    let mut report = create_test_report();
    report.missing_error_tests = vec!["user_input".to_string()];

    let suggestions = suggest_missing_tests(&report);
    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].category, "testing");
    assert!(suggestions[0].description.contains("error handling"));
    assert!(suggestions[0].description.contains("user_input"));
    assert_eq!(suggestions[0].priority, 9);
  }

  #[test]
  fn test_suggest_missing_tests_auth_tests() {
    let mut report = create_test_report();
    report.missing_auth_tests = vec!["admin".to_string()];

    let suggestions = suggest_missing_tests(&report);
    assert_eq!(suggestions.len(), 1);
    assert!(suggestions[0].description.contains("authentication"));
    assert_eq!(suggestions[0].priority, 10);
  }

  #[test]
  fn test_suggest_missing_tests_edge_cases() {
    let mut report = create_test_report();
    report.missing_edge_cases = vec!["boundary".to_string()];

    let suggestions = suggest_missing_tests(&report);
    assert_eq!(suggestions.len(), 1);
    assert!(suggestions[0].description.contains("edge case"));
    assert_eq!(suggestions[0].priority, 7);
  }

  #[test]
  fn test_suggest_missing_tests_unverified_behaviors() {
    let mut report = create_test_report();
    report.unverified_behaviors = vec!["process_data".to_string()];

    let suggestions = suggest_missing_tests(&report);
    assert_eq!(suggestions.len(), 1);
    assert!(suggestions[0].description.contains("verification"));
    assert_eq!(suggestions[0].priority, 8);
  }

  #[test]
  fn test_suggest_missing_tests_low_testability_issue() {
    let mut report = create_test_report();
    report.add_issue(QualityIssueReport::new(
      IssueCategory::LowTestability,
      6,
      "field",
      "No acceptance criteria",
    ));

    let suggestions = suggest_missing_tests(&report);
    assert_eq!(suggestions.len(), 1);
    assert!(suggestions[0].description.contains("testability"));
  }

  // =========================================================================
  // suggest_vague_rules_improvements Tests
  // =========================================================================

  #[test]
  fn test_suggest_vague_rules_performance() {
    let mut report = create_test_report();
    report.vague_rules = vec!["system should be fast".to_string()];

    let suggestions = suggest_vague_rules_improvements(&report);
    assert_eq!(suggestions.len(), 1);
    assert!(suggestions[0].description.contains("performance"));
    assert!(suggestions[0].suggested_action.contains("200ms"));
  }

  #[test]
  fn test_suggest_vague_rules_quality() {
    let mut report = create_test_report();
    report.vague_rules = vec!["provide good error handling".to_string()];

    let suggestions = suggest_vague_rules_improvements(&report);
    assert_eq!(suggestions.len(), 1);
    assert!(suggestions[0].description.contains("quality"));
  }

  #[test]
  fn test_suggest_vague_rules_scope() {
    let mut report = create_test_report();
    report.vague_rules = vec!["support various inputs".to_string()];

    let suggestions = suggest_vague_rules_improvements(&report);
    assert_eq!(suggestions.len(), 1);
    assert!(suggestions[0].description.contains("scope"));
  }

  #[test]
  fn test_suggest_vague_rules_modal() {
    let mut report = create_test_report();
    report.vague_rules = vec!["should validate input".to_string()];

    let suggestions = suggest_vague_rules_improvements(&report);
    assert_eq!(suggestions.len(), 1);
    assert!(suggestions[0].description.contains("requirement strength"));
  }

  #[test]
  fn test_suggest_vague_rules_etc() {
    let mut report = create_test_report();
    report.vague_rules = vec!["support txt, pdf, etc".to_string()];

    let suggestions = suggest_vague_rules_improvements(&report);
    assert_eq!(suggestions.len(), 1);
    assert!(suggestions[0].description.contains("list"));
  }

  #[test]
  fn test_suggest_vague_rules_generic() {
    let mut report = create_test_report();
    report.vague_rules = vec!["handle user requests".to_string()];

    let suggestions = suggest_vague_rules_improvements(&report);
    assert_eq!(suggestions.len(), 1);
    assert!(suggestions[0].description.contains("specificity"));
  }

  #[test]
  fn test_suggest_vague_rules_with_low_clarity_issue() {
    let mut report = create_test_report();
    report.add_issue(
      QualityIssueReport::new(IssueCategory::LowClarity, 7, "field", "Unclear requirement")
        .with_context("Needs clarification"),
    );

    let suggestions = suggest_vague_rules_improvements(&report);
    assert_eq!(suggestions.len(), 1);
    assert!(suggestions[0].description.contains("Clarify"));
  }

  // =========================================================================
  // suggest_examples_improvements Tests
  // =========================================================================

  #[test]
  fn test_suggest_examples_missing_examples() {
    let mut report = create_test_report();
    report.behaviors_without_examples = vec!["calculate_price".to_string()];

    let suggestions = suggest_examples_improvements(&report);
    assert_eq!(suggestions.len(), 1);
    assert!(suggestions[0].description.contains("example"));
    assert!(suggestions[0].description.contains("calculate_price"));
    assert_eq!(suggestions[0].priority, 6);
  }

  #[test]
  fn test_suggest_examples_with_low_completeness_issue() {
    let mut report = create_test_report();
    report.add_issue(
      QualityIssueReport::new(
        IssueCategory::LowCompleteness,
        8,
        "field",
        "Missing description",
      )
      .with_context("Add details"),
    );

    let suggestions = suggest_examples_improvements(&report);
    assert_eq!(suggestions.len(), 1);
    assert!(suggestions[0].description.contains("missing content"));
  }

  // =========================================================================
  // Security Improvements Tests
  // =========================================================================

  #[test]
  fn test_suggest_security_low_security_issue() {
    let mut report = create_test_report();
    report.add_issue(
      QualityIssueReport::new(IssueCategory::LowSecurity, 10, "api", "No input validation")
        .with_context("SQL injection risk"),
    );

    let suggestions = suggest_improvements(&report);
    let security_suggestions: Vec<_> = suggestions
      .iter()
      .filter(|s| s.category == "security")
      .collect();
    assert!(!security_suggestions.is_empty());
  }

  #[test]
  fn test_suggest_security_missing_auth() {
    let mut report = create_test_report();
    report.missing_auth_tests = vec!["api_endpoint".to_string()];

    let suggestions = suggest_improvements(&report);
    let security_suggestions: Vec<_> = suggestions
      .iter()
      .filter(|s| s.category == "security")
      .collect();
    assert!(!security_suggestions.is_empty());
  }

  // =========================================================================
  // Completeness Improvements Tests
  // =========================================================================

  #[test]
  fn test_suggest_completeness_critical_score() {
    let report = QualityReport::with_scores(40, 10, 2);

    let suggestions = suggest_improvements(&report);
    let completeness: Vec<_> = suggestions
      .iter()
      .filter(|s| s.category == "completeness" && s.affected_field == "overall")
      .collect();
    assert!(!completeness.is_empty());
    assert!(completeness.iter().any(|s| s.priority == 10));
  }

  #[test]
  fn test_suggest_completeness_moderate_score() {
    let report = QualityReport::with_scores(60, 10, 2);

    let suggestions = suggest_improvements(&report);
    let completeness: Vec<_> = suggestions
      .iter()
      .filter(|s| s.category == "completeness" && s.affected_field == "overall")
      .collect();
    assert!(!completeness.is_empty());
  }

  #[test]
  fn test_suggest_completeness_many_unverified() {
    let mut report = QualityReport::with_scores(70, 10, 2);
    report.unverified_behaviors = vec![
      "b1".to_string(),
      "b2".to_string(),
      "b3".to_string(),
      "b4".to_string(),
      "b5".to_string(),
      "b6".to_string(),
    ];

    let suggestions = suggest_improvements(&report);
    let verification: Vec<_> = suggestions
      .iter()
      .filter(|s| s.affected_field == "verification")
      .collect();
    assert!(!verification.is_empty());
  }

  // =========================================================================
  // Consistency Improvements Tests
  // =========================================================================

  #[test]
  fn test_suggest_consistency_issue() {
    let mut report = create_test_report();
    report.add_issue(
      QualityIssueReport::new(
        IssueCategory::LowConsistency,
        8,
        "requirements",
        "Contradictory requirements found",
      )
      .with_context("Must vs must not"),
    );

    let suggestions = suggest_improvements(&report);
    let consistency: Vec<_> = suggestions
      .iter()
      .filter(|s| s.category == "consistency")
      .collect();
    assert!(!consistency.is_empty());
  }

  // =========================================================================
  // Clarity Improvements Tests
  // =========================================================================

  #[test]
  fn test_suggest_clarity_many_vague_rules() {
    let mut report = create_test_report();
    report.vague_rules = vec![
      "rule1".to_string(),
      "rule2".to_string(),
      "rule3".to_string(),
      "rule4".to_string(),
    ];

    let suggestions = suggest_improvements(&report);
    let glossary: Vec<_> = suggestions
      .iter()
      .filter(|s| s.affected_field == "documentation" && s.description.contains("glossary"))
      .collect();
    assert!(!glossary.is_empty());
  }

  // =========================================================================
  // Integration Tests
  // =========================================================================

  #[test]
  fn test_full_improvement_workflow() {
    let mut report = QualityReport::with_scores(55, 20, 5);
    report.missing_error_tests = vec!["api_handler".to_string()];
    report.missing_auth_tests = vec!["admin_routes".to_string()];
    report.missing_edge_cases = vec!["input_validation".to_string()];
    report.unverified_behaviors = vec!["process_order".to_string(), "validate_payment".to_string()];
    report.behaviors_without_examples = vec!["calculate_tax".to_string()];
    report.vague_rules = vec!["system should respond quickly".to_string()];
    report.add_issue(QualityIssueReport::new(
      IssueCategory::LowSecurity,
      10,
      "auth",
      "No rate limiting",
    ));

    let suggestions = suggest_improvements(&report);

    // Should have multiple suggestions
    assert!(suggestions.len() > 5);

    // Should be sorted by priority
    for i in 1..suggestions.len() {
      assert!(suggestions[i - 1].priority >= suggestions[i].priority);
    }

    // Should cover multiple categories
    let categories: std::collections::HashSet<_> =
      suggestions.iter().map(|s| s.category.as_str()).collect();
    assert!(categories.contains("testing"));
    assert!(categories.contains("clarity"));
    assert!(categories.contains("completeness"));
  }

  #[test]
  fn test_improvement_suggestion_serialization() {
    let suggestion =
      ImprovementSuggestion::new("testing", "Add test", 8, "field", "Create test").expect("valid");

    let json = serde_json::to_string(&suggestion).expect("should serialize");
    let parsed: ImprovementSuggestion = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(suggestion, parsed);
  }

  #[test]
  fn test_quality_report_serialization() {
    let mut report = QualityReport::with_scores(75, 10, 3);
    report.missing_error_tests = vec!["test".to_string()];
    report.add_issue(QualityIssueReport::new(
      IssueCategory::MissingTests,
      8,
      "field",
      "issue",
    ));

    let json = serde_json::to_string(&report).expect("should serialize");
    let parsed: QualityReport = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(report, parsed);
  }

  // =========================================================================
  // Boundary Tests
  // =========================================================================

  #[test]
  fn test_priority_boundary_high_medium() {
    let high = ImprovementSuggestion::new("cat", "desc", 8, "field", "action").expect("valid");
    let medium = ImprovementSuggestion::new("cat", "desc", 7, "field", "action").expect("valid");

    assert!(high.is_high_priority());
    assert!(!high.is_medium_priority());
    assert!(!medium.is_high_priority());
    assert!(medium.is_medium_priority());
  }

  #[test]
  fn test_priority_boundary_medium_low() {
    let medium = ImprovementSuggestion::new("cat", "desc", 4, "field", "action").expect("valid");
    let low = ImprovementSuggestion::new("cat", "desc", 3, "field", "action").expect("valid");

    assert!(medium.is_medium_priority());
    assert!(!medium.is_low_priority());
    assert!(!low.is_medium_priority());
    assert!(low.is_low_priority());
  }

  #[test]
  fn test_critical_issue_boundary() {
    let mut report = create_test_report();

    report.add_issue(QualityIssueReport::new(
      IssueCategory::MissingTests,
      7,
      "field",
      "not critical",
    ));
    assert!(!report.has_critical_issues());

    report.add_issue(QualityIssueReport::new(
      IssueCategory::MissingTests,
      8,
      "field",
      "critical",
    ));
    assert!(report.has_critical_issues());
  }

  // =========================================================================
  // Empty Input Tests
  // =========================================================================

  #[test]
  fn test_suggest_missing_tests_empty_report() {
    let report = create_test_report();
    let suggestions = suggest_missing_tests(&report);
    assert!(suggestions.is_empty());
  }

  #[test]
  fn test_suggest_vague_rules_empty_report() {
    let report = create_test_report();
    let suggestions = suggest_vague_rules_improvements(&report);
    assert!(suggestions.is_empty());
  }

  #[test]
  fn test_suggest_examples_empty_report() {
    let report = create_test_report();
    let suggestions = suggest_examples_improvements(&report);
    assert!(suggestions.is_empty());
  }

  // =========================================================================
  // analyze_vague_rule Tests
  // =========================================================================

  #[test]
  fn test_analyze_vague_rule_fast() {
    let (desc, action) = analyze_vague_rule("must be fast");
    assert!(desc.to_lowercase().contains("performance"));
    assert!(action.contains("200ms"));
  }

  #[test]
  fn test_analyze_vague_rule_good() {
    let (desc, action) = analyze_vague_rule("provide good quality");
    assert!(desc.to_lowercase().contains("quality"));
    assert!(action.contains("0.1%"));
  }

  #[test]
  fn test_analyze_vague_rule_some() {
    let (desc, _action) = analyze_vague_rule("handle some cases");
    assert!(desc.to_lowercase().contains("scope"));
  }

  #[test]
  fn test_analyze_vague_rule_should() {
    let (desc, _action) = analyze_vague_rule("should work");
    assert!(desc.to_lowercase().contains("requirement"));
  }

  #[test]
  fn test_analyze_vague_rule_etc() {
    let (desc, _action) = analyze_vague_rule("items, etc");
    assert!(desc.to_lowercase().contains("list"));
  }

  #[test]
  fn test_analyze_vague_rule_generic() {
    let (desc, _action) = analyze_vague_rule("handle transactions");
    assert!(desc.to_lowercase().contains("specificity"));
  }

  // =========================================================================
  // Mutant Catching Tests
  // =========================================================================

  #[test]
  fn test_improvement_suggestion_priority_eight_is_high() {
    // Boundary: priority 8 should be high
    let s = ImprovementSuggestion::new("cat", "desc", 8, "field", "action").expect("valid");
    assert!(s.is_high_priority(), "Priority 8 must be high priority");
  }

  #[test]
  fn test_improvement_suggestion_priority_seven_is_medium() {
    // Boundary: priority 7 should be medium
    let s = ImprovementSuggestion::new("cat", "desc", 7, "field", "action").expect("valid");
    assert!(s.is_medium_priority(), "Priority 7 must be medium priority");
    assert!(
      !s.is_high_priority(),
      "Priority 7 must not be high priority"
    );
  }

  #[test]
  fn test_improvement_suggestion_priority_four_is_medium() {
    // Boundary: priority 4 should be medium
    let s = ImprovementSuggestion::new("cat", "desc", 4, "field", "action").expect("valid");
    assert!(s.is_medium_priority(), "Priority 4 must be medium priority");
    assert!(!s.is_low_priority(), "Priority 4 must not be low priority");
  }

  #[test]
  fn test_improvement_suggestion_priority_three_is_low() {
    // Boundary: priority 3 should be low
    let s = ImprovementSuggestion::new("cat", "desc", 3, "field", "action").expect("valid");
    assert!(s.is_low_priority(), "Priority 3 must be low priority");
    assert!(
      !s.is_medium_priority(),
      "Priority 3 must not be medium priority"
    );
  }

  #[test]
  fn test_critical_issue_severity_eight() {
    // Boundary: severity 8 is critical
    let mut report = create_test_report();
    report.add_issue(QualityIssueReport::new(
      IssueCategory::MissingTests,
      8,
      "field",
      "issue",
    ));
    assert!(report.has_critical_issues(), "Severity 8 must be critical");
  }

  #[test]
  fn test_non_critical_issue_severity_seven() {
    // Boundary: severity 7 is not critical
    let mut report = create_test_report();
    report.add_issue(QualityIssueReport::new(
      IssueCategory::MissingTests,
      7,
      "field",
      "issue",
    ));
    assert!(
      !report.has_critical_issues(),
      "Severity 7 must not be critical"
    );
  }

  #[test]
  fn test_suggest_improvements_returns_all_categories() {
    // Ensure all suggestion functions are called
    let mut report = QualityReport::with_scores(40, 10, 3);
    report.missing_error_tests = vec!["err".to_string()];
    report.missing_auth_tests = vec!["auth".to_string()];
    report.missing_edge_cases = vec!["edge".to_string()];
    report.unverified_behaviors = vec!["verify".to_string()];
    report.behaviors_without_examples = vec!["example".to_string()];
    report.vague_rules = vec!["vague".to_string()];
    report.add_issue(QualityIssueReport::new(
      IssueCategory::LowTestability,
      5,
      "f",
      "t",
    ));
    report.add_issue(QualityIssueReport::new(
      IssueCategory::LowClarity,
      5,
      "f",
      "c",
    ));
    report.add_issue(QualityIssueReport::new(
      IssueCategory::LowCompleteness,
      5,
      "f",
      "comp",
    ));
    report.add_issue(QualityIssueReport::new(
      IssueCategory::LowSecurity,
      5,
      "f",
      "sec",
    ));
    report.add_issue(QualityIssueReport::new(
      IssueCategory::LowConsistency,
      5,
      "f",
      "cons",
    ));

    let suggestions = suggest_improvements(&report);

    let categories: std::collections::HashSet<_> =
      suggestions.iter().map(|s| s.category.as_str()).collect();

    assert!(
      categories.contains("testing"),
      "Should have testing suggestions"
    );
    assert!(
      categories.contains("clarity"),
      "Should have clarity suggestions"
    );
    assert!(
      categories.contains("completeness"),
      "Should have completeness suggestions"
    );
    assert!(
      categories.contains("security"),
      "Should have security suggestions"
    );
    assert!(
      categories.contains("consistency"),
      "Should have consistency suggestions"
    );
  }

  #[test]
  fn test_suggestions_sorted_correctly() {
    let mut report = create_test_report();
    report.missing_auth_tests = vec!["auth".to_string()]; // Priority 10
    report.missing_error_tests = vec!["err".to_string()]; // Priority 9
    report.unverified_behaviors = vec!["verify".to_string()]; // Priority 8
    report.missing_edge_cases = vec!["edge".to_string()]; // Priority 7
    report.behaviors_without_examples = vec!["example".to_string()]; // Priority 6

    let suggestions = suggest_improvements(&report);

    // Verify strictly descending order
    for window in suggestions.windows(2) {
      assert!(
        window[0].priority >= window[1].priority,
        "Suggestions must be sorted by priority descending: {window:?}"
      );
    }
  }

  #[test]
  fn test_completeness_threshold_forty_nine() {
    // Score 49 triggers critical (under 50)
    let report = QualityReport::with_scores(49, 10, 2);
    let suggestions = suggest_improvements(&report);
    let critical: Vec<_> = suggestions
      .iter()
      .filter(|s| s.category == "completeness" && s.priority == 10)
      .collect();
    assert!(
      !critical.is_empty(),
      "Score 49 should trigger critical priority"
    );
  }

  #[test]
  fn test_completeness_threshold_fifty() {
    // Score 50 does not trigger critical (not under 50)
    let report = QualityReport::with_scores(50, 10, 2);
    let suggestions = suggest_improvements(&report);
    let critical: Vec<_> = suggestions
      .iter()
      .filter(|s| s.category == "completeness" && s.priority == 10 && s.affected_field == "overall")
      .collect();
    assert!(
      critical.is_empty(),
      "Score 50 should not trigger critical priority"
    );
  }

  #[test]
  fn test_completeness_threshold_sixty_nine() {
    // Score 69 triggers moderate (under 70)
    let report = QualityReport::with_scores(69, 10, 2);
    let suggestions = suggest_improvements(&report);
    let moderate: Vec<_> = suggestions
      .iter()
      .filter(|s| s.category == "completeness" && s.priority == 8 && s.affected_field == "overall")
      .collect();
    assert!(
      !moderate.is_empty(),
      "Score 69 should trigger moderate priority"
    );
  }

  #[test]
  fn test_completeness_threshold_seventy() {
    // Score 70 does not trigger moderate (not under 70)
    let report = QualityReport::with_scores(70, 10, 2);
    let suggestions = suggest_improvements(&report);
    let moderate: Vec<_> = suggestions
      .iter()
      .filter(|s| s.category == "completeness" && s.affected_field == "overall")
      .collect();
    assert!(
      moderate.is_empty(),
      "Score 70 should not trigger completeness suggestion"
    );
  }

  #[test]
  fn test_verification_threshold_exactly_half() {
    // Exactly half should NOT trigger the warning (> half, not >= half)
    let mut report = QualityReport::with_scores(70, 10, 2);
    report.unverified_behaviors = vec![
      "b1".to_string(),
      "b2".to_string(),
      "b3".to_string(),
      "b4".to_string(),
      "b5".to_string(),
    ]; // 5 of 10 = exactly half

    let suggestions = suggest_improvements(&report);
    let verification: Vec<_> = suggestions
      .iter()
      .filter(|s| s.affected_field == "verification" && s.description.contains("half"))
      .collect();
    assert!(
      verification.is_empty(),
      "Exactly half should not trigger warning"
    );
  }

  #[test]
  fn test_verification_threshold_more_than_half() {
    // More than half should trigger
    let mut report = QualityReport::with_scores(70, 10, 2);
    report.unverified_behaviors = vec![
      "b1".to_string(),
      "b2".to_string(),
      "b3".to_string(),
      "b4".to_string(),
      "b5".to_string(),
      "b6".to_string(),
    ]; // 6 of 10 = more than half

    let suggestions = suggest_improvements(&report);
    let verification: Vec<_> = suggestions
      .iter()
      .filter(|s| s.affected_field == "verification" && s.description.contains("half"))
      .collect();
    assert!(
      !verification.is_empty(),
      "More than half should trigger warning"
    );
  }

  #[test]
  fn test_glossary_threshold_exactly_three() {
    // Exactly 3 vague rules should NOT trigger (> 3, not >= 3)
    let mut report = create_test_report();
    report.vague_rules = vec![
      "rule1".to_string(),
      "rule2".to_string(),
      "rule3".to_string(),
    ];

    let suggestions = suggest_improvements(&report);
    let glossary: Vec<_> = suggestions
      .iter()
      .filter(|s| s.affected_field == "documentation" && s.description.contains("glossary"))
      .collect();
    assert!(
      glossary.is_empty(),
      "Exactly 3 rules should not trigger glossary suggestion"
    );
  }

  #[test]
  fn test_glossary_threshold_four() {
    // 4 vague rules should trigger
    let mut report = create_test_report();
    report.vague_rules = vec![
      "rule1".to_string(),
      "rule2".to_string(),
      "rule3".to_string(),
      "rule4".to_string(),
    ];

    let suggestions = suggest_improvements(&report);
    let glossary: Vec<_> = suggestions
      .iter()
      .filter(|s| s.affected_field == "documentation" && s.description.contains("glossary"))
      .collect();
    assert!(
      !glossary.is_empty(),
      "4 rules should trigger glossary suggestion"
    );
  }
}
