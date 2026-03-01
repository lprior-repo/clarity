#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Verification - how to verify a behavior works correctly
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Verification {
  /// Verification type (e.g., "`unit_test`", "`integration_test`", "manual")
  /// Deprecated: Use criteria and example fields instead
  #[deprecated(since = "0.1.0", note = "Use criteria and example fields instead")]
  #[serde(default)]
  pub verification_type: String,
  /// Description of how to verify
  /// Deprecated: Use criteria and example fields instead
  #[deprecated(since = "0.1.0", note = "Use criteria and example fields instead")]
  #[serde(default)]
  pub description: String,
  /// Example test case or verification steps
  #[serde(default)]
  pub example: String,
  /// Criteria list for verification
  #[serde(default)]
  pub criteria: Vec<String>,
}

impl Verification {
  /// Create a new verification (legacy constructor with deprecated fields)
  #[deprecated(
    since = "0.1.0",
    note = "Use Verification::with_criteria_and_example instead"
  )]
  #[allow(deprecated)]
  #[must_use]
  pub const fn new(verification_type: String, description: String) -> Self {
    Self {
      verification_type,
      description,
      example: String::new(),
      criteria: Vec::new(),
    }
  }

  /// Create a new verification with criteria and example (recommended constructor)
  #[allow(deprecated)]
  #[must_use]
  pub const fn with_criteria_and_example(example: String, criteria: Vec<String>) -> Self {
    Self {
      verification_type: String::new(),
      description: String::new(),
      example,
      criteria,
    }
  }

  /// Builder method to set example
  #[must_use]
  pub fn with_example(self, example: String) -> Self {
    Self { example, ..self }
  }

  /// Builder method to set criteria
  #[must_use]
  pub fn with_criteria(self, criteria: Vec<String>) -> Self {
    Self { criteria, ..self }
  }
}

#[cfg(test)]
mod tests {
  #![allow(deprecated)]

  use super::Verification;

  #[test]
  fn test_verification_builder() {
    let verification = Verification::new("unit_test".to_string(), "Test login".to_string())
      .with_example("assert!(login(user, pass))".to_string());

    assert_eq!(verification.verification_type, "unit_test");
    assert_eq!(verification.description, "Test login");
    assert_eq!(verification.example, "assert!(login(user, pass))");
  }

  /// Test backward compatibility: old JSON with verification_type and description
  #[test]
  fn test_verification_backward_compatibility_old_format() {
    let old_json = r#"{
      "verification_type": "integration_test",
      "description": "Verify user login flow"
    }"#;

    let result = serde_json::from_str::<Verification>(old_json);
    assert!(result.is_ok());
    let verification = result.unwrap();
    assert_eq!(verification.verification_type, "integration_test");
    assert_eq!(verification.description, "Verify user login flow");
    assert!(verification.example.is_empty());
    assert!(verification.criteria.is_empty());
  }

  /// Test new format with criteria and example (replacing deprecated fields)
  #[test]
  fn test_verification_new_format_with_criteria_and_example() {
    let verification = Verification::default()
      .with_example("assert!(authenticate(creds) == Ok(Session))".to_string())
      .with_criteria(vec![
        "Returns success for valid credentials".to_string(),
        "Returns error for invalid credentials".to_string(),
      ]);

    assert!(verification.verification_type.is_empty());
    assert!(verification.description.is_empty());
    assert_eq!(
      verification.example,
      "assert!(authenticate(creds) == Ok(Session))"
    );
    assert_eq!(verification.criteria.len(), 2);
  }

  /// Test serialization/deserialization roundtrip with new format
  #[test]
  fn test_verification_roundtrip_new_format() {
    let original = Verification::default()
      .with_example("assert!(login(user))".to_string())
      .with_criteria(vec!["success case".to_string()]);

    let json = serde_json::to_string(&original);
    assert!(json.is_ok());
    let json_str = json.unwrap();

    let parsed = serde_json::from_str::<Verification>(&json_str);
    assert!(parsed.is_ok());
    let deserialized = parsed.unwrap();

    assert_eq!(original, deserialized);
  }

  /// Test backward compatibility roundtrip (old format still works)
  #[test]
  fn test_verification_roundtrip_old_format() {
    let original = Verification::new("manual".to_string(), "Check UI manually".to_string());

    let json = serde_json::to_string(&original);
    assert!(json.is_ok());
    let json_str = json.unwrap();

    let parsed = serde_json::from_str::<Verification>(&json_str);
    assert!(parsed.is_ok());
    let deserialized = parsed.unwrap();

    assert_eq!(original.verification_type, deserialized.verification_type);
    assert_eq!(original.description, deserialized.description);
  }

  /// Test that default verification works
  #[test]
  fn test_verification_default() {
    let verification = Verification::default();
    assert!(verification.verification_type.is_empty());
    assert!(verification.description.is_empty());
    assert!(verification.example.is_empty());
    assert!(verification.criteria.is_empty());
  }

  /// Test the new constructor with criteria and example
  #[test]
  fn test_verification_with_criteria_and_example() {
    #[allow(deprecated)]
    let verification = Verification::with_criteria_and_example(
      "assert!(login(user))".to_string(),
      vec!["returns token".to_string(), "creates session".to_string()],
    );

    assert!(verification.verification_type.is_empty());
    assert!(verification.description.is_empty());
    assert_eq!(verification.example, "assert!(login(user))");
    assert_eq!(verification.criteria.len(), 2);
  }
}
