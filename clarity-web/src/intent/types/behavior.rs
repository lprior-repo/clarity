#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::{TypeError, Verification};

/// Maximum number of preconditions allowed in a behavior
const MAX_PRECONDITIONS: usize = 20;
/// Maximum number of postconditions allowed in a behavior
const MAX_POSTCONDITIONS: usize = 20;

/// Validate behavior names as `snake_case` with leading lowercase letter.
fn is_valid_behavior_name(name: &str) -> bool {
  let mut chars = name.chars();
  match chars.next() {
    Some(first) if first.is_ascii_lowercase() => {
      chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    }
    _ => false,
  }
}

/// Behavior - a single behavior with verification criteria
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Behavior {
  /// Behavior name in `snake_case`
  pub name: String,
  /// Human-readable description of the behavior
  #[serde(default)]
  pub description: String,
  /// How to verify this behavior
  #[serde(default)]
  pub verification: Option<Verification>,
  /// Pre-conditions for this behavior
  #[serde(default)]
  pub preconditions: Vec<String>,
  /// Post-conditions after this behavior
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
    if !is_valid_behavior_name(&name) {
      return Err(TypeError::InvalidBehaviorName(name));
    }
    Ok(Self {
      name,
      description: String::new(),
      verification: None,
      preconditions: Vec::new(),
      postconditions: Vec::new(),
    })
  }

  /// Builder method to set description
  #[must_use]
  pub fn with_description(self, desc: String) -> Self {
    Self {
      description: desc,
      ..self
    }
  }

  /// Builder method to set verification
  #[must_use]
  pub fn with_verification(self, verification: Verification) -> Self {
    Self {
      verification: Some(verification),
      ..self
    }
  }

  /// Add a pre-condition
  pub fn add_precondition(&mut self, condition: String) -> &mut Self {
    self.preconditions.push(condition);
    self
  }

  /// Add a post-condition
  pub fn add_postcondition(&mut self, condition: String) -> &mut Self {
    self.postconditions.push(condition);
    self
  }

  /// Validate the behavior
  ///
  /// # Errors
  /// Returns appropriate `TypeError` variant if validation fails
  pub fn validate(&self) -> Result<(), TypeError> {
    if self.preconditions.len() > MAX_PRECONDITIONS {
      return Err(TypeError::TooManyPreconditions(
        self.name.clone(),
        self.preconditions.len(),
        MAX_PRECONDITIONS,
      ));
    }
    if self.postconditions.len() > MAX_POSTCONDITIONS {
      return Err(TypeError::TooManyPostconditions(
        self.name.clone(),
        self.postconditions.len(),
        MAX_POSTCONDITIONS,
      ));
    }
    Ok(())
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
  use super::Behavior;
  use crate::intent::types::TypeError;

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
  fn test_behavior_validate_too_many_preconditions() {
    let mut behavior = Behavior::new("test".to_string()).unwrap();
    for i in 0..21 {
      behavior.preconditions.push(format!("precondition_{}", i));
    }

    let result = behavior.validate();
    assert!(matches!(result, Err(TypeError::TooManyPreconditions(_, _, _))));
  }

  #[test]
  fn test_behavior_validate_too_many_postconditions() {
    let mut behavior = Behavior::new("test".to_string()).unwrap();
    for i in 0..21 {
      behavior.postconditions.push(format!("postcondition_{}", i));
    }

    let result = behavior.validate();
    assert!(matches!(result, Err(TypeError::TooManyPostconditions(_, _, _))));
  }
}
