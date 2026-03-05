#![allow(clippy::expect_used)]

use crate::intent::quality::improver::{
  suggest_examples_improvements, suggest_improvements, suggest_missing_tests,
  suggest_vague_rules_improvements, ImprovementSuggestion, IssueCategory, QualityIssueReport,
  QualityReport,
};

#[test]
fn improvement_suggestion_priority_helpers() {
  let high = ImprovementSuggestion::new("testing", "a", 9, "f", "x").expect("setup");
  let medium = ImprovementSuggestion::new("testing", "a", 5, "f", "x").expect("setup");
  let low = ImprovementSuggestion::new("testing", "a", 2, "f", "x").expect("setup");
  assert!(high.is_high_priority());
  assert!(medium.is_medium_priority());
  assert!(low.is_low_priority());
}

#[test]
fn missing_test_suggestions_are_created() {
  let mut report = QualityReport::new();
  report.missing_auth_tests = vec!["admin".to_string()];
  report.missing_error_tests = vec!["create_user".to_string()];

  let suggestions = suggest_missing_tests(&report);
  assert!(suggestions.iter().any(|item| item.priority == 10));
  assert!(suggestions
    .iter()
    .any(|item| item.description.contains("error handling")));
}

#[test]
fn vague_rule_suggestions_are_created() {
  let mut report = QualityReport::new();
  report.vague_rules = vec!["system should be fast".to_string()];
  let suggestions = suggest_vague_rules_improvements(&report);
  assert_eq!(suggestions.len(), 1);
  assert!(suggestions[0]
    .description
    .to_lowercase()
    .contains("performance"));
}

#[test]
fn full_suggestion_pipeline_is_sorted() {
  let mut report = QualityReport::with_scores(40, 10, 2);
  report.missing_auth_tests = vec!["auth".to_string()];
  report.behaviors_without_examples = vec!["calc".to_string()];
  report.vague_rules = vec!["rule should be good".to_string()];
  report.add_issue(QualityIssueReport::new(
    IssueCategory::LowSecurity,
    8,
    "auth",
    "missing rate limit",
  ));

  let suggestions = suggest_improvements(&report);
  assert!(!suggestions.is_empty());
  suggestions.windows(2).for_each(|pair| {
    assert!(pair[0].priority >= pair[1].priority);
  });
}

#[test]
fn examples_suggestions_detect_missing_examples() {
  let mut report = QualityReport::new();
  report.behaviors_without_examples = vec!["calculate_price".to_string()];
  let suggestions = suggest_examples_improvements(&report);
  assert_eq!(suggestions.len(), 1);
  assert!(suggestions[0].description.contains("Add example"));
}
