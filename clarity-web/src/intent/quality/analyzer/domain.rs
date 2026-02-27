use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityIssue {
  MissingErrorTests,
  MissingAuthenticationTest,
  MissingEdgeCases,
  VagueRules,
  NoExamples,
  MissingExplanations,
  UntestedInvariants,
  MissingAiHints,
  MissingPreconditions,
  MissingPostconditions,
}

impl QualityIssue {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityReport {
  pub coverage_score: u8,
  pub clarity_score: u8,
  pub testability_score: u8,
  pub ai_readiness_score: u8,
  pub overall_score: u8,
  pub issues: Vec<QualityIssue>,
  pub suggestions: Vec<String>,
}

impl QualityReport {
  #[must_use]
  pub fn new(
    coverage_score: u8,
    clarity_score: u8,
    testability_score: u8,
    ai_readiness_score: u8,
  ) -> Self {
    let overall_score = super::scoring::calculate_overall_score_from_values(
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

  #[must_use]
  pub fn has_issues(&self) -> bool {
    !self.issues.is_empty()
  }

  #[must_use]
  pub fn issue_count(&self) -> usize {
    self.issues.len()
  }

  pub fn add_issue(&mut self, issue: QualityIssue) {
    if !self.issues.contains(&issue) {
      self.issues.push(issue);
      self.suggestions.push(issue.suggestion().to_string());
    }
  }

  pub fn merge_issues(&mut self, other: &QualityReport) {
    other.issues.iter().for_each(|issue| self.add_issue(*issue));
  }
}

#[cfg(test)]
mod tests {
  use super::{QualityIssue, QualityReport};

  #[test]
  fn quality_issue_texts_exist() {
    assert!(!QualityIssue::MissingErrorTests.description().is_empty());
    assert!(!QualityIssue::MissingErrorTests.suggestion().is_empty());
  }

  #[test]
  fn report_add_issue_is_deduplicated() {
    let mut report = QualityReport::new(80, 70, 90, 60);
    report.add_issue(QualityIssue::MissingErrorTests);
    report.add_issue(QualityIssue::MissingErrorTests);
    assert_eq!(report.issue_count(), 1);
  }
}
