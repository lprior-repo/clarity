#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

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
  pub const fn required() -> Self {
    Self::Required
  }

  #[must_use]
  pub fn pattern(regex: impl Into<String>) -> Self {
    Self::Pattern {
      pattern: regex.into(),
    }
  }

  #[must_use]
  pub const fn range(min: f64, max: f64) -> Self {
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

/// Rule outcome - explicit state machine replacing bool + Option<String>.
///
/// A rule can either Pass (with optional value) or Fail (with required message).
/// This makes illegal states unrepresentable (e.g., "passed but has error message").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleOutcome {
  /// Rule passed validation
  Pass { value: Option<String> },
  /// Rule failed validation with an error message
  Fail {
    message: String,
    value: Option<String>,
  },
}

impl RuleOutcome {
  /// Create a passing outcome with an optional value.
  #[must_use]
  pub const fn pass(value: Option<String>) -> Self {
    Self::Pass { value }
  }

  /// Create a failing outcome with message and optional value.
  #[must_use]
  pub const fn fail(message: String, value: Option<String>) -> Self {
    Self::Fail { message, value }
  }

  /// Check if the rule passed.
  #[must_use]
  pub const fn is_pass(&self) -> bool {
    matches!(self, Self::Pass { .. })
  }

  /// Check if the rule failed.
  #[must_use]
  pub const fn is_fail(&self) -> bool {
    matches!(self, Self::Fail { .. })
  }

  /// Get the error message if failed.
  #[must_use]
  pub fn message(&self) -> Option<&str> {
    match self {
      Self::Pass { .. } => None,
      Self::Fail { message, .. } => Some(message),
    }
  }

  /// Get the value (available in both states).
  #[must_use]
  pub fn value(&self) -> Option<&str> {
    match self {
      Self::Pass { value } | Self::Fail { value, .. } => value.as_deref(),
    }
  }
}

/// Result of applying a validation rule.
///
/// Uses explicit `RuleOutcome` enum to track pass/fail state with associated data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleResult {
  pub rule_name: String,
  pub outcome: RuleOutcome,
}

impl RuleResult {
  /// Create a passing result.
  #[must_use]
  pub fn passed(rule_name: impl Into<String>, value: Option<String>) -> Self {
    Self {
      rule_name: rule_name.into(),
      outcome: RuleOutcome::pass(value),
    }
  }

  /// Create a failing result.
  #[must_use]
  pub fn failed(
    rule_name: impl Into<String>,
    message: impl Into<String>,
    value: Option<String>,
  ) -> Self {
    Self {
      rule_name: rule_name.into(),
      outcome: RuleOutcome::fail(message.into(), value),
    }
  }

  /// Check if the rule passed.
  #[must_use]
  pub const fn is_pass(&self) -> bool {
    self.outcome.is_pass()
  }

  /// Check if the rule failed.
  #[must_use]
  pub const fn is_fail(&self) -> bool {
    self.outcome.is_fail()
  }

  /// Get the error message if failed.
  #[must_use]
  pub fn message(&self) -> Option<&str> {
    self.outcome.message()
  }

  /// Get the value.
  #[must_use]
  pub fn value(&self) -> Option<&str> {
    self.outcome.value()
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {
  use super::{RuleOutcome, RuleResult};

  // ============================================
  // Exhaustive match tests for RuleOutcome
  // ============================================

  #[test]
  fn rule_outcome_pass_is_pass() {
    let outcome = RuleOutcome::pass(Some("value".to_string()));
    assert!(outcome.is_pass());
    assert!(!outcome.is_fail());
  }

  #[test]
  fn rule_outcome_fail_is_fail() {
    let outcome = RuleOutcome::fail("error message".to_string(), Some("value".to_string()));
    assert!(outcome.is_fail());
    assert!(!outcome.is_pass());
  }

  #[test]
  fn rule_outcome_pass_has_no_message() {
    let outcome = RuleOutcome::pass(None);
    assert!(outcome.message().is_none());
  }

  #[test]
  fn rule_outcome_fail_has_message() {
    let outcome = RuleOutcome::fail("error".to_string(), None);
    assert_eq!(outcome.message(), Some("error"));
  }

  #[test]
  fn rule_outcome_pass_can_have_value() {
    let outcome = RuleOutcome::pass(Some("test".to_string()));
    assert_eq!(outcome.value(), Some("test"));
  }

  #[test]
  fn rule_outcome_pass_can_have_no_value() {
    let outcome = RuleOutcome::pass(None);
    assert!(outcome.value().is_none());
  }

  #[test]
  fn rule_outcome_fail_can_have_value() {
    let outcome = RuleOutcome::fail("err".to_string(), Some("val".to_string()));
    assert_eq!(outcome.value(), Some("val"));
  }

  #[test]
  fn rule_outcome_fail_can_have_no_value() {
    let outcome = RuleOutcome::fail("err".to_string(), None);
    assert!(outcome.value().is_none());
  }

  // ============================================
  // Exhaustive match tests for RuleResult
  // ============================================

  #[test]
  fn rule_result_passed_creates_pass_outcome() {
    let result = RuleResult::passed("required", Some("value".to_string()));
    assert!(result.is_pass());
    assert_eq!(result.rule_name, "required");
    assert!(result.message().is_none());
    assert_eq!(result.value(), Some("value"));
  }

  #[test]
  fn rule_result_failed_creates_fail_outcome() {
    let result = RuleResult::failed("required", "is empty", Some(String::new()));
    assert!(result.is_fail());
    assert_eq!(result.rule_name, "required");
    assert_eq!(result.message(), Some("is empty"));
    assert_eq!(result.value(), Some(""));
  }

  #[test]
  fn rule_result_delegates_is_pass() {
    let pass = RuleResult::passed("test", None);
    let fail = RuleResult::failed("test", "msg", None);
    assert!(pass.is_pass());
    assert!(!fail.is_pass());
  }

  #[test]
  fn rule_result_delegates_is_fail() {
    let pass = RuleResult::passed("test", None);
    let fail = RuleResult::failed("test", "msg", None);
    assert!(!pass.is_fail());
    assert!(fail.is_fail());
  }

  #[test]
  fn rule_result_delegates_message() {
    let pass = RuleResult::passed("test", None);
    let fail = RuleResult::failed("test", "error message", None);
    assert!(pass.message().is_none());
    assert_eq!(fail.message(), Some("error message"));
  }

  #[test]
  fn rule_result_delegates_value() {
    let with_value = RuleResult::passed("test", Some("data".to_string()));
    let without_value = RuleResult::passed("test", None);
    assert_eq!(with_value.value(), Some("data"));
    assert!(without_value.value().is_none());
  }
}
