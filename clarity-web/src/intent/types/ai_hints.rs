#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// `AIHints` - hints to guide AI code generation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub struct AIHints {
  /// Implementation hints
  #[serde(default)]
  pub implementation: ImplementationHints,
  /// Entity hints for data modeling
  #[serde(default)]
  pub entities: Vec<EntityHint>,
  /// Security considerations
  #[serde(default)]
  pub security: SecurityHints,
  /// Preferred libraries or frameworks
  #[serde(default)]
  pub preferred_libraries: Vec<String>,
  /// Code style preferences
  #[serde(default)]
  pub style_hints: Vec<String>,
}


/// `ImplementationHints` - hints for implementation approach
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub struct ImplementationHints {
  /// Suggested architecture pattern
  #[serde(default)]
  pub architecture: String,
  /// Performance considerations
  #[serde(default)]
  pub performance_notes: String,
  /// Error handling approach
  #[serde(default)]
  pub error_handling: String,
}


/// `EntityHint` - hint for data entity modeling
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub struct EntityHint {
  /// Entity name
  pub name: String,
  /// Entity description
  #[serde(default)]
  pub description: String,
  /// Suggested fields
  #[serde(default)]
  pub fields: Vec<String>,
  /// Relationships to other entities
  #[serde(default)]
  pub relationships: Vec<String>,
}


/// `SecurityHints` - security-related considerations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub struct SecurityHints {
  /// Authentication requirements
  #[serde(default)]
  pub authentication: String,
  /// Authorization model
  #[serde(default)]
  pub authorization: String,
  /// Data sensitivity classification
  #[serde(default)]
  pub data_sensitivity: String,
  /// Security concerns to address
  #[serde(default)]
  pub concerns: Vec<String>,
}


#[cfg(test)]
mod tests {
  use super::AIHints;

  #[test]
  fn test_ai_hints_default() {
    let hints = AIHints::default();
    assert!(hints.entities.is_empty());
    assert!(hints.preferred_libraries.is_empty());
    assert!(hints.style_hints.is_empty());
  }
}
