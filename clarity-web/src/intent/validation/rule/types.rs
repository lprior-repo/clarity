use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Rule {
  Required,
  Pattern { pattern: String },
  Range { min: f64, max: f64 },
  Custom { name: String, check: String },
}

impl Rule {
  #[must_use]
  pub fn name(&self) -> &str {
    match self {
      Self::Required => "required",
      Self::Pattern { .. } => "pattern",
      Self::Range { .. } => "range",
      Self::Custom { name, .. } => name,
    }
  }

  #[must_use]
  pub fn required() -> Self {
    Self::Required
  }

  #[must_use]
  pub fn pattern(regex: impl Into<String>) -> Self {
    Self::Pattern {
      pattern: regex.into(),
    }
  }

  #[must_use]
  pub fn range(min: f64, max: f64) -> Self {
    Self::Range { min, max }
  }

  #[must_use]
  pub fn custom(name: impl Into<String>, check: impl Into<String>) -> Self {
    Self::Custom {
      name: name.into(),
      check: check.into(),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleResult {
  pub rule_name: String,
  pub passed: bool,
  pub message: Option<String>,
  pub value: Option<String>,
}

impl RuleResult {
  #[must_use]
  pub fn passed(rule_name: impl Into<String>, value: Option<String>) -> Self {
    Self {
      rule_name: rule_name.into(),
      passed: true,
      message: None,
      value,
    }
  }

  #[must_use]
  pub fn failed(
    rule_name: impl Into<String>,
    message: impl Into<String>,
    value: Option<String>,
  ) -> Self {
    Self {
      rule_name: rule_name.into(),
      passed: false,
      message: Some(message.into()),
      value,
    }
  }

  #[must_use]
  pub const fn is_pass(&self) -> bool {
    self.passed
  }

  #[must_use]
  pub const fn is_fail(&self) -> bool {
    !self.passed
  }
}

#[derive(Debug, Clone, Copy)]
pub enum Comparison {
  Gt,
  Lt,
  Gte,
  Lte,
  Eq,
  Ne,
}
