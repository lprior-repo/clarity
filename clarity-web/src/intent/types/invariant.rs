#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Invariant - a system property that must always hold
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Invariant {
  /// Invariant name/identifier
  pub name: String,
  /// Description of the invariant
  #[serde(default)]
  pub description: String,
  /// Formal or informal specification
  #[serde(default)]
  pub constraint: String,
}

impl Invariant {
  /// Create a new invariant
  #[must_use]
  pub const fn new(name: String, description: String) -> Self {
    Self {
      name,
      description,
      constraint: String::new(),
    }
  }

  /// Builder method to set constraint
  #[must_use]
  pub fn with_constraint(self, constraint: String) -> Self {
    Self { constraint, ..self }
  }
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
  use super::Invariant;

  #[test]
  fn test_invariant_builder() {
    let invariant = Invariant::new(
      "unique_email".to_string(),
      "Emails must be unique".to_string(),
    )
    .with_constraint("email UNIQUE in users".to_string());

    assert_eq!(invariant.name, "unique_email");
    assert_eq!(invariant.description, "Emails must be unique");
    assert_eq!(invariant.constraint, "email UNIQUE in users");
  }
}
