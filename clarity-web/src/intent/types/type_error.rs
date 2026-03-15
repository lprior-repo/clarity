#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Type errors for spec validation
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum TypeError {
  /// Name field cannot be empty
  #[error("name cannot be empty")]
  EmptyName,

  /// Behavior name does not match required `snake_case` pattern
  #[error("behavior name '{0}' must be snake_case (lowercase letters, numbers, underscores, starting with letter)")]
  InvalidBehaviorName(String),

  /// Duplicate feature name detected
  #[error("duplicate feature name: '{0}'")]
  DuplicateFeature(String),

  /// Duplicate behavior name within a feature
  #[error("duplicate behavior name '{0}' in feature '{1}'")]
  DuplicateBehavior(String, String),

  /// Circular dependency detected in feature graph
  #[error("circular dependency detected: {0} -> {1}")]
  CircularDependency(String, String),

  /// Feature referenced but not found
  #[error("unknown feature dependency: '{0}'")]
  UnknownFeatureDependency(String),

  /// Too many features in specification
  #[error("too many features: {0} (maximum {1})")]
  TooManyFeatures(usize, usize),

  /// Too many invariants in specification
  #[error("too many invariants: {0} (maximum {1})")]
  TooManyInvariants(usize, usize),

  /// Too many anti-patterns in specification
  #[error("too many anti-patterns: {0} (maximum {1})")]
  TooManyAntiPatterns(usize, usize),

  /// Too many behaviors in feature
  #[error("too many behaviors in feature '{0}': {1} (maximum {2})")]
  TooManyBehaviors(String, usize, usize),

  /// Too many dependencies in feature
  #[error("too many dependencies in feature '{0}': {1} (maximum {2})")]
  TooManyDependencies(String, usize, usize),

  /// Too many preconditions in behavior
  #[error("too many preconditions in behavior '{0}': {1} (maximum {2})")]
  TooManyPreconditions(String, usize, usize),

  /// Too many postconditions in behavior
  #[error("too many postconditions in behavior '{0}': {1} (maximum {2})")]
  TooManyPostconditions(String, usize, usize),
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
  use super::TypeError;

  #[test]
  fn test_type_error_display() {
    let empty_name = TypeError::EmptyName;
    assert_eq!(format!("{empty_name}"), "name cannot be empty");

    let invalid_behavior = TypeError::InvalidBehaviorName("BadName".to_string());
    assert!(format!("{invalid_behavior}").contains("BadName"));

    let duplicate_feature = TypeError::DuplicateFeature("auth".to_string());
    assert!(format!("{duplicate_feature}").contains("auth"));
  }
}
