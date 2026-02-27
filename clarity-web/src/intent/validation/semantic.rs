//! Semantic Validator - WP31
//!
//! Cross-reference and consistency validation for specifications.
//!
//! ## Validation Checks
//!
//! - **Cross-reference validation**: Ensures all behavior references (preconditions, postconditions)
//!   refer to valid behaviors within the spec
//! - **Terminology consistency**: Checks for consistent naming and terminology across features
//! - **Semantic constraint validation**: Validates that behaviors satisfy semantic constraints
//! - **Dependency consistency**: Ensures feature dependencies reference valid features
//!
//! ## Design Principles
//!
//! - Zero panics: All operations return `Result<T, E>`
//! - Deterministic: Same input always produces same output
//! - Pure functions: No side effects, only analyzes input

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::intent::types::Spec;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Newtype wrapper for validated spec names
///
/// Following Scott Wlaschin's DDD principle of using types to make
/// invalid states unrepresentable. A value of this type is guaranteed
/// to have passed spec name validation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidatedSpecName(String);

impl ValidatedSpecName {
  /// Get the inner string value
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }

  /// Create a new validated spec name without validation
  ///
  /// # Safety
  ///
  /// This should only be called after proper validation has been performed.
  #[must_use]
  pub const fn unchecked_new(name: String) -> Self {
    Self(name)
  }
}

impl From<ValidatedSpecName> for String {
  fn from(value: ValidatedSpecName) -> Self {
    value.0
  }
}

impl AsRef<str> for ValidatedSpecName {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

/// Newtype wrapper for validated feature names
///
/// Following Scott Wlaschin's DDD principle of using types to make
/// invalid states unrepresentable. A value of this type is guaranteed
/// to have passed feature name validation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidatedFeatureName(String);

impl ValidatedFeatureName {
  /// Get the inner string value
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }

  /// Create a new validated feature name without validation
  ///
  /// # Safety
  ///
  /// This should only be called after proper validation has been performed.
  #[must_use]
  pub const fn unchecked_new(name: String) -> Self {
    Self(name)
  }
}

impl From<ValidatedFeatureName> for String {
  fn from(value: ValidatedFeatureName) -> Self {
    value.0
  }
}

impl AsRef<str> for ValidatedFeatureName {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

/// Newtype wrapper for validated behavior names
///
/// Following Scott Wlaschin's DDD principle of using types to make
/// invalid states unrepresentable. A value of this type is guaranteed
/// to have passed behavior name validation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidatedBehaviorName(String);

impl ValidatedBehaviorName {
  /// Get the inner string value
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }

  /// Create a new validated behavior name without validation
  ///
  /// # Safety
  ///
  /// This should only be called after proper validation has been performed.
  #[must_use]
  pub const fn unchecked_new(name: String) -> Self {
    Self(name)
  }
}

impl From<ValidatedBehaviorName> for String {
  fn from(value: ValidatedBehaviorName) -> Self {
    value.0
  }
}

impl AsRef<str> for ValidatedBehaviorName {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

/// Errors that can occur during semantic validation
///
/// Each error variant represents a specific category of semantic issue
/// that can be detected when validating a specification.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SemanticError {
  /// A behavior reference in preconditions/postconditions does not exist
  ///
  /// This occurs when a behavior references another behavior in its
  /// preconditions or postconditions, but the target behavior cannot be
  /// found in the specification.
  #[error("broken reference: behavior '{behavior}' references non-existent behavior '{target}' in {context}")]
  BrokenReference {
    /// The behavior that contains the invalid reference
    behavior: String,
    /// The behavior that was referenced but doesn't exist
    target: String,
    /// Where the reference was found (e.g., "precondition" or "postcondition")
    context: String,
  },

  /// A feature dependency references a non-existent feature
  ///
  /// This occurs when a feature declares a dependency on another feature
  /// that doesn't exist in the specification.
  #[error("broken dependency: feature '{feature}' depends on non-existent feature '{dependency}'")]
  BrokenDependency {
    /// The feature that declared the dependency
    feature: String,
    /// The feature that was depended on but doesn't exist
    dependency: String,
  },

  /// Inconsistent terminology detected
  ///
  /// This occurs when the validator detects naming inconsistencies,
  /// such as mixing kebab-case and `snake_case`, or using similar terms
  /// that should be unified.
  #[error("inconsistent terminology: '{term}' is used inconsistently - {details}")]
  InconsistentTerminology {
    /// The term that is used inconsistently
    term: String,
    /// Details about the inconsistency
    details: String,
  },

  /// Overlapping preconditions and postconditions
  ///
  /// This occurs when a behavior has the same condition in both
  /// its preconditions and postconditions, creating a logical inconsistency.
  #[error("behavior '{behavior}' has overlapping preconditions and postconditions: {overlaps:?}")]
  OverlappingPreconditions {
    /// The qualified behavior name (feature.behavior)
    behavior: String,
    /// The list of overlapping condition names
    overlaps: Vec<String>,
  },

  /// Dependency chain too deep
  ///
  /// This occurs when feature dependencies form a chain that is
  /// too deep, indicating tight coupling and complexity.
  #[error("dependency chain too deep: {depth} levels (max recommended: {max})")]
  DependencyChainTooDeep {
    /// The actual depth detected
    depth: usize,
    /// The maximum recommended depth
    max: usize,
  },

  /// Behavior with preconditions but no description
  ///
  /// This occurs when a behavior has dependencies (preconditions) but
  /// lacks a description explaining the dependency relationship.
  #[error("behavior '{behavior}' has preconditions but no description")]
  BehaviorWithPreconditionsNoDescription {
    /// The qualified behavior name (feature.behavior)
    behavior: String,
  },

  /// Empty specification name
  ///
  /// This occurs when the specification name is empty or contains only whitespace.
  #[error("empty spec name")]
  EmptySpecName,

  /// No features defined
  ///
  /// This occurs when a specification contains no features.
  #[error("spec has no features defined")]
  NoFeatures,
}

impl SemanticError {
  /// Create a broken reference error
  ///
  /// # Arguments
  ///
  /// * `behavior` - The behavior that contains the invalid reference
  /// * `target` - The behavior that was referenced but doesn't exist
  /// * `context` - Where the reference was found (e.g., "precondition" or "postcondition")
  ///
  /// # Returns
  ///
  /// A `SemanticError::BrokenReference` variant
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::validation::semantic::SemanticError;
  ///
  /// let error = SemanticError::broken_reference(
  ///     "auth.login".to_string(),
  ///     "auth.authenticate".to_string(),
  ///     "precondition".to_string()
  /// );
  /// ```
  #[allow(clippy::missing_const_for_fn)]
  #[must_use]
  pub fn broken_reference(behavior: String, target: String, context: String) -> Self {
    Self::BrokenReference {
      behavior,
      target,
      context,
    }
  }

  /// Create a broken dependency error
  ///
  /// # Arguments
  ///
  /// * `feature` - The feature that declared the dependency
  /// * `dependency` - The feature that was depended on but doesn't exist
  ///
  /// # Returns
  ///
  /// A `SemanticError::BrokenDependency` variant
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::validation::semantic::SemanticError;
  ///
  /// let error = SemanticError::broken_dependency(
  ///     "users".to_string(),
  ///     "auth".to_string()
  /// );
  /// ```
  #[allow(clippy::missing_const_for_fn)]
  #[must_use]
  pub fn broken_dependency(feature: String, dependency: String) -> Self {
    Self::BrokenDependency {
      feature,
      dependency,
    }
  }

  /// Create an inconsistent terminology error
  ///
  /// # Arguments
  ///
  /// * `term` - The term that is used inconsistently
  /// * `details` - Details about the inconsistency
  ///
  /// # Returns
  ///
  /// A `SemanticError::InconsistentTerminology` variant
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::validation::semantic::SemanticError;
  ///
  /// let error = SemanticError::inconsistent_terminology(
  ///     "naming convention".to_string(),
  ///     "mix of kebab-case and snake_case detected".to_string()
  /// );
  /// ```
  #[allow(clippy::missing_const_for_fn)]
  #[must_use]
  pub fn inconsistent_terminology(term: String, details: String) -> Self {
    Self::InconsistentTerminology { term, details }
  }

  /// Create an overlapping preconditions error
  ///
  /// # Arguments
  ///
  /// * `behavior` - The qualified behavior name (feature.behavior)
  /// * `overlaps` - The list of overlapping condition names
  ///
  /// # Returns
  ///
  /// A `SemanticError::OverlappingPreconditions` variant
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::validation::semantic::SemanticError;
  ///
  /// let error = SemanticError::overlapping_preconditions(
  ///     "auth.session".to_string(),
  ///     vec!["authenticate".to_string()]
  /// );
  /// ```
  #[allow(clippy::missing_const_for_fn)]
  #[must_use]
  pub fn overlapping_preconditions(behavior: String, overlaps: Vec<String>) -> Self {
    Self::OverlappingPreconditions { behavior, overlaps }
  }

  /// Create a dependency chain too deep error
  ///
  /// # Arguments
  ///
  /// * `depth` - The actual depth detected
  /// * `max` - The maximum recommended depth
  ///
  /// # Returns
  ///
  /// A `SemanticError::DependencyChainTooDeep` variant
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::validation::semantic::SemanticError;
  ///
  /// let error = SemanticError::dependency_chain_too_deep(7, 5);
  /// ```
  #[must_use]
  pub const fn dependency_chain_too_deep(depth: usize, max: usize) -> Self {
    Self::DependencyChainTooDeep { depth, max }
  }

  /// Create a behavior with preconditions no description error
  ///
  /// # Arguments
  ///
  /// * `behavior` - The qualified behavior name (feature.behavior)
  ///
  /// # Returns
  ///
  /// A `SemanticError::BehaviorWithPreconditionsNoDescription` variant
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::validation::semantic::SemanticError;
  ///
  /// let error = SemanticError::behavior_with_preconditions_no_description(
  ///     "auth.login".to_string()
  /// );
  /// ```
  #[allow(clippy::missing_const_for_fn)]
  #[must_use]
  pub fn behavior_with_preconditions_no_description(behavior: String) -> Self {
    Self::BehaviorWithPreconditionsNoDescription { behavior }
  }
}

/// Result type for semantic validation
pub type SemanticResult<T> = Result<T, Vec<SemanticError>>;

/// Result of terminology consistency checks
///
/// Contains information about terminology inconsistencies found
/// during validation, along with a collection of all unique terms
/// used across the specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminologyCheck {
  /// Inconsistencies found during terminology checking
  pub inconsistencies: Vec<SemanticError>,
  /// All unique terms (feature and behavior names) across the spec
  pub unique_terms: HashSet<String>,
}

/// Result of cross-reference validation
///
/// Contains information about broken references found in behavior
/// preconditions, postconditions, and feature dependencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossReferenceResult {
  /// All broken references found during validation
  pub broken_references: Vec<SemanticError>,
  /// Total number of references checked (both valid and invalid)
  pub total_references: usize,
}

/// Complete semantic validation result
///
/// Aggregates all results from semantic validation, including
/// cross-reference checks, terminology consistency, and constraint
/// validation. Validity is derived from the presence of errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticValidationResult {
  /// All errors found during validation
  pub errors: Vec<SemanticError>,
  /// Cross-reference validation results
  pub cross_references: CrossReferenceResult,
  /// Terminology check results
  pub terminology: TerminologyCheck,
}

impl SemanticValidationResult {
  /// Create a new validation result
  #[must_use]
  pub fn new() -> Self {
    Self {
      errors: Vec::new(),
      cross_references: CrossReferenceResult {
        broken_references: Vec::new(),
        total_references: 0,
      },
      terminology: TerminologyCheck {
        inconsistencies: Vec::new(),
        unique_terms: HashSet::new(),
      },
    }
  }

  /// Check if validation passed (no errors found)
  ///
  /// This method derives validity from the absence of errors,
  /// following Scott Wlaschin's DDD principle of making states
  /// explicit rather than storing redundant flags.
  ///
  /// # Returns
  ///
  /// `true` if no errors were found during validation
  ///
  /// # Examples
  ///
  /// ```
  /// use clarity_web::intent::validation::semantic::SemanticValidationResult;
  ///
  /// let result = SemanticValidationResult::new();
  /// assert!(result.is_valid());
  /// ```
  #[must_use]
  pub const fn is_valid(&self) -> bool {
    self.errors.is_empty()
  }

  /// Add an error to the result
  fn add_error(&mut self, error: SemanticError) {
    self.errors.push(error.clone());

    // Categorize error into appropriate section
    match &error {
      SemanticError::BrokenReference { .. } | SemanticError::BrokenDependency { .. } => {
        self.cross_references.broken_references.push(error);
      }
      SemanticError::InconsistentTerminology { .. } => {
        self.terminology.inconsistencies.push(error);
      }
      _ => {
        // Already added to errors
      }
    }
  }
}

impl Default for SemanticValidationResult {
  fn default() -> Self {
    Self::new()
  }
}

/// Semantic validator for specifications
///
/// Performs cross-reference validation, terminology consistency checks,
/// and semantic constraint validation on specifications.
///
/// # Examples
///
/// ```
/// use clarity_web::intent::validation::semantic::SemanticValidator;
/// use clarity_web::intent::types::{Spec, Feature, Behavior};
///
/// let validator = SemanticValidator::new();
/// let mut spec = Spec::new("my-spec".to_string()).unwrap();
/// // Add a feature to make the spec valid
/// let feature = Feature::new("auth".to_string()).unwrap();
/// spec.add_feature(feature);
/// let result = validator.validate_semantics(&spec).unwrap();
/// if result.is_valid() {
///     println!("Spec is semantically valid!");
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SemanticValidator;

impl SemanticValidator {
  /// Create a new semantic validator
  #[must_use]
  pub const fn new() -> Self {
    Self
  }

  /// Validate all semantics of a specification
  ///
  /// Performs comprehensive semantic validation including:
  /// - Cross-reference validation (behavior references, feature dependencies)
  /// - Consistency checks (terminology, naming conventions)
  /// - Semantic constraint validation (dependency depth, overlapping conditions)
  ///
  /// # Arguments
  ///
  /// * `spec` - The specification to validate
  ///
  /// # Returns
  ///
  /// A `SemanticValidationResult` containing all errors and detailed check results
  ///
  /// # Errors
  ///
  /// Returns a vector of `SemanticError` if the spec has structural issues
  /// (e.g., empty name, no features). Otherwise returns `Ok` with the validation
  /// result, which may still contain errors.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # use clarity_web::intent::validation::semantic::SemanticValidator;
  /// # use clarity_web::intent::types::Spec;
  /// let validator = SemanticValidator::new();
  /// let spec = Spec::new("my-spec".to_string()).unwrap();
  ///
  /// match validator.validate_semantics(&spec) {
  ///     Ok(result) => {
  ///         if result.is_valid() {
  ///             println!("Spec is valid!");
  ///         } else {
  ///             println!("Found {} errors", result.errors.len());
  ///             for error in &result.errors {
  ///                 println!("  - {}", error);
  ///             }
  ///         }
  ///     }
  ///     Err(errors) => {
  ///         println!("Validation failed with structural errors");
  ///         for error in &errors {
  ///             println!("  - {}", error);
  ///         }
  ///     }
  /// }
  /// ```
  pub fn validate_semantics(&self, spec: &Spec) -> SemanticResult<SemanticValidationResult> {
    let mut result = SemanticValidationResult::new();

    // Basic validation
    if spec.name.trim().is_empty() {
      return Err(vec![SemanticError::EmptySpecName]);
    }

    if spec.features.is_empty() {
      return Err(vec![SemanticError::NoFeatures]);
    }

    // Cross-reference validation
    let cross_ref_result = self.cross_reference_validation(spec)?;
    result.cross_references = cross_ref_result.clone();
    result.errors.extend(cross_ref_result.broken_references);

    // Terminology consistency
    let terminology_result = self.consistency_checks(spec)?;
    result.terminology = terminology_result.clone();
    result.errors.extend(terminology_result.inconsistencies);

    // Semantic constraints
    let constraint_errors = self.validate_semantic_constraints(spec)?;
    result.errors.extend(constraint_errors);

    Ok(result)
  }

  /// Validate cross-references between behaviors
  ///
  /// Checks that all behavior references and feature dependencies are valid:
  /// - All precondition references point to existing behaviors
  /// - All postcondition references point to existing behaviors
  /// - All feature dependencies point to existing features
  ///
  /// References can be either simple (e.g., "login") or qualified
  /// (e.g., "auth.login"). Qualified references specify both the feature
  /// and behavior name, while simple references look for the behavior
  /// in any feature.
  ///
  /// # Arguments
  ///
  /// * `spec` - The specification to validate
  ///
  /// # Returns
  ///
  /// A `CrossReferenceResult` containing any broken references found and
  /// the total number of references checked
  ///
  /// # Errors
  ///
  /// This function currently always returns `Ok`, but the return type uses
  /// `Result` for consistency with other validation functions.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # use clarity_web::intent::validation::semantic::SemanticValidator;
  /// # use clarity_web::intent::types::Spec;
  /// let validator = SemanticValidator::new();
  /// let spec = Spec::new("my-spec".to_string()).unwrap();
  ///
  /// let result = validator.cross_reference_validation(&spec).unwrap();
  /// if result.broken_references.is_empty() {
  ///     println!("All {} references are valid!", result.total_references);
  /// } else {
  ///     println!("Found {} broken references:", result.broken_references.len());
  ///     for error in &result.broken_references {
  ///         println!("  - {}", error);
  ///     }
  /// }
  /// ```
  #[must_use]
  pub fn cross_reference_validation(&self, spec: &Spec) -> SemanticResult<CrossReferenceResult> {
    let mut result = CrossReferenceResult {
      broken_references: Vec::new(),
      total_references: 0,
    };

    // Build a map of all valid behaviors: feature_name -> behavior_names
    let valid_behaviors: HashMap<String, HashSet<String>> = spec
      .features
      .iter()
      .map(|feature| {
        let behaviors: HashSet<String> = feature.behaviors.iter().map(|b| b.name.clone()).collect();
        (feature.name.clone(), behaviors)
      })
      .collect();

    // Build set of all valid features
    let valid_features: HashSet<String> = spec.features.iter().map(|f| f.name.clone()).collect();

    // Check feature dependencies
    for feature in &spec.features {
      for dependency in &feature.depends_on {
        result.total_references += 1;
        if !valid_features.contains(dependency) {
          result
            .broken_references
            .push(SemanticError::broken_dependency(
              feature.name.clone(),
              dependency.clone(),
            ));
        }
      }
    }

    // Check behavior preconditions and postconditions
    for feature in &spec.features {
      for behavior in &feature.behaviors {
        // Check preconditions
        for precondition in &behavior.preconditions {
          result.total_references += 1;
          if !Self::is_valid_reference(precondition, &valid_behaviors) {
            result
              .broken_references
              .push(SemanticError::broken_reference(
                format!("{}.{}", feature.name, behavior.name),
                precondition.clone(),
                "precondition".to_string(),
              ));
          }
        }

        // Check postconditions
        for postcondition in &behavior.postconditions {
          result.total_references += 1;
          if !Self::is_valid_reference(postcondition, &valid_behaviors) {
            result
              .broken_references
              .push(SemanticError::broken_reference(
                format!("{}.{}", feature.name, behavior.name),
                postcondition.clone(),
                "postcondition".to_string(),
              ));
          }
        }
      }
    }

    Ok(result)
  }

  /// Check if a reference string is valid
  ///
  /// A reference can be either:
  /// - A simple behavior name (e.g., "login")
  /// - A qualified reference with feature and behavior (e.g., "auth.login")
  ///
  /// For simple references, the behavior is searched across all features.
  /// For qualified references, the feature must match exactly.
  ///
  /// # Arguments
  ///
  /// * `reference` - The reference string to validate
  /// * `valid_behaviors` - Map of feature names to their behavior names
  ///
  /// # Returns
  ///
  /// `true` if the reference points to an existing behavior, `false` otherwise
  fn is_valid_reference(
    reference: &str,
    valid_behaviors: &HashMap<String, HashSet<String>>,
  ) -> bool {
    // Try as qualified reference first (feature.behavior)
    if let Some((feature_part, behavior_part)) = reference.split_once('.') {
      if let Some(behaviors) = valid_behaviors.get(feature_part) {
        return behaviors.contains(behavior_part);
      }
    }

    // Try as simple behavior name (check all features)
    valid_behaviors
      .values()
      .any(|behaviors| behaviors.contains(reference))
  }

  /// Check terminology consistency across the specification
  ///
  /// Analyzes the specification for terminology issues:
  /// - Inconsistent naming conventions (kebab-case vs `snake_case`)
  /// - Similar behavior names that should potentially be unified
  /// - Conflicting terminology across features
  ///
  /// # Arguments
  ///
  /// * `spec` - The specification to check
  ///
  /// # Returns
  ///
  /// A `TerminologyCheck` containing any inconsistencies found and all
  /// unique terms in the specification
  ///
  /// # Errors
  ///
  /// This function currently always returns `Ok`, but the return type uses
  /// `Result` for consistency with other validation functions.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # use clarity_web::intent::validation::semantic::SemanticValidator;
  /// # use clarity_web::intent::types::Spec;
  /// let validator = SemanticValidator::new();
  /// let spec = Spec::new("my-spec".to_string()).unwrap();
  ///
  /// let result = validator.consistency_checks(&spec).unwrap();
  /// if result.inconsistencies.is_empty() {
  ///     println!("No terminology issues found!");
  ///     println!("Found {} unique terms", result.unique_terms.len());
  /// } else {
  ///     println!("Found {} terminology issues:", result.inconsistencies.len());
  ///     for error in &result.inconsistencies {
  ///         println!("  - {}", error);
  ///     }
  /// }
  /// ```
  pub fn consistency_checks(&self, spec: &Spec) -> SemanticResult<TerminologyCheck> {
    let mut result = TerminologyCheck {
      inconsistencies: Vec::new(),
      unique_terms: HashSet::new(),
    };

    // Extract all behavior names
    let all_behaviors: Vec<String> = spec
      .features
      .iter()
      .flat_map(|feature| {
        feature
          .behaviors
          .iter()
          .map(|b| format!("{}.{}", feature.name, b.name))
      })
      .collect();

    // Check for similar behavior names (potential terminology issues)
    for (name_a, name_b) in all_behaviors.iter().tuple_windows() {
      if Self::are_similar_terms(name_a, name_b) {
        result
          .inconsistencies
          .push(SemanticError::inconsistent_terminology(
            name_a.clone(),
            format!("similar to '{name_b}' - consider unifying terminology"),
          ));
      }
    }

    // Extract all feature and behavior names as unique terms
    for feature in &spec.features {
      result.unique_terms.insert(feature.name.clone());
      for behavior in &feature.behaviors {
        result.unique_terms.insert(behavior.name.clone());
      }
    }

    // Check for inconsistent naming conventions
    let all_names: Vec<&str> = spec
      .features
      .iter()
      .flat_map(|f| {
        f.behaviors
          .iter()
          .map(|b| b.name.as_str())
          .chain(std::iter::once(f.name.as_str()))
      })
      .collect();

    let has_kebab_case = all_names.iter().any(|n| n.contains('-'));
    let has_snake_case = all_names.iter().any(|n| n.contains('_'));

    if has_kebab_case && has_snake_case {
      result
        .inconsistencies
        .push(SemanticError::inconsistent_terminology(
          "naming convention".to_string(),
          "mix of kebab-case and snake_case detected".to_string(),
        ));
    }

    Ok(result)
  }

  /// Check if two terms are similar (potential terminology conflict)
  ///
  /// Uses simple substring heuristics to detect similar terms that might
  /// indicate inconsistent naming. For example, "`user_login`" and "login"
  /// would be flagged as similar.
  ///
  /// # Arguments
  ///
  /// * `term_a` - First term to compare (may be qualified, e.g., "auth.login")
  /// * `term_b` - Second term to compare (may be qualified, e.g., "user.login")
  ///
  /// # Returns
  ///
  /// `true` if the terms appear similar enough to warrant review, `false` otherwise
  fn are_similar_terms(term_a: &str, term_b: &str) -> bool {
    // Get just the behavior names (after the last dot)
    let name_a = term_a.rsplit('.').next().unwrap_or(term_a);
    let name_b = term_b.rsplit('.').next().unwrap_or(term_b);

    // Skip if identical
    if name_a == name_b {
      return false;
    }

    // Check if one is a substring of the other (min length 4)
    if name_a.len() >= 4 && name_b.contains(name_a) {
      return true;
    }
    if name_b.len() >= 4 && name_a.contains(name_b) {
      return true;
    }

    false
  }

  /// Validate semantic constraints
  ///
  /// Checks that the specification satisfies semantic constraints:
  /// - Behaviors with preconditions have descriptions explaining dependencies
  /// - Feature dependency chains are reasonable (recommended max depth: 5)
  /// - No behavior has overlapping preconditions and postconditions
  ///
  /// # Arguments
  ///
  /// * `spec` - The specification to validate
  ///
  /// # Returns
  ///
  /// A vector of `SemanticError` with any constraint violations found
  ///
  /// # Errors
  ///
  /// This function currently always returns `Ok`, but the return type uses
  /// `Result` for consistency with other validation functions.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # use clarity_web::intent::validation::semantic::SemanticValidator;
  /// # use clarity_web::intent::types::Spec;
  /// let validator = SemanticValidator::new();
  /// let spec = Spec::new("my-spec".to_string()).unwrap();
  ///
  /// let violations = validator.validate_semantic_constraints(&spec).unwrap();
  /// if violations.is_empty() {
  ///     println!("No constraint violations!");
  /// } else {
  ///     println!("Found {} constraint violations:", violations.len());
  ///     for violation in &violations {
  ///         println!("  - {}", violation);
  ///     }
  /// }
  /// ```
  pub fn validate_semantic_constraints(&self, spec: &Spec) -> SemanticResult<Vec<SemanticError>> {
    let mut errors = Vec::new();

    // Check max dependency depth
    let max_depth = self.calculate_max_dependency_depth(spec);
    if max_depth > 5 {
      errors.push(SemanticError::dependency_chain_too_deep(max_depth, 5));
    }

    // Check for behaviors with preconditions that reference their own postconditions
    for feature in &spec.features {
      for behavior in &feature.behaviors {
        let preconditions: HashSet<&str> = behavior
          .preconditions
          .iter()
          .map(std::string::String::as_str)
          .collect();
        let postconditions: HashSet<&str> = behavior
          .postconditions
          .iter()
          .map(std::string::String::as_str)
          .collect();

        // Check for overlap
        let overlap: Vec<String> = preconditions
          .intersection(&postconditions)
          .map(std::string::ToString::to_string)
          .collect();

        if !overlap.is_empty() {
          errors.push(SemanticError::overlapping_preconditions(
            format!("{}.{}", feature.name, behavior.name),
            overlap,
          ));
        }

        // Check that behaviors with dependencies have descriptions
        if !behavior.preconditions.is_empty() && behavior.description.trim().is_empty() {
          errors.push(SemanticError::behavior_with_preconditions_no_description(
            format!("{}.{}", feature.name, behavior.name),
          ));
        }
      }
    }

    Ok(errors)
  }

  /// Calculate the maximum depth of feature dependency chains
  ///
  /// Traverses the dependency graph to find the longest chain of
  /// feature dependencies. This helps identify specs that may be
  /// overly complex or tightly coupled.
  ///
  /// # Arguments
  ///
  /// * `spec` - The specification to analyze
  ///
  /// # Returns
  ///
  /// The maximum dependency depth found (0 if no dependencies exist)
  fn calculate_max_dependency_depth(&self, spec: &Spec) -> usize {
    // Build dependency map
    let dep_map: HashMap<&str, &Vec<String>> = spec
      .features
      .iter()
      .map(|f| (f.name.as_str(), &f.depends_on))
      .collect();

    let mut max_depth = 0;

    for feature in &spec.features {
      let depth = Self::calculate_depth(feature.name.as_str(), &dep_map, &mut HashSet::new());
      max_depth = max_depth.max(depth);
    }

    max_depth
  }

  /// Recursively calculate dependency depth for a feature
  ///
  /// Performs a depth-first traversal of the dependency graph,
  /// tracking visited nodes to prevent infinite recursion in case
  /// of circular dependencies.
  ///
  /// # Arguments
  ///
  /// * `feature_name` - The feature to calculate depth for
  /// * `dep_map` - Map of feature names to their dependencies
  /// * `visiting` - Set of features currently being visited (for cycle detection)
  ///
  /// # Returns
  ///
  /// The maximum depth from this feature to a leaf (0 if no dependencies)
  fn calculate_depth<'a>(
    feature_name: &'a str,
    dep_map: &HashMap<&'a str, &'a Vec<String>>,
    visiting: &mut HashSet<&'a str>,
  ) -> usize {
    // Prevent infinite recursion
    if visiting.contains(feature_name) {
      return 0;
    }
    visiting.insert(feature_name);

    let dependencies = match dep_map.get(feature_name) {
      Some(deps) => deps,
      None => return 0,
    };

    if dependencies.is_empty() {
      return 0;
    }

    let max_child_depth = dependencies
      .iter()
      .map(|dep| Self::calculate_depth(dep.as_str(), dep_map, visiting))
      .max()
      .unwrap_or(0);

    visiting.remove(feature_name);
    max_child_depth + 1
  }
}

impl Default for SemanticValidator {
  fn default() -> Self {
    Self
  }
}

/// Convenience function to validate semantics
///
/// Creates a default `SemanticValidator` and validates the specification.
/// Equivalent to `SemanticValidator::new().validate_semantics(spec)`.
///
/// # Arguments
///
/// * `spec` - The specification to validate
///
/// # Returns
///
/// A `SemanticValidationResult` containing all errors and detailed check results
///
/// # Errors
///
/// Returns a vector of `SemanticError` if the spec has structural issues
///
/// # Examples
///
/// ```no_run
/// # use clarity_web::intent::validation::semantic::validate_semantics;
/// # use clarity_web::intent::types::Spec;
/// let spec = Spec::new("my-spec".to_string()).unwrap();
///
/// match validate_semantics(&spec) {
///     Ok(result) => println!("Validation: {}", if result.is_valid() { "passed" } else { "failed" }),
///     Err(errors) => println!("Validation errors: {}", errors.len()),
/// }
/// ```
pub fn validate_semantics(spec: &Spec) -> SemanticResult<SemanticValidationResult> {
  SemanticValidator.validate_semantics(spec)
}

/// Convenience function to validate cross-references
///
/// Creates a default `SemanticValidator` and validates cross-references.
/// Equivalent to `SemanticValidator::new().cross_reference_validation(spec)`.
///
/// # Arguments
///
/// * `spec` - The specification to validate
///
/// # Returns
///
/// A `CrossReferenceResult` containing any broken references found
///
/// # Errors
///
/// This function currently always returns `Ok`
///
/// # Examples
///
/// ```no_run
/// # use clarity_web::intent::validation::semantic::cross_reference_validation;
/// # use clarity_web::intent::types::Spec;
/// let spec = Spec::new("my-spec".to_string()).unwrap();
///
/// let result = cross_reference_validation(&spec).unwrap();
/// println!("Checked {} references, found {} issues",
///     result.total_references,
///     result.broken_references.len()
/// );
/// ```
pub fn cross_reference_validation(spec: &Spec) -> SemanticResult<CrossReferenceResult> {
  SemanticValidator.cross_reference_validation(spec)
}

/// Convenience function to check terminology consistency
///
/// Creates a default `SemanticValidator` and checks terminology.
/// Equivalent to `SemanticValidator::new().consistency_checks(spec)`.
///
/// # Arguments
///
/// * `spec` - The specification to check
///
/// # Returns
///
/// A `TerminologyCheck` containing any inconsistencies found
///
/// # Errors
///
/// This function currently always returns `Ok`
///
/// # Examples
///
/// ```no_run
/// # use clarity_web::intent::validation::semantic::consistency_checks;
/// # use clarity_web::intent::types::Spec;
/// let spec = Spec::new("my-spec".to_string()).unwrap();
///
/// let result = consistency_checks(&spec).unwrap();
/// if result.inconsistencies.is_empty() {
///     println!("Terminology is consistent!");
/// }
/// ```
pub fn consistency_checks(spec: &Spec) -> SemanticResult<TerminologyCheck> {
  SemanticValidator.consistency_checks(spec)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::intent::types::{Behavior, Feature};

  fn create_test_spec() -> Spec {
    match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        let mut auth_feature = match Feature::new("auth".to_string()) {
          Ok(f) => f,
          Err(_) => return spec,
        };

        let login = match Behavior::new("login".to_string()) {
          Ok(b) => b.with_description("User logs in".to_string()),
          Err(_) => return spec,
        };

        let logout = match Behavior::new("logout".to_string()) {
          Ok(b) => b.with_description("User logs out".to_string()),
          Err(_) => return spec,
        };

        let _ = auth_feature.add_behavior(login);
        let _ = auth_feature.add_behavior(logout);

        let mut user_feature = match Feature::new("users".to_string()) {
          Ok(f) => f,
          Err(_) => return spec,
        };

        user_feature.add_dependency("auth".to_string());

        let create_user = match Behavior::new("create".to_string()) {
          Ok(mut b) => {
            b.description = "Create a new user".to_string();
            b.preconditions.push("auth.login".to_string());
            b
          }
          Err(_) => return spec,
        };

        let _ = user_feature.add_behavior(create_user);

        let _ = spec.add_feature(auth_feature);
        let _ = spec.add_feature(user_feature);
        spec
      }
      Err(_) => {
        panic!("Failed to create test spec");
      }
    }
  }

  #[test]
  fn test_validate_semantics_valid_spec() {
    let spec = create_test_spec();
    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    assert!(result.is_ok());
    let validation_result = match result {
      Ok(r) => r,
      Err(_) => return,
    };
    assert!(validation_result.is_valid());
    assert!(validation_result.errors.is_empty());
  }

  #[test]
  fn test_empty_spec_name_returns_validation_error() {
    let mut spec = match Spec::new("   ".to_string()) {
      Ok(s) => s,
      Err(_) => return,
    };

    let mut feature = match Feature::new("test".to_string()) {
      Ok(f) => f,
      Err(_) => return,
    };

    let behavior = match Behavior::new("test_behavior".to_string()) {
      Ok(b) => b,
      Err(_) => return,
    };

    let _ = feature.add_behavior(behavior);
    let _ = spec.add_feature(feature);

    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    assert!(result.is_err());
    match result {
      Err(errors) => {
        assert!(errors.iter().any(|e| e == &SemanticError::EmptySpecName));
      }
      Ok(_) => {}
    }
  }

  #[test]
  fn test_no_features_returns_error() {
    let spec = match Spec::new("empty-spec".to_string()) {
      Ok(s) => s,
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    assert!(result.is_err());
    match result {
      Err(errors) => {
        assert!(errors.iter().any(|e| e == &SemanticError::NoFeatures));
      }
      Ok(_) => {}
    }
  }

  #[test]
  fn test_broken_reference_returns_error() {
    let mut spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let mut login = match Behavior::new("login".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };

        // Add a non-existent precondition
        login.preconditions.push("nonexistent_behavior".to_string());

        let _ = feature.add_behavior(login);
        let _ = s.add_feature(feature);
        s
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.cross_reference_validation(&spec);

    assert!(result.is_ok());
    let cross_ref_result = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    assert!(!cross_ref_result.broken_references.is_empty());
    assert!(cross_ref_result.broken_references.iter().any(|e| match e {
      SemanticError::BrokenReference { target, .. } => target == "nonexistent_behavior",
      _ => false,
    }));
  }

  #[test]
  fn test_broken_feature_dependency_returns_error() {
    let mut spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        // Add dependency on non-existent feature
        feature.add_dependency("nonexistent_feature".to_string());

        let behavior = match Behavior::new("login".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = s.add_feature(feature);
        s
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.cross_reference_validation(&spec);

    assert!(result.is_ok());
    let cross_ref_result = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    assert!(!cross_ref_result.broken_references.is_empty());
    assert!(cross_ref_result.broken_references.iter().any(|e| match e {
      SemanticError::BrokenDependency { dependency, .. } => {
        dependency == "nonexistent_feature"
      }
      _ => false,
    }));
  }

  #[test]
  fn test_valid_qualified_reference() {
    let spec = create_test_spec();
    let validator = SemanticValidator::new();
    let result = validator.cross_reference_validation(&spec);

    assert!(result.is_ok());
    let cross_ref_result = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    assert!(cross_ref_result.broken_references.is_empty());
  }

  #[test]
  fn test_inconsistent_naming_conventions() {
    let mut spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        // Feature with kebab-case
        let mut feature1 = match Feature::new("auth-service".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let behavior1 = match Behavior::new("login".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };

        let _ = feature1.add_behavior(behavior1);
        let _ = s.add_feature(feature1);

        // Feature with snake_case
        let mut feature2 = match Feature::new("user_service".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let behavior2 = match Behavior::new("create".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };

        let _ = feature2.add_behavior(behavior2);
        let _ = s.add_feature(feature2);
        s
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.consistency_checks(&spec);

    assert!(result.is_ok());
    let terminology = match result {
      Ok(t) => t,
      Err(_) => return,
    };

    assert!(!terminology.inconsistencies.is_empty());
    assert!(terminology.inconsistencies.iter().any(|e| match e {
      SemanticError::InconsistentTerminology { term, .. } => {
        term == "naming convention"
      }
      _ => false,
    }));
  }

  #[test]
  fn test_overlapping_pre_post_conditions() {
    let mut spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let mut session = match Behavior::new("session".to_string()) {
          Ok(mut b) => {
            // Add same behavior to both pre and post conditions
            b.preconditions.push("authenticate".to_string());
            b.postconditions.push("authenticate".to_string());
            b
          }
          Err(_) => return,
        };

        let authenticate = match Behavior::new("authenticate".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };

        let _ = feature.add_behavior(session);
        let _ = feature.add_behavior(authenticate);
        let _ = s.add_feature(feature);
        s
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    assert!(result.is_ok());
    let validation_result = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should have constraint violation for overlapping
    assert!(validation_result.errors.iter().any(|e| match e {
      SemanticError::OverlappingPreconditions { .. } => true,
      _ => false,
    }));
  }

  #[test]
  fn test_validation_is_deterministic() {
    let spec = create_test_spec();
    let validator = SemanticValidator::new();

    let result1 = validator.validate_semantics(&spec);
    let result2 = validator.validate_semantics(&spec);

    match (result1, result2) {
      (Ok(r1), Ok(r2)) => {
        assert_eq!(r1, r2);
      }
      _ => {}
    }
  }

  #[test]
  fn test_no_unwrap_in_semantic_validator() {
    // This test is verified by the lints at the top of the file
    // #![deny(clippy::unwrap_used)]
    // #![deny(clippy::expect_used)]
    // #![deny(clippy::panic)]
    let spec = create_test_spec();
    let _ = SemanticValidator::new().validate_semantics(&spec);
    // If we got here without panicking, the test passes
  }

  #[test]
  fn test_deep_dependency_chain_violation() {
    // Create a chain of dependencies > 5 deep
    let mut spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        let mut prev_feature_name: Option<String> = None;

        for i in 0..7 {
          let mut feature = match Feature::new(format!("feature_{i}")) {
            Ok(f) => f,
            Err(_) => return,
          };

          let behavior = match Behavior::new(format!("behavior_{i}")) {
            Ok(b) => b,
            Err(_) => return,
          };

          let _ = feature.add_behavior(behavior);

          if let Some(prev_name) = prev_feature_name {
            feature.add_dependency(prev_name);
          }

          prev_feature_name = Some(feature.name.clone());
          let _ = s.add_feature(feature);
        }
        s
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    assert!(result.is_ok());
    let validation_result = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    assert!(validation_result
      .errors
      .iter()
      .any(|e| matches!(e, SemanticError::DependencyChainTooDeep { .. })));
  }

  #[test]
  fn test_behavior_with_preconditions_no_description() {
    let mut spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let mut authenticate = match Behavior::new("authenticate".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };

        let mut login = match Behavior::new("login".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };

        // Login has precondition but no description
        login.preconditions.push("authenticate".to_string());
        // Intentionally leave description empty

        let _ = feature.add_behavior(authenticate);
        let _ = feature.add_behavior(login);
        let _ = s.add_feature(feature);
        s
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    assert!(result.is_ok());
    let validation_result = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    assert!(validation_result.errors.iter().any(|e| matches!(
      e,
      SemanticError::BehaviorWithPreconditionsNoDescription { .. }
    )));
  }
}
