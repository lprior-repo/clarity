#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
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

  /// Behavior name does not match required snake_case pattern
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
}

#[cfg(test)]
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
