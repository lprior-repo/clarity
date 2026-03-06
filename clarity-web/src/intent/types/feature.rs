#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::{Behavior, TypeError};

/// Feature - a named collection of behaviors
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Feature {
  /// Unique feature name
  pub name: String,
  /// Human-readable description
  #[serde(default)]
  pub description: String,
  /// Behaviors that define this feature
  #[serde(default)]
  pub behaviors: Vec<Behavior>,
  /// Other features this feature depends on
  #[serde(default)]
  pub depends_on: Vec<String>,
}

impl Feature {
  /// Create a new feature with the given name
  ///
  /// # Errors
  /// Returns `TypeError::EmptyName` if name is empty or whitespace-only
  pub fn new(name: String) -> Result<Self, TypeError> {
    if name.trim().is_empty() {
      return Err(TypeError::EmptyName);
    }
    Ok(Self {
      name,
      description: String::new(),
      behaviors: Vec::new(),
      depends_on: Vec::new(),
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

  /// Add a behavior to this feature
  ///
  /// # Errors
  /// Returns `TypeError::DuplicateBehavior` if a behavior with the same name already exists
  pub fn add_behavior(&mut self, behavior: Behavior) -> Result<&mut Self, TypeError> {
    if self.behaviors.iter().any(|b| b.name == behavior.name) {
      return Err(TypeError::DuplicateBehavior(
        behavior.name,
        self.name.clone(),
      ));
    }
    self.behaviors.push(behavior);
    Ok(self)
  }

  /// Add a dependency on another feature
  pub fn add_dependency(&mut self, feature_name: String) -> &mut Self {
    if !self.depends_on.contains(&feature_name) {
      self.depends_on.push(feature_name);
    }
    self
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {
  use super::Feature;
  use crate::intent::types::{Behavior, TypeError};

  #[test]
  fn test_feature_new_valid() {
    let feature_result = Feature::new("user-auth".to_string());
    assert!(feature_result.is_ok());

    let feature = match feature_result {
      Ok(value) => value,
      Err(_) => return,
    };

    assert_eq!(feature.name, "user-auth");
  }

  #[test]
  fn test_feature_new_empty_name() {
    let result = Feature::new(String::new());
    assert!(matches!(result, Err(TypeError::EmptyName)));
  }

  #[test]
  fn test_feature_add_behavior_duplicate() {
    let mut feature = match Feature::new("auth".to_string()) {
      Ok(value) => value,
      Err(_) => return,
    };

    let behavior1 = match Behavior::new("login".to_string()) {
      Ok(value) => value,
      Err(_) => return,
    };

    let behavior2 = match Behavior::new("login".to_string()) {
      Ok(value) => value,
      Err(_) => return,
    };

    let result1 = feature.add_behavior(behavior1);
    assert!(result1.is_ok());

    let result2 = feature.add_behavior(behavior2);
    assert!(matches!(result2, Err(TypeError::DuplicateBehavior(_, _))));
  }

  #[test]
  fn test_serde_roundtrip_feature() {
    let feature = match Feature::new("auth".to_string()) {
      Ok(value) => value.with_description("Authentication".to_string()),
      Err(_) => return,
    };

    let json_result = serde_json::to_string(&feature);
    assert!(json_result.is_ok());

    let json = match json_result {
      Ok(value) => value,
      Err(_) => return,
    };

    let parsed_result: Result<Feature, _> = serde_json::from_str(&json);
    assert!(parsed_result.is_ok());

    let parsed = match parsed_result {
      Ok(value) => value,
      Err(_) => return,
    };

    assert_eq!(feature, parsed);
  }
}
