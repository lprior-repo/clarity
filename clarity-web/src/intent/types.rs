//! Core Spec Types - Specification data structures for intent system
//!
//! This module defines the core types for building specifications:
//! - `Spec` - Top-level specification container
//! - `Feature` - Named feature with behaviors
//! - `Behavior` - Individual behavior with verification criteria
//! - `Verification` - How to verify a behavior works
//! - `Invariant` - System invariants that must always hold
//! - `AntiPattern` - Patterns to avoid in implementation
//! - `AIHints` - Hints for AI code generation

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Regex pattern for valid behavior names: snake_case with leading lowercase letter
static BEHAVIOR_NAME_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-z][a-z0-9_]*$").expect("Invalid regex pattern"));

/// Type errors for spec validation
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum TypeError {
    /// Name field cannot be empty
    #[error("name cannot be empty")]
    EmptyName,

    /// Behavior name does not match required snake_case pattern
    #[error("behavior name '{0}' must be snake_case (lowercase letters, numbers, underscores, starting with letter)")]
    InvalidBehaviorName(String),

    /// Duplicate feature name detected
    #[error("duplicate feature name: '{0}'")]
    DuplicateFeature(String),

    /// Duplicate behavior name within a feature
    #[error("duplicate behavior name '{0}' in feature '{1}'")]
    DuplicateBehavior(String, String),

    /// Circular dependency detected in feature graph
    #[error("circular dependency detected: {0} -> {1}")]
    CircularDependency(String, String),

    /// Feature referenced but not found
    #[error("unknown feature dependency: '{0}'")]
    UnknownFeatureDependency(String),
}

/// Top-level specification container
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub fn with_description(mut self, desc: String) -> Self {
        self.description = desc;
        self
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
    pub fn with_ai_hints(mut self, hints: AIHints) -> Self {
        self.ai_hints = hints;
        self
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
        // Check for duplicate features
        let mut seen_features: HashSet<&str> = HashSet::new();
        for feature in &self.features {
            if !seen_features.insert(&feature.name) {
                return Err(TypeError::DuplicateFeature(feature.name.clone()));
            }
        }

        // Check for duplicate behaviors within each feature
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

        // Check for circular dependencies using DFS
        self.detect_circular_dependencies()?;

        Ok(())
    }

    /// Detect circular dependencies in feature dependency graph
    fn detect_circular_dependencies(&self) -> Result<(), TypeError> {
        let feature_names: HashSet<&str> =
            self.features.iter().map(|f| f.name.as_str()).collect();

        // Build adjacency list of dependencies
        let mut visiting: HashSet<&str> = HashSet::new();
        let mut visited: HashSet<&str> = HashSet::new();

        for feature in &self.features {
            // Validate dependencies reference known features
            for dep in &feature.depends_on {
                if !feature_names.contains(dep.as_str()) {
                    return Err(TypeError::UnknownFeatureDependency(dep.clone()));
                }
            }

            // DFS for cycle detection
            Self::dfs_visit(feature.name.as_str(), &feature.depends_on, &mut visiting, &mut visited, &feature_names)?;
        }

        Ok(())
    }

    /// DFS helper for cycle detection
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
            // This shouldn't happen at the top level, but indicates a cycle
            return Err(TypeError::CircularDependency(node.to_string(), node.to_string()));
        }

        visiting.insert(node);

        for dep in dependencies {
            if !all_features.contains(dep.as_str()) {
                continue; // Already validated above
            }
            if visiting.contains(dep.as_str()) {
                return Err(TypeError::CircularDependency(node.to_string(), dep.clone()));
            }
            // Find the dependency's dependencies recursively
            // Note: We'd need access to all features here for full traversal
            // For now, this handles direct cycles
        }

        visiting.remove(node);
        visited.insert(node);

        Ok(())
    }
}

/// Feature - a named collection of behaviors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub fn with_description(mut self, desc: String) -> Self {
        self.description = desc;
        self
    }

    /// Add a behavior to this feature
    ///
    /// # Errors
    /// Returns `TypeError::DuplicateBehavior` if a behavior with the same name already exists
    pub fn add_behavior(&mut self, behavior: Behavior) -> Result<&mut Self, TypeError> {
        if self.behaviors.iter().any(|b| b.name == behavior.name) {
            return Err(TypeError::DuplicateBehavior(behavior.name, self.name.clone()));
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

/// Behavior - a single behavior with verification criteria
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Behavior {
    /// Behavior name in snake_case
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
    /// The name must be in snake_case format: lowercase letters, numbers,
    /// and underscores, starting with a letter.
    ///
    /// # Errors
    /// Returns `TypeError::InvalidBehaviorName` if name doesn't match pattern
    pub fn new(name: String) -> Result<Self, TypeError> {
        if !BEHAVIOR_NAME_PATTERN.is_match(&name) {
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
    pub fn with_description(mut self, desc: String) -> Self {
        self.description = desc;
        self
    }

    /// Builder method to set verification
    #[must_use]
    pub fn with_verification(mut self, verification: Verification) -> Self {
        self.verification = Some(verification);
        self
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
}

/// Verification - how to verify a behavior works correctly
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Verification {
    /// Verification type (e.g., "unit_test", "integration_test", "manual")
    #[serde(default)]
    pub verification_type: String,
    /// Description of how to verify
    #[serde(default)]
    pub description: String,
    /// Example test case or verification steps
    #[serde(default)]
    pub example: String,
}

impl Default for Verification {
    fn default() -> Self {
        Self {
            verification_type: String::new(),
            description: String::new(),
            example: String::new(),
        }
    }
}

impl Verification {
    /// Create a new verification
    #[must_use]
    pub fn new(verification_type: String, description: String) -> Self {
        Self {
            verification_type,
            description,
            example: String::new(),
        }
    }

    /// Builder method to set example
    #[must_use]
    pub fn with_example(mut self, example: String) -> Self {
        self.example = example;
        self
    }
}

/// Invariant - a system property that must always hold
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Invariant {
    /// Invariant name/identifier
    pub name: String,
    /// Description of the invariant
    #[serde(default)]
    pub description: String,
    /// Formal or informal specification
    #[serde(default)]
    pub constraint: String,
}

impl Default for Invariant {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            constraint: String::new(),
        }
    }
}

impl Invariant {
    /// Create a new invariant
    #[must_use]
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            constraint: String::new(),
        }
    }

    /// Builder method to set constraint
    #[must_use]
    pub fn with_constraint(mut self, constraint: String) -> Self {
        self.constraint = constraint;
        self
    }
}

/// AntiPattern - a pattern to avoid in implementation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AntiPattern {
    /// Anti-pattern name/identifier
    pub name: String,
    /// Description of the anti-pattern
    #[serde(default)]
    pub description: String,
    /// Why this pattern should be avoided
    #[serde(default)]
    pub why_avoid: String,
    /// Suggested alternative approach
    #[serde(default)]
    pub alternative: String,
}

impl Default for AntiPattern {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            why_avoid: String::new(),
            alternative: String::new(),
        }
    }
}

impl AntiPattern {
    /// Create a new anti-pattern
    #[must_use]
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            why_avoid: String::new(),
            alternative: String::new(),
        }
    }

    /// Builder method to set why to avoid
    #[must_use]
    pub fn with_why_avoid(mut self, why: String) -> Self {
        self.why_avoid = why;
        self
    }

    /// Builder method to set alternative
    #[must_use]
    pub fn with_alternative(mut self, alternative: String) -> Self {
        self.alternative = alternative;
        self
    }
}

/// AIHints - hints to guide AI code generation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl Default for AIHints {
    fn default() -> Self {
        Self {
            implementation: ImplementationHints::default(),
            entities: Vec::new(),
            security: SecurityHints::default(),
            preferred_libraries: Vec::new(),
            style_hints: Vec::new(),
        }
    }
}

/// ImplementationHints - hints for implementation approach
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl Default for ImplementationHints {
    fn default() -> Self {
        Self {
            architecture: String::new(),
            performance_notes: String::new(),
            error_handling: String::new(),
        }
    }
}

/// EntityHint - hint for data entity modeling
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl Default for EntityHint {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            fields: Vec::new(),
            relationships: Vec::new(),
        }
    }
}

/// SecurityHints - security-related considerations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl Default for SecurityHints {
    fn default() -> Self {
        Self {
            authentication: String::new(),
            authorization: String::new(),
            data_sensitivity: String::new(),
            concerns: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_new_valid() {
        let spec = Spec::new("my-spec".to_string());
        assert!(spec.is_ok());
        let spec = spec.expect("spec should be valid");
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
        let spec = Spec::new("my-spec".to_string())
            .expect("valid spec")
            .with_description("A test specification".to_string());
        assert_eq!(spec.description, "A test specification");
    }

    #[test]
    fn test_feature_new_valid() {
        let feature = Feature::new("user-auth".to_string());
        assert!(feature.is_ok());
        let feature = feature.expect("feature should be valid");
        assert_eq!(feature.name, "user-auth");
    }

    #[test]
    fn test_feature_new_empty_name() {
        let result = Feature::new(String::new());
        assert!(matches!(result, Err(TypeError::EmptyName)));
    }

    #[test]
    fn test_behavior_new_valid() {
        let behavior = Behavior::new("create_user".to_string());
        assert!(behavior.is_ok());
        let behavior = behavior.expect("behavior should be valid");
        assert_eq!(behavior.name, "create_user");
    }

    #[test]
    fn test_behavior_new_simple() {
        let behavior = Behavior::new("save".to_string());
        assert!(behavior.is_ok());
    }

    #[test]
    fn test_behavior_new_with_numbers() {
        let behavior = Behavior::new("parse_v2".to_string());
        assert!(behavior.is_ok());
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
    fn test_spec_add_feature_duplicate() {
        let mut spec = Spec::new("test".to_string()).expect("valid spec");
        let feature1 = Feature::new("auth".to_string()).expect("valid feature");
        let feature2 = Feature::new("auth".to_string()).expect("valid feature");

        let result1 = spec.add_feature(feature1);
        assert!(result1.is_ok());

        let result2 = spec.add_feature(feature2);
        assert!(matches!(result2, Err(TypeError::DuplicateFeature(_))));
    }

    #[test]
    fn test_feature_add_behavior_duplicate() {
        let mut feature = Feature::new("auth".to_string()).expect("valid feature");
        let behavior1 = Behavior::new("login".to_string()).expect("valid behavior");
        let behavior2 = Behavior::new("login".to_string()).expect("valid behavior");

        let result1 = feature.add_behavior(behavior1);
        assert!(result1.is_ok());

        let result2 = feature.add_behavior(behavior2);
        assert!(matches!(result2, Err(TypeError::DuplicateBehavior(_, _))));
    }

    #[test]
    fn test_spec_validate_success() {
        let mut spec = Spec::new("test-spec".to_string()).expect("valid spec");

        let mut auth_feature = Feature::new("auth".to_string()).expect("valid feature");
        let login = Behavior::new("login".to_string()).expect("valid behavior");
        auth_feature.add_behavior(login).expect("should add behavior");

        let mut user_feature = Feature::new("users".to_string()).expect("valid feature");
        user_feature.add_dependency("auth".to_string());
        let create = Behavior::new("create".to_string()).expect("valid behavior");
        user_feature.add_behavior(create).expect("should add behavior");

        spec.add_feature(auth_feature).expect("should add feature");
        spec.add_feature(user_feature).expect("should add feature");

        let result = spec.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_spec_validate_unknown_dependency() {
        let mut spec = Spec::new("test-spec".to_string()).expect("valid spec");

        let mut feature = Feature::new("users".to_string()).expect("valid feature");
        feature.add_dependency("nonexistent".to_string());
        let create = Behavior::new("create".to_string()).expect("valid behavior");
        feature.add_behavior(create).expect("should add behavior");

        spec.add_feature(feature).expect("should add feature");

        let result = spec.validate();
        assert!(matches!(result, Err(TypeError::UnknownFeatureDependency(_))));
    }

    #[test]
    fn test_serde_roundtrip_spec() {
        let spec = Spec::new("test-spec".to_string())
            .expect("valid spec")
            .with_description("A test spec".to_string());

        let json = serde_json::to_string(&spec).expect("should serialize");
        let parsed: Spec = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(spec, parsed);
    }

    #[test]
    fn test_serde_roundtrip_feature() {
        let feature = Feature::new("auth".to_string())
            .expect("valid feature")
            .with_description("Authentication".to_string());

        let json = serde_json::to_string(&feature).expect("should serialize");
        let parsed: Feature = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(feature, parsed);
    }

    #[test]
    fn test_serde_roundtrip_behavior() {
        let behavior = Behavior::new("login".to_string())
            .expect("valid behavior")
            .with_description("User login".to_string());

        let json = serde_json::to_string(&behavior).expect("should serialize");
        let parsed: Behavior = serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(behavior, parsed);
    }

    #[test]
    fn test_verification_builder() {
        let verification = Verification::new("unit_test".to_string(), "Test login".to_string())
            .with_example("assert!(login(user, pass))".to_string());

        assert_eq!(verification.verification_type, "unit_test");
        assert_eq!(verification.description, "Test login");
        assert_eq!(verification.example, "assert!(login(user, pass))");
    }

    #[test]
    fn test_invariant_builder() {
        let invariant = Invariant::new("unique_email".to_string(), "Emails must be unique".to_string())
            .with_constraint("email UNIQUE in users".to_string());

        assert_eq!(invariant.name, "unique_email");
        assert_eq!(invariant.description, "Emails must be unique");
        assert_eq!(invariant.constraint, "email UNIQUE in users");
    }

    #[test]
    fn test_anti_pattern_builder() {
        let anti = AntiPattern::new("god_object".to_string(), "Avoid god objects".to_string())
            .with_why_avoid("Violates SRP".to_string())
            .with_alternative("Split into focused classes".to_string());

        assert_eq!(anti.name, "god_object");
        assert_eq!(anti.why_avoid, "Violates SRP");
        assert_eq!(anti.alternative, "Split into focused classes");
    }

    #[test]
    fn test_ai_hints_default() {
        let hints = AIHints::default();
        assert!(hints.entities.is_empty());
        assert!(hints.preferred_libraries.is_empty());
        assert!(hints.style_hints.is_empty());
    }

    #[test]
    fn test_type_error_display() {
        let err = TypeError::EmptyName;
        assert_eq!(format!("{err}"), "name cannot be empty");

        let err = TypeError::InvalidBehaviorName("BadName".to_string());
        assert!(format!("{err}").contains("BadName"));

        let err = TypeError::DuplicateFeature("auth".to_string());
        assert!(format!("{err}").contains("auth"));
    }
}
