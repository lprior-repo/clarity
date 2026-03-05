use itertools::Itertools;

use super::types::{ImprovementSuggestion, IssueCategory, QualityIssueReport, QualityReport};
use super::vague_rules::analyze_vague_rule;

#[must_use]
pub fn suggest_improvements(report: &QualityReport) -> Vec<ImprovementSuggestion> {
  [
    suggest_missing_tests(report),
    suggest_vague_rules_improvements(report),
    suggest_examples_improvements(report),
    suggest_security_improvements(report),
    suggest_completeness_improvements(report),
    suggest_clarity_improvements(report),
    suggest_consistency_improvements(report),
  ]
  .into_iter()
  .flatten()
  .sorted_by(|left, right| {
    right
      .priority
      .cmp(&left.priority)
      .then(left.category.cmp(&right.category))
  })
  .collect()
}

#[must_use]
pub fn suggest_missing_tests(report: &QualityReport) -> Vec<ImprovementSuggestion> {
  let error_test_suggestions = report.missing_error_tests.iter().filter_map(|area| {
        suggestion(
            "testing",
            format!("Add error handling tests for {area}"),
            9,
            area,
            format!(
                "Create test cases that verify error conditions in {area}. Include tests for: invalid inputs, boundary conditions, resource exhaustion, and failure states."
            ),
        )
    });

  let auth_test_suggestions = report.missing_auth_tests.iter().filter_map(|area| {
        suggestion(
            "testing",
            format!("Add authentication/authorization tests for {area}"),
            10,
            area,
            format!(
                "Create test cases that verify authentication and authorization in {area}. Include tests for: unauthenticated access, insufficient permissions, token expiration, and role-based access control."
            ),
        )
    });

  let edge_case_suggestions = report.missing_edge_cases.iter().filter_map(|area| {
        suggestion(
            "testing",
            format!("Add edge case tests for {area}"),
            7,
            area,
            format!(
                "Create test cases for edge cases in {area}. Consider: empty inputs, maximum values, null/nil handling, concurrent access, and timeout scenarios."
            ),
        )
    });

  let unverified_behavior_suggestions = report.unverified_behaviors.iter().filter_map(|behavior| {
        suggestion(
            "testing",
            format!("Add verification for behavior: {behavior}"),
            8,
            behavior,
            format!(
                "Define verification criteria for {behavior}. Specify: test type (unit/integration/manual), expected outcomes, and validation steps."
            ),
        )
    });

  let low_testability = report
    .issues_by_category(IssueCategory::LowTestability)
    .into_iter()
    .filter_map(|issue| {
      suggestion(
        "testing",
        format!("Improve testability: {}", issue.description),
        issue.severity,
        issue.field.clone(),
        format!(
          "Add acceptance criteria and verification steps. {}",
          context_or_default(issue, "")
        ),
      )
    });

  error_test_suggestions
    .chain(auth_test_suggestions)
    .chain(edge_case_suggestions)
    .chain(unverified_behavior_suggestions)
    .chain(low_testability)
    .collect()
}

#[must_use]
pub fn suggest_vague_rules_improvements(report: &QualityReport) -> Vec<ImprovementSuggestion> {
  let by_rule = report.vague_rules.iter().filter_map(|rule| {
    let (description, action) = analyze_vague_rule(rule);
    suggestion("clarity", description, 7, rule, action)
  });

  let by_low_clarity = report
        .issues_by_category(IssueCategory::LowClarity)
        .into_iter()
        .filter_map(|issue| {
            suggestion(
                "clarity",
                format!("Clarify: {}", issue.description),
                issue.severity,
                issue.field.clone(),
                format!(
                    "Rewrite with specific values and examples. Avoid ambiguous terms like 'fast', 'good', or 'appropriate'. {}",
                    context_or_default(issue, "Use measurable criteria.")
                ),
            )
        });

  by_rule.chain(by_low_clarity).collect()
}

#[must_use]
pub fn suggest_examples_improvements(report: &QualityReport) -> Vec<ImprovementSuggestion> {
  let missing_examples = report
        .behaviors_without_examples
        .iter()
        .filter_map(|behavior| {
            suggestion(
                "completeness",
                format!("Add example for behavior: {behavior}"),
                6,
                behavior,
                format!(
                    "Provide a concrete example demonstrating {behavior}. Include: input values, expected output, and any relevant preconditions or context."
                ),
            )
        });

  let low_completeness = report
    .issues_by_category(IssueCategory::LowCompleteness)
    .into_iter()
    .filter_map(|issue| {
      suggestion(
        "completeness",
        format!("Add missing content: {}", issue.description),
        issue.severity,
        issue.field.clone(),
        format!(
          "Fill in the missing information. {}",
          context_or_default(issue, "Provide complete details for this field.")
        ),
      )
    });

  missing_examples.chain(low_completeness).collect()
}

fn suggest_security_improvements(report: &QualityReport) -> Vec<ImprovementSuggestion> {
  let by_issue = report
    .issues_by_category(IssueCategory::LowSecurity)
    .into_iter()
    .filter_map(|issue| {
      suggestion(
        "security",
        format!("Address security concern: {}", issue.description),
        10,
        issue.field.clone(),
        format!(
          "Add security controls. {}",
          context_or_default(
            issue,
            "Consider authentication, authorization, encryption, and input validation."
          )
        ),
      )
    });

  let by_missing_auth = (!report.missing_auth_tests.is_empty()).then(|| {
        suggestion(
            "security",
            "Add comprehensive security test coverage".to_string(),
            10,
            "security".to_string(),
            format!(
                "The following areas need security tests: {}. Include tests for authentication, authorization, input validation, and injection prevention.",
                report.missing_auth_tests.join(", ")
            ),
        )
    });

  by_issue
    .chain(by_missing_auth.into_iter().flatten())
    .collect()
}

fn suggest_completeness_improvements(report: &QualityReport) -> Vec<ImprovementSuggestion> {
  let score_suggestion = if report.overall_score < 50 {
    suggestion(
            "completeness",
            "Overall quality score is critically low",
            10,
            "overall",
            "Focus on filling in missing required fields and adding verification criteria before addressing other issues.",
        )
  } else if report.overall_score < 70 {
    suggestion(
            "completeness",
            "Overall quality score needs improvement",
            8,
            "overall",
            "Address the identified gaps to improve the overall quality score. Prioritize high-severity issues first.",
        )
  } else {
    None
  };

  let verification_gap = (report.behavior_count > 0
        && report.unverified_behaviors.len() > report.behavior_count / 2)
        .then(|| {
            suggestion(
                "completeness",
                "More than half of behaviors lack verification",
                9,
                "verification",
                format!(
                    "{} of {} behaviors need verification criteria. Define how each behavior will be tested and validated.",
                    report.unverified_behaviors.len(),
                    report.behavior_count
                ),
            )
        });

  score_suggestion
    .into_iter()
    .chain(verification_gap.into_iter().flatten())
    .collect()
}

fn suggest_clarity_improvements(report: &QualityReport) -> Vec<ImprovementSuggestion> {
  (report.vague_rules.len() > 3)
        .then(|| {
            suggestion(
                "clarity",
                "Multiple vague rules detected - consider glossary",
                6,
                "documentation",
                "Create a glossary defining common terms and acceptable value ranges to ensure consistent interpretation across all rules.",
            )
        })
        .into_iter()
        .flatten()
        .collect()
}

fn suggest_consistency_improvements(report: &QualityReport) -> Vec<ImprovementSuggestion> {
  report
    .issues_by_category(IssueCategory::LowConsistency)
    .into_iter()
    .filter_map(|issue| {
      suggestion(
        "consistency",
        format!("Resolve inconsistency: {}", issue.description),
        8,
        issue.field.clone(),
        format!(
          "Review and resolve the contradiction. {}",
          context_or_default(issue, "Ensure all requirements align and do not conflict.")
        ),
      )
    })
    .collect()
}

fn suggestion(
  category: impl Into<String>,
  description: impl Into<String>,
  priority: u8,
  affected_field: impl Into<String>,
  suggested_action: impl Into<String>,
) -> Option<ImprovementSuggestion> {
  ImprovementSuggestion::new(
    category,
    description,
    priority,
    affected_field,
    suggested_action,
  )
  .ok()
}

fn context_or_default<'a>(issue: &'a QualityIssueReport, default: &'a str) -> &'a str {
  issue.context.as_deref().map_or(default, |value| value)
}
