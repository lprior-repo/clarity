use crate::intent::types::Spec;

use super::domain::{QualityIssue, QualityReport};
use super::scoring::{
  calculate_description_ratio, calculate_example_ratio, calculate_postcondition_ratio,
  calculate_precondition_ratio, check_has_auth_tests, check_has_edge_cases, check_has_error_tests,
  check_invariants_tested, count_vague_language,
};

pub(super) fn collect_coverage_issues(spec: &Spec, report: &mut QualityReport) {
  [
    (
      !check_has_error_tests(spec),
      QualityIssue::MissingErrorTests,
    ),
    (
      !check_has_auth_tests(spec),
      QualityIssue::MissingAuthenticationTest,
    ),
    (!check_has_edge_cases(spec), QualityIssue::MissingEdgeCases),
    (
      !spec.invariants.is_empty() && !check_invariants_tested(spec),
      QualityIssue::UntestedInvariants,
    ),
  ]
  .into_iter()
  .filter_map(|(cond, issue)| cond.then_some(issue))
  .for_each(|issue| report.add_issue(issue));
}

pub(super) fn collect_clarity_issues(spec: &Spec, report: &mut QualityReport) {
  [
    (
      calculate_description_ratio(spec) < 0.5,
      QualityIssue::MissingExplanations,
    ),
    (count_vague_language(spec) > 3, QualityIssue::VagueRules),
  ]
  .into_iter()
  .filter_map(|(cond, issue)| cond.then_some(issue))
  .for_each(|issue| report.add_issue(issue));
}

pub(super) fn collect_testability_issues(spec: &Spec, report: &mut QualityReport) {
  [
    (
      calculate_precondition_ratio(spec) < 0.5,
      QualityIssue::MissingPreconditions,
    ),
    (
      calculate_postcondition_ratio(spec) < 0.5,
      QualityIssue::MissingPostconditions,
    ),
    (
      calculate_example_ratio(spec) < 0.5,
      QualityIssue::NoExamples,
    ),
  ]
  .into_iter()
  .filter_map(|(cond, issue)| cond.then_some(issue))
  .for_each(|issue| report.add_issue(issue));
}

pub(super) fn collect_ai_readiness_issues(spec: &Spec, report: &mut QualityReport) {
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
