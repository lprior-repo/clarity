#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// `AIHints` - hints to guide AI code generation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AIHints {
  /// Implementation hints
  #[serde(default)]
  pub implementation: ImplementationHints,
  /// Entity hints for data modeling (keyed by entity name)
  #[serde(default)]
  pub entities: HashMap<String, EntityHint>,
  /// Security considerations
  #[serde(default)]
  pub security: SecurityHints,
  /// Common pitfalls to avoid
  #[serde(default)]
  pub pitfalls: Vec<String>,
}

/// `ImplementationHints` - hints for implementation approach
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ImplementationHints {
  /// Suggested technology stack
  #[serde(default)]
  pub suggested_stack: Vec<String>,
}

/// `EntityHint` - hint for data entity modeling
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EntityHint {
  /// Suggested fields with their type hints (as JSON values)
  #[serde(default)]
  pub fields: HashMap<String, Value>,
}

/// `SecurityHints` - security-related considerations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SecurityHints {
  /// Password hashing algorithm
  #[serde(default)]
  pub password_hashing: String,
  /// JWT algorithm to use
  #[serde(default)]
  pub jwt_algorithm: String,
  /// JWT expiration time
  #[serde(default)]
  pub jwt_expiry: String,
  /// Rate limiting configuration
  #[serde(default)]
  pub rate_limiting: String,
}

#[cfg(test)]
mod tests {
  use super::AIHints;

  #[test]
  fn test_ai_hints_default() {
    let hints = AIHints::default();
    assert!(hints.entities.is_empty());
    assert!(hints.pitfalls.is_empty());
  }
}
