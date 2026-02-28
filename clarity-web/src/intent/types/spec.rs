#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::{AIHints, AntiPattern, Feature, Invariant, SpecName, TypeError};

/// Top-level specification container
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spec {
  /// Unique specification name
  pub name: String,
  /// Human-readable description
  #[serde(default)]
  pub description: String,
  /// Target audience for this specification
  #[serde(default)]
  pub audience: String,
  /// Version string for this specification
  #[serde(default)]
  pub version: String,
  /// Success criteria for this specification
  #[serde(default)]
  pub success_criteria: Vec<String>,
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
    let validated_name = SpecName::parse(name).map_err(|_| TypeError::EmptyName)?;
    Ok(Self {
      name: validated_name.into(),
      description: String::new(),
      audience: String::new(),
      version: String::new(),
      success_criteria: Vec::new(),
      features: Vec::new(),
      invariants: Vec::new(),
      anti_patterns: Vec::new(),
      ai_hints: AIHints::default(),
    })
  }

  /// Create a spec from a validated `SpecName`.
  ///
  /// This constructor accepts a pre-validated name, avoiding redundant validation.
  #[must_use]
  pub fn from_validated_name(name: SpecName) -> Self {
    Self {
      name: name.into(),
      description: String::new(),
      audience: String::new(),
      version: String::new(),
      success_criteria: Vec::new(),
      features: Vec::new(),
      invariants: Vec::new(),
      anti_patterns: Vec::new(),
      ai_hints: AIHints::default(),
    }
  }

  /// Get the spec name as a validated `SpecName`.
  ///
  /// Returns `None` if the name is invalid (should not happen for well-constructed specs).
  #[must_use]
  pub fn validated_name(&self) -> Option<SpecName> {
    SpecName::parse(self.name.clone()).ok()
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

    // Build dependency map for efficient lookup
    let dep_map: HashMap<&str, &Vec<String>> = self
      .features
      .iter()
      .map(|f| (f.name.as_str(), &f.depends_on))
      .collect();

    let mut visiting: HashSet<&str> = HashSet::new();
    let mut visited: HashSet<&str> = HashSet::new();

    for feature in &self.features {
      for dep in &feature.depends_on {
        if !feature_names.contains(dep.as_str()) {
          return Err(TypeError::UnknownFeatureDependency(dep.clone()));
        }
      }

      Self::dfs_visit(feature.name.as_str(), &dep_map, &mut visiting, &mut visited)?;
    }

    Ok(())
  }

  fn dfs_visit<'a>(
    node: &'a str,
    dep_map: &HashMap<&'a str, &'a Vec<String>>,
    visiting: &mut HashSet<&'a str>,
    visited: &mut HashSet<&'a str>,
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

    // Get this node's dependencies and recursively visit each one
    if let Some(dependencies) = dep_map.get(node) {
      for dep in *dependencies {
        if visiting.contains(dep.as_str()) {
          return Err(TypeError::CircularDependency(node.to_string(), dep.clone()));
        }
        // Recursively visit the dependency to traverse its own dependencies
        Self::dfs_visit(dep.as_str(), dep_map, visiting, visited)?;
      }
    }

    visiting.remove(node);
    visited.insert(node);

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::Spec;
  use crate::intent::types::{Behavior, Feature, SpecName, TypeError};

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
  fn test_spec_from_validated_name() {
    let name = match SpecName::parse("my-spec".to_string()) {
      Ok(n) => n,
      Err(_) => return,
    };
    let spec = Spec::from_validated_name(name);
    assert_eq!(spec.name, "my-spec");
  }

  #[test]
  fn test_spec_validated_name() {
    let spec = match Spec::new("my-spec".to_string()) {
      Ok(s) => s,
      Err(_) => return,
    };
    let validated = spec.validated_name();
    assert!(validated.is_some());
    let validated = validated.unwrap();
    assert_eq!(validated.as_str(), "my-spec");
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
  fn test_spec_validate_direct_cycle() {
    // Test A -> B -> A cycle detection
    let mut spec = match Spec::new("test-spec".to_string()) {
      Ok(value) => value,
      Err(_) => return,
    };

    // Feature A depends on B
    let mut feature_a = match Feature::new("feature_a".to_string()) {
      Ok(value) => value,
      Err(_) => return,
    };
    feature_a.add_dependency("feature_b".to_string());

    // Feature B depends on A (creates cycle)
    let mut feature_b = match Feature::new("feature_b".to_string()) {
      Ok(value) => value,
      Err(_) => return,
    };
    feature_b.add_dependency("feature_a".to_string());

    let add_a_result = spec.add_feature(feature_a);
    assert!(add_a_result.is_ok());

    let add_b_result = spec.add_feature(feature_b);
    assert!(add_b_result.is_ok());

    let result = spec.validate();
    assert!(matches!(result, Err(TypeError::CircularDependency(_, _))));
  }

  #[test]
  fn test_spec_validate_multi_hop_cycle() {
    // Test A -> B -> C -> A cycle detection (3-hop cycle)
    let mut spec = match Spec::new("test-spec".to_string()) {
      Ok(value) => value,
      Err(_) => return,
    };

    // Feature A depends on B
    let mut feature_a = match Feature::new("feature_a".to_string()) {
      Ok(value) => value,
      Err(_) => return,
    };
    feature_a.add_dependency("feature_b".to_string());

    // Feature B depends on C
    let mut feature_b = match Feature::new("feature_b".to_string()) {
      Ok(value) => value,
      Err(_) => return,
    };
    feature_b.add_dependency("feature_c".to_string());

    // Feature C depends on A (creates 3-hop cycle)
    let mut feature_c = match Feature::new("feature_c".to_string()) {
      Ok(value) => value,
      Err(_) => return,
    };
    feature_c.add_dependency("feature_a".to_string());

    let add_a_result = spec.add_feature(feature_a);
    assert!(add_a_result.is_ok());

    let add_b_result = spec.add_feature(feature_b);
    assert!(add_b_result.is_ok());

    let add_c_result = spec.add_feature(feature_c);
    assert!(add_c_result.is_ok());

    let result = spec.validate();
    assert!(matches!(result, Err(TypeError::CircularDependency(_, _))));
  }
}
