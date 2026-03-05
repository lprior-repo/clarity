#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
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
  pub const fn label(&self) -> &'static str {
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
  pub const fn score(&self) -> u8 {
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
      .map(|s| s.to_string())
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    "Coverage: {}% | Gaps: {} critical, {} high, {} medium priority",
    overall_coverage, critical, high, medium
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
  let mut gaps = Vec::new();
  let mut category_coverage = Vec::new();
  let mut gap_id = 0;

  // Analyze each category
  for category in GapCategory::all() {
    let (coverage, category_gaps) = analyze_category(category, requirements, &mut gap_id);
    category_coverage.push((category, coverage));
    gaps.extend(category_gaps);
  }

  GapAnalysis::new(gaps, category_coverage)
}

/// Analyze a single category for gaps
fn analyze_category(
  category: GapCategory,
  requirements: &[&str],
  gap_id: &mut usize,
) -> (u8, Vec<DetectedGap>) {
  let indicators = category.coverage_indicators();

  // Count how many indicators are found
  let mut found_indicators = 0;

  for req in requirements {
    let lower = req.to_lowercase();
    for indicator in indicators {
      if lower.contains(indicator) {
        found_indicators += 1;
      }
    }
  }

  // Calculate coverage score
  let coverage = if indicators.is_empty() {
    100
  } else {
    let ratio = found_indicators as f32 / indicators.len() as f32;
    (ratio * 100.0).min(100.0) as u8
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
      format!("GAP-{:03}", gap_id),
      category,
      severity,
      format!("Missing {} requirements", category.label()),
    )
    .with_evidence(format!(
      "Found {} of {} expected indicators",
      found_indicators,
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

  for gap in analysis.prioritized_gaps() {
    template.push_str(&format!(
      "## {} ({:?})\n\n",
      gap.category.label(),
      gap.severity
    ));
    template.push_str(&format!("{}\n\n", gap.description));

    for suggestion in &gap.suggestions {
      template.push_str(&format!("- [ ] {}\n", suggestion));
    }
    template.push('\n');
  }

  template
}

#[cfg(test)]
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
    assert!(analysis.overall_coverage > 30);
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
}
