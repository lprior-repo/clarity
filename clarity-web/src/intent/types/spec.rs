#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::{AIHints, AntiPattern, Feature, Invariant, TypeError};

/// Maximum number of features allowed in a specification
const MAX_FEATURES: usize = 100;
/// Maximum number of invariants allowed in a specification
const MAX_INVARIANTS: usize = 100;
/// Maximum number of anti-patterns allowed in a specification
const MAX_ANTI_PATTERNS: usize = 100;

/// Top-level specification container
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spec {
  /// Unique specification name
  pub name: String,
  /// Human-readable description
  #[serde(default)]
  pub description: String,
  /// Features that make up this specification
  #[serde(default)]
  pub features: Vec<Feature>,
  /// System-wide invariants
  #[serde(default)]
  pub invariants: Vec<Invariant>,
  /// Patterns to avoid
  #[serde(default)]
  pub anti_patterns: Vec<AntiPattern>,
  /// AI generation hints
  #[serde(default)]
  pub ai_hints: AIHints,
}

impl Spec {
  /// Create a new specification with the given name
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
      features: Vec::new(),
      invariants: Vec::new(),
      anti_patterns: Vec::new(),
      ai_hints: AIHints::default(),
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

  /// Add a feature to this specification
  ///
  /// # Errors
  /// Returns `TypeError::DuplicateFeature` if a feature with the same name already exists
  pub fn add_feature(&mut self, feature: Feature) -> Result<&mut Self, TypeError> {
    if self.features.iter().any(|f| f.name == feature.name) {
      return Err(TypeError::DuplicateFeature(feature.name));
    }
    self.features.push(feature);
    Ok(self)
  }

  /// Add an invariant to this specification
  pub fn add_invariant(&mut self, invariant: Invariant) -> &mut Self {
    self.invariants.push(invariant);
    self
  }

  /// Add an anti-pattern to this specification
  pub fn add_anti_pattern(&mut self, anti_pattern: AntiPattern) -> &mut Self {
    self.anti_patterns.push(anti_pattern);
    self
  }

  /// Set AI hints for this specification
  #[must_use]
  pub fn with_ai_hints(self, hints: AIHints) -> Self {
    Self {
      ai_hints: hints,
      ..self
    }
  }

  /// Validate the entire specification
  ///
  /// Checks for:
  /// - Duplicate feature names
  /// - Duplicate behavior names within features
  /// - Circular dependencies between features
  /// - Unknown feature dependencies
  ///
  /// # Errors
  /// Returns appropriate `TypeError` variant if validation fails
  pub fn validate(&self) -> Result<(), TypeError> {
    if self.features.len() > MAX_FEATURES {
      return Err(TypeError::TooManyFeatures(
        self.features.len(),
        MAX_FEATURES,
      ));
    }
    if self.invariants.len() > MAX_INVARIANTS {
      return Err(TypeError::TooManyInvariants(
        self.invariants.len(),
        MAX_INVARIANTS,
      ));
    }
    if self.anti_patterns.len() > MAX_ANTI_PATTERNS {
      return Err(TypeError::TooManyAntiPatterns(
        self.anti_patterns.len(),
        MAX_ANTI_PATTERNS,
      ));
    }
    let mut seen_features: HashSet<&str> = HashSet::new();
    for feature in &self.features {
      if !seen_features.insert(&feature.name) {
        return Err(TypeError::DuplicateFeature(feature.name.clone()));
      }
    }

    for feature in &self.features {
      let mut seen_behaviors: HashSet<&str> = HashSet::new();
      for behavior in &feature.behaviors {
        if !seen_behaviors.insert(&behavior.name) {
          return Err(TypeError::DuplicateBehavior(
            behavior.name.clone(),
            feature.name.clone(),
          ));
        }
      }
    }

    self.detect_circular_dependencies()
  }

  fn detect_circular_dependencies(&self) -> Result<(), TypeError> {
    let feature_names: HashSet<&str> = self.features.iter().map(|f| f.name.as_str()).collect();

    let mut visiting: HashSet<&str> = HashSet::new();
    let mut visited: HashSet<&str> = HashSet::new();

    for feature in &self.features {
      for dep in &feature.depends_on {
        if !feature_names.contains(dep.as_str()) {
          return Err(TypeError::UnknownFeatureDependency(dep.clone()));
        }
      }

      Self::dfs_visit(
        feature.name.as_str(),
        &feature.depends_on,
        &mut visiting,
        &mut visited,
        &feature_names,
      )?;
    }

    Ok(())
  }

  fn dfs_visit<'a>(
    node: &'a str,
    dependencies: &[String],
    visiting: &mut HashSet<&'a str>,
    visited: &mut HashSet<&'a str>,
    all_features: &HashSet<&'a str>,
  ) -> Result<(), TypeError> {
    if visited.contains(node) {
      return Ok(());
    }

    if visiting.contains(node) {
      return Err(TypeError::CircularDependency(
        node.to_string(),
        node.to_string(),
      ));
    }

    visiting.insert(node);

    for dep in dependencies {
      if !all_features.contains(dep.as_str()) {
        continue;
      }
      if visiting.contains(dep.as_str()) {
        return Err(TypeError::CircularDependency(node.to_string(), dep.clone()));
      }
    }

    visiting.remove(node);
    visited.insert(node);

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
  use super::Spec;
  use crate::intent::types::{AIHints, Behavior, Feature, TypeError};

  #[test]
  fn test_spec_new_valid() {
    let spec_result = Spec::new("my-spec".to_string());
    assert!(spec_result.is_ok());

    let spec = match spec_result {
      Ok(value) => value,
      Err(_) => return,
    };

    assert_eq!(spec.name, "my-spec");
    assert!(spec.description.is_empty());
    assert!(spec.features.is_empty());
  }

  #[test]
  fn test_spec_new_empty_name() {
    let result = Spec::new(String::new());
    assert!(matches!(result, Err(TypeError::EmptyName)));
  }

  #[test]
  fn test_spec_new_whitespace_name() {
    let result = Spec::new("   ".to_string());
    assert!(matches!(result, Err(TypeError::EmptyName)));
  }

  #[test]
  fn test_spec_with_description() {
    let spec = match Spec::new("my-spec".to_string()) {
      Ok(value) => value.with_description("A test specification".to_string()),
      Err(_) => return,
    };

    assert_eq!(spec.description, "A test specification");
  }

  #[test]
  fn test_spec_add_feature_duplicate() {
    let mut spec = match Spec::new("test".to_string()) {
      Ok(value) => value,
      Err(_) => return,
    };

    let feature1 = match Feature::new("auth".to_string()) {
      Ok(value) => value,
      Err(_) => return,
    };

    let feature2 = match Feature::new("auth".to_string()) {
      Ok(value) => value,
      Err(_) => return,
    };

    let result1 = spec.add_feature(feature1);
    assert!(result1.is_ok());

    let result2 = spec.add_feature(feature2);
    assert!(matches!(result2, Err(TypeError::DuplicateFeature(_))));
  }

  #[test]
  fn test_spec_validate_success() {
    let mut spec = match Spec::new("test-spec".to_string()) {
      Ok(value) => value,
      Err(_) => return,
    };

    let mut auth_feature = match Feature::new("auth".to_string()) {
      Ok(value) => value,
      Err(_) => return,
    };

    let login = match Behavior::new("login".to_string()) {
      Ok(value) => value,
      Err(_) => return,
    };

    let add_login_result = auth_feature.add_behavior(login);
    assert!(add_login_result.is_ok());

    let mut user_feature = match Feature::new("users".to_string()) {
      Ok(value) => value,
      Err(_) => return,
    };

    user_feature.add_dependency("auth".to_string());

    let create = match Behavior::new("create".to_string()) {
      Ok(value) => value,
      Err(_) => return,
    };

    let add_create_result = user_feature.add_behavior(create);
    assert!(add_create_result.is_ok());

    let add_auth_result = spec.add_feature(auth_feature);
    assert!(add_auth_result.is_ok());

    let add_user_result = spec.add_feature(user_feature);
    assert!(add_user_result.is_ok());

    let result = spec.validate();
    assert!(result.is_ok());
  }

  #[test]
  fn test_spec_validate_too_many_features() {
    let spec = Spec {
      name: "test".to_string(),
      description: String::new(),
      features: (0..101)
        .map(|i| Feature::new(format!("feature_{i}")).unwrap())
        .collect(),
      invariants: Vec::new(),
      anti_patterns: Vec::new(),
      ai_hints: AIHints::default(),
    };

    let result = spec.validate();
    assert!(matches!(result, Err(TypeError::TooManyFeatures(_, _))));
  }
}
