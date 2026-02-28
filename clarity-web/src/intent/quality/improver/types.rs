use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ImproverError {
  #[error("quality report is empty")]
  EmptyReport,
  #[error("invalid priority value: {0}")]
  InvalidPriority(u8),
  #[error("category not found: {0}")]
  CategoryNotFound(String),
  #[error("invalid field reference: {0}")]
  InvalidFieldReference(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImprovementSuggestion {
  pub category: String,
  pub description: String,
  pub priority: u8,
  pub affected_field: String,
  pub suggested_action: String,
}

impl ImprovementSuggestion {
  /// Creates a new improvement suggestion with validated priority.
  ///
  /// # Errors
  /// Returns `ImproverError::InvalidPriority` when `priority` is outside `1..=10`.
  pub fn new(
    category: impl Into<String>,
    description: impl Into<String>,
    priority: u8,
    affected_field: impl Into<String>,
    suggested_action: impl Into<String>,
  ) -> Result<Self, ImproverError> {
    if (1..=10).contains(&priority) {
      Ok(Self {
        category: category.into(),
        description: description.into(),
        priority,
        affected_field: affected_field.into(),
        suggested_action: suggested_action.into(),
      })
    } else {
      Err(ImproverError::InvalidPriority(priority))
    }
  }

  #[must_use]
  pub const fn is_high_priority(&self) -> bool {
    self.priority >= 8
  }

  #[must_use]
  pub fn is_medium_priority(&self) -> bool {
    (4..=7).contains(&self.priority)
  }

  #[must_use]
  pub const fn is_low_priority(&self) -> bool {
    self.priority <= 3
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueCategory {
  MissingTests,
  VagueRules,
  MissingExamples,
  MissingErrors,
  MissingAuth,
  MissingEdgeCases,
  LowCompleteness,
  LowClarity,
  LowSecurity,
  LowTestability,
  LowConsistency,
}

impl IssueCategory {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityIssueReport {
  pub category: IssueCategory,
  pub severity: u8,
  pub field: String,
  pub description: String,
  pub context: Option<String>,
}

impl QualityIssueReport {
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

  #[must_use]
  pub fn with_context(mut self, context: impl Into<String>) -> Self {
    self.context = Some(context.into());
    self
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityReport {
  pub overall_score: u8,
  pub issues: Vec<QualityIssueReport>,
  pub behavior_count: usize,
  pub feature_count: usize,
  pub unverified_behaviors: Vec<String>,
  pub behaviors_without_examples: Vec<String>,
  pub vague_rules: Vec<String>,
  pub missing_error_tests: Vec<String>,
  pub missing_auth_tests: Vec<String>,
  pub missing_edge_cases: Vec<String>,
}

impl QualityReport {
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

  #[must_use]
  pub fn with_scores(overall_score: u8, behavior_count: usize, feature_count: usize) -> Self {
    Self {
      overall_score,
      behavior_count,
      feature_count,
      ..Self::new()
    }
  }

  pub fn add_issue(&mut self, issue: QualityIssueReport) {
    self.issues.push(issue);
  }

  #[must_use]
  pub fn issues_by_category(&self, category: IssueCategory) -> Vec<&QualityIssueReport> {
    self
      .issues
      .iter()
      .filter(|issue| issue.category == category)
      .collect()
  }

  #[must_use]
  pub fn has_critical_issues(&self) -> bool {
    self.issues.iter().any(|issue| issue.severity >= 8)
  }

  #[must_use]
  pub fn count_by_category(&self, category: IssueCategory) -> usize {
    self
      .issues
      .iter()
      .filter(|issue| issue.category == category)
      .count()
  }
}

impl Default for QualityReport {
  fn default() -> Self {
    Self::new()
  }
}
