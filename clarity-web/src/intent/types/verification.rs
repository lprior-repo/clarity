#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Verification - how to verify a behavior works correctly
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Verification {
  /// Verification type (e.g., "`unit_test`", "`integration_test`", "manual")
  #[serde(default)]
  pub verification_type: String,
  /// Description of how to verify
  #[serde(default)]
  pub description: String,
  /// Example test case or verification steps
  #[serde(default)]
  pub example: String,
}

impl Verification {
  /// Create a new verification
  #[must_use]
  pub const fn new(verification_type: String, description: String) -> Self {
    Self {
      verification_type,
      description,
      example: String::new(),
    }
  }

  /// Builder method to set example
  #[must_use]
  pub fn with_example(self, example: String) -> Self {
    Self { example, ..self }
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
  use super::Verification;

  #[test]
  fn test_verification_builder() {
    let verification = Verification::new("unit_test".to_string(), "Test login".to_string())
      .with_example("assert!(login(user, pass))".to_string());

    assert_eq!(verification.verification_type, "unit_test");
    assert_eq!(verification.description, "Test login");
    assert_eq!(verification.example, "assert!(login(user, pass))");
  }
}
