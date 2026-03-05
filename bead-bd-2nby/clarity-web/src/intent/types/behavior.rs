#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::{BehaviorName, BehaviorReference, TypeError, Verification};

/// Behavior - a single behavior with verification criteria
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Behavior {
  /// Behavior name in `snake_case`
  pub name: String,
  /// Plain English description of what this behavior demonstrates
  #[serde(default)]
  pub intent: String,
  /// Human-readable description of the behavior (deprecated, use intent)
  #[serde(default)]
  pub description: String,
<<<<<<< conflict 1 of 4
+++++++ yynnkqvl f8ad025d "feat(types): add missing Behavior fields to match Gleam type" (rebase destination)
  /// Additional context and notes
  #[serde(default)]
  pub notes: String,
  /// Behavior dependencies (references to other behaviors)
  #[serde(default)]
  pub requires: Vec<String>,
  /// Classification tags
  #[serde(default)]
  pub tags: Vec<String>,
  /// How to verify this behavior
%%%%%%% diff from: turvxutt 0b410a2a "feat(intent-cli): add Rust CLI binary with clap command structure" (parents of rebased revision)
\\\\\\\        to: rwnnoyur 831ba1cb "types: Change Behavior verification to plural" (rebased revision)
-  /// How to verify this behavior
+  /// How to verify this behavior (multiple verifications allowed)
>>>>>>> conflict 1 of 4 ends
  #[serde(default)]
  pub verifications: Vec<Verification>,
  /// Pre-conditions for this behavior (behavior references)
  #[serde(default)]
  pub preconditions: Vec<String>,
  /// Post-conditions after this behavior (behavior references)
  #[serde(default)]
  pub postconditions: Vec<String>,
}

impl Behavior {
  /// Create a new behavior with the given name
  ///
  /// The name must be in `snake_case` format: lowercase letters, numbers,
  /// and underscores, starting with a letter.
  ///
  /// # Errors
  /// Returns `TypeError::InvalidBehaviorName` if name doesn't match pattern
  pub fn new(name: String) -> Result<Self, TypeError> {
    let validated_name = BehaviorName::parse(name.clone())
      .map_err(|_| TypeError::InvalidBehaviorName(name))?;
    Ok(Self {
      name: validated_name.into(),
      intent: String::new(),
      description: String::new(),
<<<<<<< conflict 2 of 4
+++++++ yynnkqvl f8ad025d "feat(types): add missing Behavior fields to match Gleam type" (rebase destination)
      notes: String::new(),
      requires: Vec::new(),
      tags: Vec::new(),
      verification: None,
%%%%%%% diff from: turvxutt 0b410a2a "feat(intent-cli): add Rust CLI binary with clap command structure" (parents of rebased revision)
\\\\\\\        to: rwnnoyur 831ba1cb "types: Change Behavior verification to plural" (rebased revision)
-      verification: None,
+      verifications: Vec::new(),
>>>>>>> conflict 2 of 4 ends
      preconditions: Vec::new(),
      postconditions: Vec::new(),
    })
  }

  /// Create a behavior from a validated `BehaviorName`.
  ///
  /// This constructor accepts a pre-validated name, avoiding redundant validation.
  #[must_use]
  pub fn from_validated_name(name: BehaviorName) -> Self {
    Self {
      name: name.into(),
      intent: String::new(),
      description: String::new(),
<<<<<<< conflict 3 of 4
+++++++ yynnkqvl f8ad025d "feat(types): add missing Behavior fields to match Gleam type" (rebase destination)
      notes: String::new(),
      requires: Vec::new(),
      tags: Vec::new(),
      verification: None,
%%%%%%% diff from: turvxutt 0b410a2a "feat(intent-cli): add Rust CLI binary with clap command structure" (parents of rebased revision)
\\\\\\\        to: rwnnoyur 831ba1cb "types: Change Behavior verification to plural" (rebased revision)
-      verification: None,
+      verifications: Vec::new(),
>>>>>>> conflict 3 of 4 ends
      preconditions: Vec::new(),
      postconditions: Vec::new(),
    }
  }

  /// Get the behavior name as a validated `BehaviorName`.
  ///
  /// Returns `None` if the name is invalid (should not happen for well-constructed behaviors).
  #[must_use]
  pub fn validated_name(&self) -> Option<BehaviorName> {
    BehaviorName::parse(self.name.clone()).ok()
  }

  /// Builder method to set description
  #[must_use]
  pub fn with_description(self, desc: String) -> Self {
    Self {
      description: desc,
      ..self
    }
  }

<<<<<<< conflict 4 of 4
+++++++ yynnkqvl f8ad025d "feat(types): add missing Behavior fields to match Gleam type" (rebase destination)
  /// Builder method to set intent
  #[must_use]
  pub fn with_intent(self, intent: String) -> Self {
    Self { intent, ..self }
  }

  /// Builder method to set notes
  #[must_use]
  pub fn with_notes(self, notes: String) -> Self {
    Self { notes, ..self }
  }

  /// Add a requirement (behavior dependency)
  pub fn add_require(&mut self, requirement: String) -> &mut Self {
    self.requires.push(requirement);
    self
  }

  /// Add a tag
  pub fn add_tag(&mut self, tag: String) -> &mut Self {
    self.tags.push(tag);
    self
  }

  /// Builder method to set verification
%%%%%%% diff from: turvxutt 0b410a2a "feat(intent-cli): add Rust CLI binary with clap command structure" (parents of rebased revision)
\\\\\\\        to: rwnnoyur 831ba1cb "types: Change Behavior verification to plural" (rebased revision)
-  /// Builder method to set verification
+  /// Builder method to add a verification
>>>>>>> conflict 4 of 4 ends
  #[must_use]
  pub fn with_verification(self, verification: Verification) -> Self {
    let mut verifications = self.verifications.clone();
    verifications.push(verification);
    Self { verifications, ..self }
  }

  /// Builder method to set verifications (replaces all)
  #[must_use]
  pub fn with_verifications(self, verifications: Vec<Verification>) -> Self {
    Self { verifications, ..self }
  }

  /// Add a verification
  pub fn add_verification(&mut self, verification: Verification) -> &mut Self {
    self.verifications.push(verification);
    self
  }

  /// Add a pre-condition
  pub fn add_precondition(&mut self, condition: String) -> &mut Self {
    self.preconditions.push(condition);
    self
  }

  /// Add a validated behavior reference as a pre-condition.
  ///
  /// # Errors
  ///
  /// Returns an error if the reference format is invalid.
  pub fn add_precondition_reference(
    &mut self,
    reference: BehaviorReference,
  ) -> &mut Self {
    self.preconditions.push(reference.into());
    self
  }

  /// Add a post-condition
  pub fn add_postcondition(&mut self, condition: String) -> &mut Self {
    self.postconditions.push(condition);
    self
  }

  /// Add a validated behavior reference as a post-condition.
  ///
  /// # Errors
  ///
  /// Returns an error if the reference format is invalid.
  pub fn add_postcondition_reference(
    &mut self,
    reference: BehaviorReference,
  ) -> &mut Self {
    self.postconditions.push(reference.into());
    self
  }

  /// Get preconditions as validated behavior references.
  ///
  /// Invalid references are filtered out.
  #[must_use]
  pub fn validated_preconditions(&self) -> Vec<BehaviorReference> {
    self
      .preconditions
      .iter()
      .filter_map(|s| BehaviorReference::parse(s.clone()).ok())
      .collect()
  }

  /// Get postconditions as validated behavior references.
  ///
  /// Invalid references are filtered out.
  #[must_use]
  pub fn validated_postconditions(&self) -> Vec<BehaviorReference> {
    self
      .postconditions
      .iter()
      .filter_map(|s| BehaviorReference::parse(s.clone()).ok())
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::Behavior;
  use crate::intent::types::{BehaviorName, BehaviorReference, TypeError};

  #[test]
  fn test_behavior_new_valid() {
    let behavior_result = Behavior::new("create_user".to_string());
    assert!(behavior_result.is_ok());

    let behavior = match behavior_result {
      Ok(value) => value,
      Err(_) => return,
    };

    assert_eq!(behavior.name, "create_user");
  }

  #[test]
  fn test_behavior_new_simple() {
    let behavior_result = Behavior::new("save".to_string());
    assert!(behavior_result.is_ok());
  }

  #[test]
  fn test_behavior_new_with_numbers() {
    let behavior_result = Behavior::new("parse_v2".to_string());
    assert!(behavior_result.is_ok());
  }

  #[test]
  fn test_behavior_new_invalid_uppercase() {
    let result = Behavior::new("CreateUser".to_string());
    assert!(matches!(result, Err(TypeError::InvalidBehaviorName(_))));
  }

  #[test]
  fn test_behavior_new_invalid_starts_with_number() {
    let result = Behavior::new("1_create".to_string());
    assert!(matches!(result, Err(TypeError::InvalidBehaviorName(_))));
  }

  #[test]
  fn test_behavior_new_invalid_hyphen() {
    let result = Behavior::new("create-user".to_string());
    assert!(matches!(result, Err(TypeError::InvalidBehaviorName(_))));
  }

  #[test]
  fn test_behavior_from_validated_name() {
    let name = match BehaviorName::parse("login".to_string()) {
      Ok(n) => n,
      Err(_) => return,
    };
    let behavior = Behavior::from_validated_name(name);
    assert_eq!(behavior.name, "login");
  }

  #[test]
  fn test_behavior_validated_name() {
    let behavior = match Behavior::new("create_user".to_string()) {
      Ok(b) => b,
      Err(_) => return,
    };
    let validated = behavior.validated_name();
    assert!(validated.is_some());
    let validated = validated.unwrap();
    assert_eq!(validated.as_str(), "create_user");
  }

  #[test]
  fn test_behavior_add_precondition_reference() {
    let mut behavior = match Behavior::new("create_user".to_string()) {
      Ok(b) => b,
      Err(_) => return,
    };
    let reference = match BehaviorReference::parse("auth.login".to_string()) {
      Ok(r) => r,
      Err(_) => return,
    };
    behavior.add_precondition_reference(reference);
    assert_eq!(behavior.preconditions.len(), 1);
    assert_eq!(behavior.preconditions[0], "auth.login");
  }

  #[test]
  fn test_behavior_add_postcondition_reference() {
    let mut behavior = match Behavior::new("login".to_string()) {
      Ok(b) => b,
      Err(_) => return,
    };
    let reference = match BehaviorReference::parse("session.create".to_string()) {
      Ok(r) => r,
      Err(_) => return,
    };
    behavior.add_postcondition_reference(reference);
    assert_eq!(behavior.postconditions.len(), 1);
    assert_eq!(behavior.postconditions[0], "session.create");
  }

  #[test]
  fn test_behavior_validated_preconditions() {
    let mut behavior = match Behavior::new("create_user".to_string()) {
      Ok(b) => b,
      Err(_) => return,
    };
    // Add valid reference
    behavior.add_precondition("auth.login".to_string());
    // Add invalid reference (no dot)
    behavior.add_precondition("invalid_ref".to_string());

    let validated = behavior.validated_preconditions();
    assert_eq!(validated.len(), 1);
    assert_eq!(validated[0].as_str(), "auth.login");
  }

  #[test]
  fn test_behavior_validated_postconditions() {
    let mut behavior = match Behavior::new("login".to_string()) {
      Ok(b) => b,
      Err(_) => return,
    };
    // Add valid reference
    behavior.add_postcondition("session.create".to_string());
    // Add invalid reference (no dot)
    behavior.add_postcondition("invalid_ref".to_string());

    let validated = behavior.validated_postconditions();
    assert_eq!(validated.len(), 1);
    assert_eq!(validated[0].as_str(), "session.create");
  }

  #[test]
  fn test_serde_roundtrip_behavior() {
    let behavior = match Behavior::new("login".to_string()) {
      Ok(value) => value.with_description("User login".to_string()),
      Err(_) => return,
    };

    let json_result = serde_json::to_string(&behavior);
    assert!(json_result.is_ok());

    let json = match json_result {
      Ok(value) => value,
      Err(_) => return,
    };

    let parsed_result: Result<Behavior, _> = serde_json::from_str(&json);
    assert!(parsed_result.is_ok());

    let parsed = match parsed_result {
      Ok(value) => value,
      Err(_) => return,
    };

    assert_eq!(behavior, parsed);
  }

  #[test]
  fn test_behavior_with_intent() {
    let behavior = match Behavior::new("create_user".to_string()) {
      Ok(b) => b.with_intent("Create a new user account".to_string()),
      Err(_) => return,
    };
    assert_eq!(behavior.intent, "Create a new user account");
  }

  #[test]
  fn test_behavior_with_notes() {
    let behavior = match Behavior::new("create_user".to_string()) {
      Ok(b) => b.with_notes("Admin only operation".to_string()),
      Err(_) => return,
    };
    assert_eq!(behavior.notes, "Admin only operation");
  }

  #[test]
  fn test_behavior_add_require() {
    let mut behavior = match Behavior::new("create_user".to_string()) {
      Ok(b) => b,
      Err(_) => return,
    };
    behavior.add_require("auth.login".to_string());
    assert_eq!(behavior.requires.len(), 1);
    assert_eq!(behavior.requires[0], "auth.login");
  }

  #[test]
  fn test_behavior_add_tag() {
    let mut behavior = match Behavior::new("create_user".to_string()) {
      Ok(b) => b,
      Err(_) => return,
    };
    behavior.add_tag("auth".to_string());
    behavior.add_tag("admin".to_string());
    assert_eq!(behavior.tags.len(), 2);
    assert_eq!(behavior.tags[0], "auth");
    assert_eq!(behavior.tags[1], "admin");
  }

  #[test]
  fn test_behavior_all_new_fields_serde() {
    let mut behavior = match Behavior::new("login".to_string()) {
      Ok(b) => b,
      Err(_) => return,
    };
    behavior.add_require("user.exists".to_string());
    behavior.add_tag("auth".to_string());

    let behavior = behavior
      .with_intent("User authenticates with credentials".to_string())
      .with_notes("Rate limited to 5 attempts per minute".to_string());

    let json_result = serde_json::to_string(&behavior);
    assert!(json_result.is_ok());

    let json = match json_result {
      Ok(value) => value,
      Err(_) => return,
    };

    let parsed_result: Result<Behavior, _> = serde_json::from_str(&json);
    assert!(parsed_result.is_ok());

    let parsed = match parsed_result {
      Ok(value) => value,
      Err(_) => return,
    };

    assert_eq!(parsed.intent, "User authenticates with credentials");
    assert_eq!(parsed.notes, "Rate limited to 5 attempts per minute");
    assert_eq!(parsed.requires, vec!["user.exists".to_string()]);
    assert_eq!(parsed.tags, vec!["auth".to_string()]);
  }
}
