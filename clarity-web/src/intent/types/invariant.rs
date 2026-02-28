#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
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
  pub criteria: Vec<String>,
}

impl Invariant {
  /// Create a new invariant
  #[must_use]
  pub const fn new(name: String, description: String) -> Self {
    Self {
      name,
      description,
      criteria: Vec::new(),
    }
  }

  /// Builder method to set criteria
  #[must_use]
  pub fn with_criteria(self, criteria: Vec<String>) -> Self {
    Self { criteria, ..self }
  }
}

#[cfg(test)]
mod tests {
  use super::Invariant;

  #[test]
  fn test_invariant_builder() {
    let invariant = Invariant::new(
      "unique_email".to_string(),
      "Emails must be unique".to_string(),
    )
    .with_criteria(vec!["email UNIQUE in users".to_string()]);

    assert_eq!(invariant.name, "unique_email");
    assert_eq!(invariant.description, "Emails must be unique");
    assert_eq!(invariant.criteria, vec!["email UNIQUE in users"]);
  }
}
