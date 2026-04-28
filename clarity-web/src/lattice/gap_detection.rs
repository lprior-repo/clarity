#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Gap Detection module for requirements analysis.
//!
//! This module identifies missing or incomplete areas in requirements
//! by analyzing coverage across multiple dimensions.

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Domain errors for gap detection
#[derive(Debug, Error, PartialEq, Clone)]
pub enum GapError {
  #[error("requirements list is empty")]
  EmptyRequirements,

  #[error("invalid gap threshold: {0}")]
  InvalidThreshold(f32),

  #[error("category not found: {0}")]
  CategoryNotFound(String),
}

/// Categories of potential gaps
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GapCategory {
  /// Missing functional requirements
  Functional,
  /// Missing non-functional requirements
  NonFunctional,
  /// Missing edge cases
  EdgeCases,
  /// Missing error handling
  ErrorHandling,
  /// Missing security requirements
  Security,
  /// Missing performance requirements
  Performance,
  /// Missing user interaction
  UserInteraction,
  /// Missing data requirements
  Data,
  /// Missing integration requirements
  Integration,
  /// Missing constraints
  Constraints,
}

impl GapCategory {
  /// Get all categories
  #[must_use]
  pub const fn all() -> [Self; 10] {
    [
      Self::Functional,
      Self::NonFunctional,
      Self::EdgeCases,
      Self::ErrorHandling,
      Self::Security,
      Self::Performance,
      Self::UserInteraction,
      Self::Data,
      Self::Integration,
      Self::Constraints,
    ]
  }

  /// Get label
  #[must_use]
  pub const fn label(self) -> &'static str {
    match self {
      Self::Functional => "Functional",
      Self::NonFunctional => "Non-Functional",
      Self::EdgeCases => "Edge Cases",
      Self::ErrorHandling => "Error Handling",
      Self::Security => "Security",
      Self::Performance => "Performance",
      Self::UserInteraction => "User Interaction",
      Self::Data => "Data",
      Self::Integration => "Integration",
      Self::Constraints => "Constraints",
    }
  }

  /// Get indicators that suggest this category is covered
  #[must_use]
  pub const fn coverage_indicators(&self) -> &[&str] {
    match self {
      Self::Functional => &["shall", "must", "will", "function", "feature"],
      Self::NonFunctional => &["scalability", "availability", "maintainability"],
      Self::EdgeCases => &["when", "if", "except", "boundary", "limit"],
      Self::ErrorHandling => &["error", "exception", "failure", "fallback", "retry"],
      Self::Security => &["auth", "encrypt", "secure", "access control", "permission"],
      Self::Performance => &["performance", "latency", "throughput", "response time"],
      Self::UserInteraction => &["user", "interface", "ui", "ux", "display"],
      Self::Data => &["data", "store", "retrieve", "database", "record"],
      Self::Integration => &["api", "integration", "external", "service", "connect"],
      Self::Constraints => &["must not", "cannot", "limited", "maximum", "minimum"],
    }
  }

  /// Get suggested questions to fill this gap
  #[must_use]
  pub const fn suggested_questions(&self) -> &[&str] {
    match self {
      Self::Functional => &[
        "What are the core functions?",
        "What inputs and outputs are expected?",
      ],
      Self::NonFunctional => &[
        "What are the scalability requirements?",
        "What availability is required?",
      ],
      Self::EdgeCases => &[
        "What happens at boundaries?",
        "What if limits are exceeded?",
      ],
      Self::ErrorHandling => &[
        "What errors could occur?",
        "How should failures be handled?",
      ],
      Self::Security => &[
        "What authentication is needed?",
        "What data must be protected?",
      ],
      Self::Performance => &[
        "What response time is acceptable?",
        "What throughput is required?",
      ],
      Self::UserInteraction => &["How will users interact?", "What feedback do users need?"],
      Self::Data => &["What data is stored?", "What data validation is needed?"],
      Self::Integration => &[
        "What external systems are involved?",
        "What APIs are needed?",
      ],
      Self::Constraints => &["What are the technical limits?", "What cannot be done?"],
    }
  }
}

/// Severity of a detected gap
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum GapSeverity {
  /// Minor gap, nice to have
  Low,
  /// Moderate gap, should address
  Medium,
  /// Significant gap, must address
  High,
  /// Critical gap, blocking
  Critical,
}

impl GapSeverity {
  /// Convert to numeric score
  #[must_use]
  pub const fn score(self) -> u8 {
    match self {
      Self::Low => 10,
      Self::Medium => 30,
      Self::High => 60,
      Self::Critical => 100,
    }
  }
}

/// A detected gap in requirements
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectedGap {
  /// Unique identifier
  pub id: String,
  /// Gap category
  pub category: GapCategory,
  /// Gap severity
  pub severity: GapSeverity,
  /// Description of the gap
  pub description: String,
  /// Evidence of missing coverage
  pub evidence: String,
  /// Suggested questions to fill the gap
  pub suggestions: Vec<String>,
  /// Related requirements that partially cover this area
  pub related_requirements: Vec<String>,
}

impl DetectedGap {
  /// Create a new detected gap
  #[must_use]
  pub fn new(
    id: String,
    category: GapCategory,
    severity: GapSeverity,
    description: String,
  ) -> Self {
    let suggestions = category
      .suggested_questions()
      .iter()
      .map(std::string::ToString::to_string)
      .collect();

    Self {
      id,
      category,
      severity,
      description,
      evidence: String::new(),
      suggestions,
      related_requirements: Vec::new(),
    }
  }

  /// Add evidence using builder pattern
  #[must_use]
  pub fn with_evidence(mut self, evidence: String) -> Self {
    self.evidence = evidence;
    self
  }

  /// Add related requirement
  #[must_use]
  pub fn with_related(mut self, requirement: String) -> Self {
    self.related_requirements.push(requirement);
    self
  }
}

/// Complete gap analysis result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GapAnalysis {
  /// All detected gaps
  pub gaps: Vec<DetectedGap>,
  /// Coverage score per category (0-100)
  pub category_coverage: Vec<(GapCategory, u8)>,
  /// Overall coverage score
  pub overall_coverage: u8,
  /// Categories with no gaps
  pub covered_categories: Vec<GapCategory>,
  /// Categories with gaps
  pub gap_categories: Vec<GapCategory>,
  /// Summary message
  pub summary: String,
}

impl GapAnalysis {
  /// Create new gap analysis
  #[must_use]
  pub fn new(gaps: Vec<DetectedGap>, category_coverage: Vec<(GapCategory, u8)>) -> Self {
    let overall_coverage = calculate_overall_coverage(&category_coverage);

    let covered_categories: Vec<GapCategory> = category_coverage
      .iter()
      .filter(|(_, score)| *score >= 80)
      .map(|(cat, _)| *cat)
      .collect();

    let gap_categories: Vec<GapCategory> = category_coverage
      .iter()
      .filter(|(_, score)| *score < 80)
      .map(|(cat, _)| *cat)
      .collect();

    let summary = generate_gap_summary(&gaps, overall_coverage);

    Self {
      gaps,
      category_coverage,
      overall_coverage,
      covered_categories,
      gap_categories,
      summary,
    }
  }

  /// Get gaps by category
  #[must_use]
  pub fn gaps_by_category(&self, category: GapCategory) -> Vec<&DetectedGap> {
    self
      .gaps
      .iter()
      .filter(|g| g.category == category)
      .collect()
  }

  /// Get gaps by severity
  #[must_use]
  pub fn gaps_by_severity(&self, severity: GapSeverity) -> Vec<&DetectedGap> {
    self
      .gaps
      .iter()
      .filter(|g| g.severity == severity)
      .collect()
  }

  /// Get critical gaps
  #[must_use]
  pub fn critical_gaps(&self) -> Vec<&DetectedGap> {
    self.gaps_by_severity(GapSeverity::Critical)
  }

  /// Get high severity gaps
  #[must_use]
  pub fn high_severity_gaps(&self) -> Vec<&DetectedGap> {
    self.gaps_by_severity(GapSeverity::High)
  }

  /// Check if any critical gaps exist
  #[must_use]
  pub fn has_critical_gaps(&self) -> bool {
    self
      .gaps
      .iter()
      .any(|g| g.severity == GapSeverity::Critical)
  }

  /// Get prioritized gaps (sorted by severity)
  #[must_use]
  pub fn prioritized_gaps(&self) -> Vec<&DetectedGap> {
    self
      .gaps
      .iter()
      .sorted_by(|a, b| b.severity.cmp(&a.severity))
      .collect()
  }

  /// Get coverage for a specific category
  #[must_use]
  pub fn get_coverage(&self, category: GapCategory) -> Option<u8> {
    self
      .category_coverage
      .iter()
      .find(|(c, _)| *c == category)
      .map(|(_, score)| *score)
  }
}

/// Calculate overall coverage from category scores
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn calculate_overall_coverage(category_coverage: &[(GapCategory, u8)]) -> u8 {
  if category_coverage.is_empty() {
    return 0;
  }

  let total: u32 = category_coverage
    .iter()
    .map(|(_, score)| u32::from(*score))
    .sum();
  (total / category_coverage.len() as u32) as u8
}

/// Generate gap analysis summary
fn generate_gap_summary(gaps: &[DetectedGap], overall_coverage: u8) -> String {
  let critical = gaps
    .iter()
    .filter(|g| g.severity == GapSeverity::Critical)
    .count();
  let high = gaps
    .iter()
    .filter(|g| g.severity == GapSeverity::High)
    .count();
  let medium = gaps
    .iter()
    .filter(|g| g.severity == GapSeverity::Medium)
    .count();

  format!(
    "Coverage: {overall_coverage}% | Gaps: {critical} critical, {high} high, {medium} medium priority"
  )
}

/// Analyze requirements for gaps
///
/// # Arguments
/// * `requirements` - List of requirement texts to analyze
///
/// # Returns
/// Complete gap analysis with detected gaps and coverage scores
#[must_use]
pub fn detect_gaps(requirements: &[&str]) -> GapAnalysis {
  let (category_coverage, gaps, _) = GapCategory::all().iter().fold(
    (Vec::new(), Vec::new(), 0usize),
    |(mut coverage, mut gaps, mut gap_id), category| {
      let (cov, cat_gaps) = analyze_category(*category, requirements, &mut gap_id);
      coverage.push((*category, cov));
      gaps.extend(cat_gaps);
      (coverage, gaps, gap_id)
    },
  );

  GapAnalysis::new(gaps, category_coverage)
}

/// Analyze a single category for gaps
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_sign_loss)]
fn analyze_category(
  category: GapCategory,
  requirements: &[&str],
  gap_id: &mut usize,
) -> (u8, Vec<DetectedGap>) {
  let indicators = category.coverage_indicators();

  // Count unique requirements that have at least one indicator
  // Bug fix: Previously counted ALL indicator matches, not unique requirements
  let requirements_with_indicators = requirements
    .iter()
    .filter(|req| {
      let lower = req.to_lowercase();
      indicators.iter().any(|indicator| lower.contains(indicator))
    })
    .count();

  // Calculate coverage score based on unique requirements, not indicator count
  let coverage = if indicators.is_empty() || requirements.is_empty() {
    0
  } else {
    let ratio = requirements_with_indicators as f64 / requirements.len() as f64;
    (ratio * 100.0).min(100.0).round() as u8
  };

  // Determine severity based on coverage
  let severity = match coverage {
    0..=20 => GapSeverity::Critical,
    21..=40 => GapSeverity::High,
    41..=60 => GapSeverity::Medium,
    _ => GapSeverity::Low,
  };

  let mut gaps = Vec::new();

  // Create gap if coverage is low
  if coverage < 80 {
    *gap_id += 1;
    let gap = DetectedGap::new(
      format!("GAP-{gap_id:03}"),
      category,
      severity,
      format!("Missing {} requirements", category.label()),
    )
    .with_evidence(format!(
      "Found {} of {} expected indicators",
      requirements_with_indicators,
      indicators.len()
    ));

    gaps.push(gap);
  }

  (coverage, gaps)
}

/// Quick gap check for a specific category
///
/// # Arguments
/// * `requirements` - Requirements to check
/// * `category` - Category to check
///
/// # Returns
/// Coverage score for the category (0-100)
#[must_use]
pub fn check_category_coverage(requirements: &[&str], category: GapCategory) -> u8 {
  let (coverage, _) = analyze_category(category, requirements, &mut 0);
  coverage
}

/// Get missing requirement areas
///
/// # Arguments
/// * `requirements` - Requirements to analyze
///
/// # Returns
/// List of categories with low coverage
#[must_use]
pub fn get_missing_areas(requirements: &[&str]) -> Vec<(GapCategory, u8)> {
  GapCategory::all()
    .iter()
    .map(|category| {
      let coverage = check_category_coverage(requirements, *category);
      (*category, coverage)
    })
    .filter(|(_, coverage)| *coverage < 80)
    .sorted_by(|a, b| a.1.cmp(&b.1))
    .collect()
}

/// Generate requirements template based on detected gaps
///
/// # Arguments
/// * `analysis` - Gap analysis result
///
/// # Returns
/// Template string with suggested requirements
#[must_use]
pub fn generate_requirements_template(analysis: &GapAnalysis) -> String {
  let mut template = String::new();
  template.push_str("# Requirements Template\n\n");
  template.push_str("Based on gap analysis, consider adding:\n\n");

  template.push_str(
    &analysis
      .prioritized_gaps()
      .iter()
      .map(|gap| {
        let suggestions = gap
          .suggestions
          .iter()
          .fold(String::new(), |mut acc, s| {
            use std::fmt::Write;
            let _ = writeln!(acc, "- [ ] {s}");
            acc
          });
        format!(
          "## {} ({:?})\n\n{}\n{}",
          gap.category.label(),
          gap.severity,
          gap.description,
          suggestions
        )
      })
      .collect::<Vec<_>>()
      .join("\n"),
  );

  template
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
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro,
  clippy::needless_pass_by_value
)]
mod tests {
  use super::*;

  #[test]
  fn test_gap_category_labels() {
    assert_eq!(GapCategory::Functional.label(), "Functional");
    assert_eq!(GapCategory::Security.label(), "Security");
  }

  #[test]
  fn test_gap_category_coverage_indicators() {
    for category in GapCategory::all() {
      let indicators = category.coverage_indicators();
      assert!(!indicators.is_empty());
    }
  }

  #[test]
  fn test_gap_category_suggested_questions() {
    for category in GapCategory::all() {
      let questions = category.suggested_questions();
      assert!(!questions.is_empty());
    }
  }

  #[test]
  fn test_gap_severity_ordering() {
    assert!(GapSeverity::Critical > GapSeverity::High);
    assert!(GapSeverity::High > GapSeverity::Medium);
    assert!(GapSeverity::Medium > GapSeverity::Low);
  }

  #[test]
  fn test_gap_severity_scores() {
    assert_eq!(GapSeverity::Low.score(), 10);
    assert_eq!(GapSeverity::Medium.score(), 30);
    assert_eq!(GapSeverity::High.score(), 60);
    assert_eq!(GapSeverity::Critical.score(), 100);
  }

  #[test]
  fn test_detected_gap_new() {
    let gap = DetectedGap::new(
      "GAP-001".to_string(),
      GapCategory::Security,
      GapSeverity::High,
      "Missing security requirements".to_string(),
    );

    assert_eq!(gap.id, "GAP-001");
    assert_eq!(gap.category, GapCategory::Security);
    assert!(!gap.suggestions.is_empty());
  }

  #[test]
  fn test_detected_gap_builder() {
    let gap = DetectedGap::new(
      "GAP-001".to_string(),
      GapCategory::Functional,
      GapSeverity::Medium,
      "Test gap".to_string(),
    )
    .with_evidence("No functional requirements found".to_string())
    .with_related("req-001".to_string());

    assert!(!gap.evidence.is_empty());
    assert_eq!(gap.related_requirements.len(), 1);
  }

  #[test]
  fn test_detect_gaps_empty_requirements() {
    let analysis = detect_gaps(&[]);

    // All categories should have gaps
    assert!(!analysis.gaps.is_empty());
    assert_eq!(analysis.overall_coverage, 0);
  }

  #[test]
  fn test_detect_gaps_comprehensive_requirements() {
    let requirements = &[
      "The system shall authenticate users.",
      "User data must be encrypted.",
      "Response time shall be under 2 seconds.",
      "The system must handle errors gracefully.",
      "Users can input data through the UI.",
      "Data shall be stored in the database.",
      "API integration with external services.",
      "System cannot exceed 100 concurrent users.",
      "When the limit is reached, queue requests.",
      "The system must be available 99.9%.",
    ];

    let analysis = detect_gaps(requirements);

    // Should have reasonable coverage
    assert!(
      analysis.overall_coverage >= 10,
      "Coverage should be at least 10% with new logic"
    );
  }

  #[test]
  fn test_check_category_coverage_present() {
    let requirements = &["The system shall authenticate users."];

    let coverage = check_category_coverage(requirements, GapCategory::Functional);

    // Should have some functional coverage due to "shall"
    assert!(coverage > 0);
  }

  #[test]
  fn test_check_category_coverage_absent() {
    let requirements = &["Some random text without indicators."];

    let coverage = check_category_coverage(requirements, GapCategory::Security);

    // Should have low security coverage
    assert!(coverage < 50);
  }

  #[test]
  fn test_get_missing_areas() {
    let requirements = &["The system shall process data."];

    let missing = get_missing_areas(requirements);

    // Should identify some missing areas
    assert!(!missing.is_empty());
  }

  #[test]
  fn test_gap_analysis_gaps_by_category() {
    let gaps = vec![
      DetectedGap::new(
        "GAP-001".to_string(),
        GapCategory::Security,
        GapSeverity::High,
        "Gap 1".to_string(),
      ),
      DetectedGap::new(
        "GAP-002".to_string(),
        GapCategory::Security,
        GapSeverity::Medium,
        "Gap 2".to_string(),
      ),
      DetectedGap::new(
        "GAP-003".to_string(),
        GapCategory::Performance,
        GapSeverity::Low,
        "Gap 3".to_string(),
      ),
    ];

    let analysis = GapAnalysis::new(gaps, vec![]);

    let security_gaps = analysis.gaps_by_category(GapCategory::Security);
    assert_eq!(security_gaps.len(), 2);

    let performance_gaps = analysis.gaps_by_category(GapCategory::Performance);
    assert_eq!(performance_gaps.len(), 1);
  }

  #[test]
  fn test_gap_analysis_gaps_by_severity() {
    let gaps = vec![
      DetectedGap::new(
        "GAP-001".to_string(),
        GapCategory::Security,
        GapSeverity::Critical,
        "Gap 1".to_string(),
      ),
      DetectedGap::new(
        "GAP-002".to_string(),
        GapCategory::Security,
        GapSeverity::High,
        "Gap 2".to_string(),
      ),
      DetectedGap::new(
        "GAP-003".to_string(),
        GapCategory::Performance,
        GapSeverity::High,
        "Gap 3".to_string(),
      ),
    ];

    let analysis = GapAnalysis::new(gaps, vec![]);

    let critical = analysis.gaps_by_severity(GapSeverity::Critical);
    assert_eq!(critical.len(), 1);

    let high = analysis.gaps_by_severity(GapSeverity::High);
    assert_eq!(high.len(), 2);
  }

  #[test]
  fn test_gap_analysis_critical_gaps() {
    let gaps = vec![
      DetectedGap::new(
        "GAP-001".to_string(),
        GapCategory::Security,
        GapSeverity::Critical,
        "Critical gap".to_string(),
      ),
      DetectedGap::new(
        "GAP-002".to_string(),
        GapCategory::Performance,
        GapSeverity::Medium,
        "Medium gap".to_string(),
      ),
    ];

    let analysis = GapAnalysis::new(gaps, vec![]);

    assert!(analysis.has_critical_gaps());
    assert_eq!(analysis.critical_gaps().len(), 1);
  }

  #[test]
  fn test_gap_analysis_prioritized_gaps() {
    let gaps = vec![
      DetectedGap::new(
        "GAP-001".to_string(),
        GapCategory::Security,
        GapSeverity::Low,
        "Low gap".to_string(),
      ),
      DetectedGap::new(
        "GAP-002".to_string(),
        GapCategory::Performance,
        GapSeverity::Critical,
        "Critical gap".to_string(),
      ),
      DetectedGap::new(
        "GAP-003".to_string(),
        GapCategory::Functional,
        GapSeverity::Medium,
        "Medium gap".to_string(),
      ),
    ];

    let analysis = GapAnalysis::new(gaps, vec![]);

    let prioritized = analysis.prioritized_gaps();

    // Should be sorted by severity (highest first)
    assert_eq!(prioritized[0].severity, GapSeverity::Critical);
    assert_eq!(prioritized[1].severity, GapSeverity::Medium);
    assert_eq!(prioritized[2].severity, GapSeverity::Low);
  }

  #[test]
  fn test_gap_analysis_get_coverage() {
    let category_coverage = vec![(GapCategory::Functional, 80), (GapCategory::Security, 40)];

    let analysis = GapAnalysis::new(vec![], category_coverage);

    assert_eq!(analysis.get_coverage(GapCategory::Functional), Some(80));
    assert_eq!(analysis.get_coverage(GapCategory::Security), Some(40));
    assert_eq!(analysis.get_coverage(GapCategory::Performance), None);
  }

  #[test]
  fn test_gap_analysis_covered_categories() {
    let category_coverage = vec![
      (GapCategory::Functional, 85),
      (GapCategory::Security, 40),
      (GapCategory::Performance, 90),
    ];

    let analysis = GapAnalysis::new(vec![], category_coverage);

    assert!(analysis
      .covered_categories
      .contains(&GapCategory::Functional));
    assert!(analysis
      .covered_categories
      .contains(&GapCategory::Performance));
    assert!(!analysis.covered_categories.contains(&GapCategory::Security));
  }

  #[test]
  fn test_calculate_overall_coverage() {
    let coverage = vec![(GapCategory::Functional, 100), (GapCategory::Security, 50)];

    let overall = calculate_overall_coverage(&coverage);

    assert_eq!(overall, 75);
  }

  #[test]
  fn test_calculate_overall_coverage_empty() {
    let overall = calculate_overall_coverage(&[]);
    assert_eq!(overall, 0);
  }

  #[test]
  fn test_generate_requirements_template() {
    let gaps = vec![DetectedGap::new(
      "GAP-001".to_string(),
      GapCategory::Security,
      GapSeverity::Critical,
      "Missing security requirements".to_string(),
    )];

    let analysis = GapAnalysis::new(gaps, vec![]);

    let template = generate_requirements_template(&analysis);

    assert!(template.contains("Requirements Template"));
    assert!(template.contains("Security"));
  }

  #[test]
  fn test_analyze_category_no_coverage() {
    let requirements = &["No relevant indicators here."];
    let mut gap_id = 0;

    let (coverage, gaps) = analyze_category(GapCategory::Security, requirements, &mut gap_id);

    assert!(coverage < 50);
    assert!(!gaps.is_empty());
  }

  #[test]
  fn test_analyze_category_full_coverage() {
    let requirements = &[
      "The system shall authenticate users.",
      "Data must be encrypted.",
      "Access control is required.",
    ];
    let mut gap_id = 0;

    let (coverage, _gaps) = analyze_category(GapCategory::Security, requirements, &mut gap_id);

    // Should have higher coverage with security indicators
    assert!(coverage > 0);
  }

  #[test]
  fn test_all_categories_analyzed() {
    let analysis = detect_gaps(&["Test requirement."]);

    // All 10 categories should have coverage scores
    assert_eq!(analysis.category_coverage.len(), 10);
  }

  #[test]
  fn test_coverage_counts_unique_requirements_not_indicators() {
    // Bug: If one requirement contains all 5 security indicators,
    // current code returns 100% even though only 1/3 requirements are covered
    let requirements = &[
      // This requirement has ALL 5 security indicators
      "The system must provide auth and encrypt data with secure access control and permission management.",
      // These two have no security indicators
      "Users can create reports.",
      "Reports can be exported.",
    ];

    let coverage = check_category_coverage(requirements, GapCategory::Security);

    // With 3 requirements and only 1 having security indicators, correct coverage should be ~33%
    // But buggy code returns 100% (5/5 indicators found)
    assert!(
      coverage <= 50,
      "Coverage should reflect unique requirements (33%), not indicator count (100%). Got {}%",
      coverage
    );
  }
}
