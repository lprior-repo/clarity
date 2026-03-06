#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// `AntiPattern` - a pattern to avoid in implementation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AntiPattern {
  /// Anti-pattern name/identifier
  pub name: String,
  /// Description of the anti-pattern
  #[serde(default)]
  pub description: String,
  /// Why this pattern should be avoided
  #[serde(default)]
  pub why_avoid: String,
  /// Suggested alternative approach
  #[serde(default)]
  pub alternative: String,
}

impl AntiPattern {
  /// Create a new anti-pattern
  #[must_use]
  pub const fn new(name: String, description: String) -> Self {
    Self {
      name,
      description,
      why_avoid: String::new(),
      alternative: String::new(),
    }
  }

  /// Builder method to set why to avoid
  #[must_use]
  pub fn with_why_avoid(self, why: String) -> Self {
    Self {
      why_avoid: why,
      ..self
    }
  }

  /// Builder method to set alternative
  #[must_use]
  pub fn with_alternative(self, alternative: String) -> Self {
    Self {
      alternative,
      ..self
    }
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
  use super::AntiPattern;

  #[test]
  fn test_anti_pattern_builder() {
    let anti = AntiPattern::new("god_object".to_string(), "Avoid god objects".to_string())
      .with_why_avoid("Violates SRP".to_string())
      .with_alternative("Split into focused classes".to_string());

    assert_eq!(anti.name, "god_object");
    assert_eq!(anti.why_avoid, "Violates SRP");
    assert_eq!(anti.alternative, "Split into focused classes");
  }
}
