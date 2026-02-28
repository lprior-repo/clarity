#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::{Behavior, FeatureDependency, FeatureName, TypeError};

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
    let validated_name = FeatureName::parse(name).map_err(|_| TypeError::EmptyName)?;
    Ok(Self {
      name: validated_name.into(),
      description: String::new(),
      behaviors: Vec::new(),
      depends_on: Vec::new(),
    })
  }

  /// Create a feature from a validated `FeatureName`.
  ///
  /// This constructor accepts a pre-validated name, avoiding redundant validation.
  #[must_use]
  pub fn from_validated_name(name: FeatureName) -> Self {
    Self {
      name: name.into(),
      description: String::new(),
      behaviors: Vec::new(),
      depends_on: Vec::new(),
    }
  }

  /// Get the feature name as a validated `FeatureName`.
  ///
  /// Returns `None` if the name is invalid (should not happen for well-constructed features).
  #[must_use]
  pub fn validated_name(&self) -> Option<FeatureName> {
    FeatureName::parse(self.name.clone()).ok()
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

  /// Add a validated dependency on another feature.
  ///
  /// This method accepts a pre-validated `FeatureDependency`.
  pub fn add_validated_dependency(&mut self, dependency: FeatureDependency) -> &mut Self {
    let dep_str: String = dependency.into();
    if !self.depends_on.contains(&dep_str) {
      self.depends_on.push(dep_str);
    }
    self
  }

  /// Get dependencies as validated `FeatureDependency` values.
  ///
  /// Invalid dependencies are filtered out.
  #[must_use]
  pub fn validated_dependencies(&self) -> Vec<FeatureDependency> {
    self
      .depends_on
      .iter()
      .filter_map(|s| FeatureDependency::parse(s.clone()).ok())
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::Feature;
  use crate::intent::types::{Behavior, FeatureDependency, FeatureName, TypeError};

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
  fn test_feature_from_validated_name() {
    let name = match FeatureName::parse("auth".to_string()) {
      Ok(n) => n,
      Err(_) => return,
    };
    let feature = Feature::from_validated_name(name);
    assert_eq!(feature.name, "auth");
  }

  #[test]
  fn test_feature_validated_name() {
    let feature = match Feature::new("user-auth".to_string()) {
      Ok(f) => f,
      Err(_) => return,
    };
    let validated = feature.validated_name();
    assert!(validated.is_some());
    let validated = validated.unwrap();
    assert_eq!(validated.as_str(), "user-auth");
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
  fn test_feature_add_validated_dependency() {
    let mut feature = match Feature::new("users".to_string()) {
      Ok(f) => f,
      Err(_) => return,
    };
    let dep = match FeatureDependency::parse("auth".to_string()) {
      Ok(d) => d,
      Err(_) => return,
    };
    feature.add_validated_dependency(dep);
    assert_eq!(feature.depends_on.len(), 1);
    assert_eq!(feature.depends_on[0], "auth");
  }

  #[test]
  fn test_feature_validated_dependencies() {
    let mut feature = match Feature::new("users".to_string()) {
      Ok(f) => f,
      Err(_) => return,
    };
    // Add valid dependency
    feature.add_dependency("auth".to_string());
    // Add empty dependency (invalid)
    feature.depends_on.push("".to_string());

    let validated = feature.validated_dependencies();
    assert_eq!(validated.len(), 1);
    assert_eq!(validated[0].as_str(), "auth");
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
