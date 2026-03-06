//! Domain Newtypes - Type-safe wrappers for domain concepts
//!
//! This module provides newtype wrappers for domain concepts like behavior references,
//! feature names, and spec names. These wrappers enforce validation at construction time,
//! making invalid states unrepresentable at the type level.
//!
//! ## Design Philosophy
//!
//! Following Scott Wlaschin's "Domain Modeling Made Functional" and the "Parse, Don't Validate"
//! principle, these types ensure that any value of the type has already passed validation.
//!
//! ## Example
//!
//! ```rust
//! use clarity_web::intent::types::names::BehaviorReference;
//!
//! // Parsing validates the format
//! let reference = BehaviorReference::parse("auth.login".to_string())?;
//! assert_eq!(reference.feature(), "auth");
//! assert_eq!(reference.behavior(), "login");
//!
//! // Invalid formats return errors
//! let invalid = BehaviorReference::parse("no_dot_here".to_string());
//! assert!(invalid.is_err());
//! # Ok::<(), clarity_web::intent::types::names::NameError>(())
//! ```

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Errors that can occur when parsing domain names
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NameError {
  /// Behavior name does not match required `snake_case` pattern
  #[error("behavior name '{0}' must be snake_case (lowercase letters, numbers, underscores, starting with letter)")]
  InvalidBehaviorName(String),

  /// Feature name is empty or invalid
  #[error("feature name cannot be empty")]
  EmptyFeatureName,

  /// Spec name is empty or invalid
  #[error("spec name cannot be empty")]
  EmptySpecName,

  /// Behavior reference is missing the qualifier (feature.behavior format)
  #[error("behavior reference '{0}' is missing qualifier (expected 'feature.behavior' format)")]
  MissingQualifier(String),

  /// Behavior reference has an invalid format
  #[error("behavior reference '{0}' has invalid format")]
  InvalidReferenceFormat(String),

  /// Dependency reference is empty
  #[error("dependency reference cannot be empty")]
  EmptyDependency,
}

// ============================================================================
// BehaviorName - Validated behavior name (snake_case)
// ============================================================================

/// A validated behavior name in `snake_case` format.
///
/// Behavior names must:
/// - Start with a lowercase letter
/// - Contain only lowercase letters, numbers, and underscores
///
/// # Examples
///
/// ```rust
/// use clarity_web::intent::types::names::BehaviorName;
///
/// let valid = BehaviorName::parse("create_user".to_string())?;
/// assert_eq!(valid.as_str(), "create_user");
///
/// let invalid = BehaviorName::parse("CreateUser".to_string());
/// assert!(invalid.is_err());
/// # Ok::<(), clarity_web::intent::types::names::NameError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct BehaviorName(String);

impl BehaviorName {
  /// Create a new validated behavior name.
  ///
  /// # Errors
  ///
  /// Returns `NameError::InvalidBehaviorName` if the name doesn't match
  /// the required `snake_case` pattern.
  pub fn parse(s: String) -> Result<Self, NameError> {
    if !is_valid_behavior_name(&s) {
      return Err(NameError::InvalidBehaviorName(s));
    }
    Ok(Self(s))
  }

  /// Get the behavior name as a string slice.
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }

  /// Create a `BehaviorName` without validation.
  ///
  /// # Safety
  ///
  /// The caller must ensure the name is valid `snake_case`.
  #[must_use]
  pub const fn unchecked_new(name: String) -> Self {
    Self(name)
  }

  /// Convert into the inner String.
  #[must_use]
  pub fn into_inner(self) -> String {
    self.0
  }
}

impl fmt::Display for BehaviorName {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl AsRef<str> for BehaviorName {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl From<BehaviorName> for String {
  fn from(value: BehaviorName) -> Self {
    value.0
  }
}

impl TryFrom<String> for BehaviorName {
  type Error = NameError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::parse(value)
  }
}

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

// ============================================================================
// FeatureName - Validated feature name
// ============================================================================

/// A validated feature name.
///
/// Feature names must be non-empty after trimming whitespace.
///
/// # Examples
///
/// ```rust
/// use clarity_web::intent::types::names::FeatureName;
///
/// let valid = FeatureName::parse("user-auth".to_string())?;
/// assert_eq!(valid.as_str(), "user-auth");
///
/// let invalid = FeatureName::parse("".to_string());
/// assert!(invalid.is_err());
/// # Ok::<(), clarity_web::intent::types::names::NameError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct FeatureName(String);

impl FeatureName {
  /// Create a new validated feature name.
  ///
  /// # Errors
  ///
  /// Returns `NameError::EmptyFeatureName` if the name is empty or whitespace-only.
  pub fn parse(s: String) -> Result<Self, NameError> {
    if s.trim().is_empty() {
      return Err(NameError::EmptyFeatureName);
    }
    Ok(Self(s))
  }

  /// Get the feature name as a string slice.
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }

  /// Create a `FeatureName` without validation.
  ///
  /// # Safety
  ///
  /// The caller must ensure the name is non-empty.
  #[must_use]
  pub const fn unchecked_new(name: String) -> Self {
    Self(name)
  }

  /// Convert into the inner String.
  #[must_use]
  pub fn into_inner(self) -> String {
    self.0
  }
}

impl fmt::Display for FeatureName {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl AsRef<str> for FeatureName {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl From<FeatureName> for String {
  fn from(value: FeatureName) -> Self {
    value.0
  }
}

impl TryFrom<String> for FeatureName {
  type Error = NameError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::parse(value)
  }
}

// ============================================================================
// SpecName - Validated specification name
// ============================================================================

/// A validated specification name.
///
/// Spec names must be non-empty after trimming whitespace.
///
/// # Examples
///
/// ```rust
/// use clarity_web::intent::types::names::SpecName;
///
/// let valid = SpecName::parse("my-api-spec".to_string())?;
/// assert_eq!(valid.as_str(), "my-api-spec");
///
/// let invalid = SpecName::parse("   ".to_string());
/// assert!(invalid.is_err());
/// # Ok::<(), clarity_web::intent::types::names::NameError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SpecName(String);

impl SpecName {
  /// Create a new validated spec name.
  ///
  /// # Errors
  ///
  /// Returns `NameError::EmptySpecName` if the name is empty or whitespace-only.
  pub fn parse(s: String) -> Result<Self, NameError> {
    if s.trim().is_empty() {
      return Err(NameError::EmptySpecName);
    }
    Ok(Self(s))
  }

  /// Get the spec name as a string slice.
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }

  /// Create a `SpecName` without validation.
  ///
  /// # Safety
  ///
  /// The caller must ensure the name is non-empty.
  #[must_use]
  pub const fn unchecked_new(name: String) -> Self {
    Self(name)
  }

  /// Convert into the inner String.
  #[must_use]
  pub fn into_inner(self) -> String {
    self.0
  }
}

impl fmt::Display for SpecName {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl AsRef<str> for SpecName {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl From<SpecName> for String {
  fn from(value: SpecName) -> Self {
    value.0
  }
}

impl TryFrom<String> for SpecName {
  type Error = NameError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::parse(value)
  }
}

// ============================================================================
// BehaviorReference - Qualified reference (feature.behavior)
// ============================================================================

/// A validated behavior reference in `feature.behavior` format.
///
/// Behavior references are used in preconditions and postconditions to
/// reference behaviors in other features or the same feature.
///
/// # Format
///
/// - Must contain exactly one dot separator
/// - Feature part: non-empty string
/// - Behavior part: non-empty string
///
/// # Examples
///
/// ```rust
/// use clarity_web::intent::types::names::BehaviorReference;
///
/// let reference = BehaviorReference::parse("auth.login".to_string())?;
/// assert_eq!(reference.feature(), "auth");
/// assert_eq!(reference.behavior(), "login");
///
/// // Invalid: no dot
/// let invalid = BehaviorReference::parse("auth_login".to_string());
/// assert!(invalid.is_err());
///
/// // Invalid: multiple dots
/// let invalid = BehaviorReference::parse("a.b.c".to_string());
/// assert!(invalid.is_err());
/// # Ok::<(), clarity_web::intent::types::names::NameError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct BehaviorReference {
  feature: String,
  behavior: String,
  /// Cached full reference string for efficient serialization
  full: String,
}

impl BehaviorReference {
  /// Create a new validated behavior reference.
  ///
  /// # Errors
  ///
  /// Returns `NameError::MissingQualifier` if no dot is present.
  /// Returns `NameError::InvalidReferenceFormat` if format is otherwise invalid.
  pub fn parse(s: String) -> Result<Self, NameError> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 2 {
      return Err(NameError::MissingQualifier(s));
    }
    let feature = parts[0];
    let behavior = parts[1];
    if feature.is_empty() || behavior.is_empty() {
      return Err(NameError::InvalidReferenceFormat(s));
    }
    Ok(Self {
      feature: feature.to_string(),
      behavior: behavior.to_string(),
      full: s,
    })
  }

  /// Get the feature part of the reference.
  #[must_use]
  pub fn feature(&self) -> &str {
    &self.feature
  }

  /// Get the behavior part of the reference.
  #[must_use]
  pub fn behavior(&self) -> &str {
    &self.behavior
  }

  /// Get the full reference as a string slice.
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.full
  }

  /// Create a `BehaviorReference` from validated parts.
  ///
  /// # Safety
  ///
  /// The caller must ensure both parts are non-empty.
  #[must_use]
  pub fn from_parts(feature: String, behavior: String) -> Self {
    let full = format!("{feature}.{behavior}");
    Self {
      feature,
      behavior,
      full,
    }
  }

  /// Convert into the inner String.
  #[must_use]
  pub fn into_inner(self) -> String {
    self.full
  }
}

impl fmt::Display for BehaviorReference {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.full)
  }
}

impl AsRef<str> for BehaviorReference {
  fn as_ref(&self) -> &str {
    &self.full
  }
}

impl From<BehaviorReference> for String {
  fn from(value: BehaviorReference) -> Self {
    value.full
  }
}

impl TryFrom<String> for BehaviorReference {
  type Error = NameError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::parse(value)
  }
}

// ============================================================================
// FeatureDependency - Validated feature dependency reference
// ============================================================================

/// A validated feature dependency reference.
///
/// Used in the `depends_on` field of features to reference other features.
/// Must be a non-empty string.
///
/// # Examples
///
/// ```rust
/// use clarity_web::intent::types::names::FeatureDependency;
///
/// let dep = FeatureDependency::parse("auth".to_string())?;
/// assert_eq!(dep.as_str(), "auth");
///
/// let invalid = FeatureDependency::parse("".to_string());
/// assert!(invalid.is_err());
/// # Ok::<(), clarity_web::intent::types::names::NameError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct FeatureDependency(String);

impl FeatureDependency {
  /// Create a new validated feature dependency.
  ///
  /// # Errors
  ///
  /// Returns `NameError::EmptyDependency` if the reference is empty.
  pub fn parse(s: String) -> Result<Self, NameError> {
    if s.trim().is_empty() {
      return Err(NameError::EmptyDependency);
    }
    Ok(Self(s))
  }

  /// Get the dependency as a string slice.
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }

  /// Create a `FeatureDependency` without validation.
  ///
  /// # Safety
  ///
  /// The caller must ensure the reference is non-empty.
  #[must_use]
  pub const fn unchecked_new(reference: String) -> Self {
    Self(reference)
  }

  /// Convert into the inner String.
  #[must_use]
  pub fn into_inner(self) -> String {
    self.0
  }
}

impl fmt::Display for FeatureDependency {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl AsRef<str> for FeatureDependency {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl From<FeatureDependency> for String {
  fn from(value: FeatureDependency) -> Self {
    value.0
  }
}

impl TryFrom<String> for FeatureDependency {
  type Error = NameError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    Self::parse(value)
  }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {

  use super::*;

  // -------------------------------------------------------------------------
  // BehaviorName tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_behavior_name_valid() {
    let name = BehaviorName::parse("create_user".to_string()).ok();
    assert!(name.is_some());
    let name = name.unwrap();
    assert_eq!(name.as_str(), "create_user");
  }

  #[test]
  fn test_behavior_name_simple() {
    let name = BehaviorName::parse("save".to_string());
    assert!(name.is_ok());
  }

  #[test]
  fn test_behavior_name_with_numbers() {
    let name = BehaviorName::parse("parse_v2".to_string());
    assert!(name.is_ok());
  }

  #[test]
  fn test_behavior_name_invalid_uppercase() {
    let result = BehaviorName::parse("CreateUser".to_string());
    assert!(matches!(result, Err(NameError::InvalidBehaviorName(_))));
  }

  #[test]
  fn test_behavior_name_invalid_starts_with_number() {
    let result = BehaviorName::parse("1_create".to_string());
    assert!(matches!(result, Err(NameError::InvalidBehaviorName(_))));
  }

  #[test]
  fn test_behavior_name_invalid_hyphen() {
    let result = BehaviorName::parse("create-user".to_string());
    assert!(matches!(result, Err(NameError::InvalidBehaviorName(_))));
  }

  #[test]
  fn test_behavior_name_empty() {
    let result = BehaviorName::parse(String::new());
    assert!(matches!(result, Err(NameError::InvalidBehaviorName(_))));
  }

  #[test]
  fn test_behavior_name_serde_roundtrip() {
    let name = BehaviorName::parse("login".to_string()).ok();
    assert!(name.is_some());
    let name = name.unwrap();

    let json = serde_json::to_string(&name).ok();
    assert!(json.is_some());
    let json = json.unwrap();

    let parsed: Result<BehaviorName, _> = serde_json::from_str(&json);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();

    assert_eq!(name, parsed);
  }

  // -------------------------------------------------------------------------
  // FeatureName tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_feature_name_valid() {
    let name = FeatureName::parse("user-auth".to_string()).ok();
    assert!(name.is_some());
    let name = name.unwrap();
    assert_eq!(name.as_str(), "user-auth");
  }

  #[test]
  fn test_feature_name_empty() {
    let result = FeatureName::parse(String::new());
    assert!(matches!(result, Err(NameError::EmptyFeatureName)));
  }

  #[test]
  fn test_feature_name_whitespace() {
    let result = FeatureName::parse("   ".to_string());
    assert!(matches!(result, Err(NameError::EmptyFeatureName)));
  }

  #[test]
  fn test_feature_name_serde_roundtrip() {
    let name = FeatureName::parse("auth".to_string()).ok();
    assert!(name.is_some());
    let name = name.unwrap();

    let json = serde_json::to_string(&name).ok();
    assert!(json.is_some());
    let json = json.unwrap();

    let parsed: Result<FeatureName, _> = serde_json::from_str(&json);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();

    assert_eq!(name, parsed);
  }

  // -------------------------------------------------------------------------
  // SpecName tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_spec_name_valid() {
    let name = SpecName::parse("my-spec".to_string()).ok();
    assert!(name.is_some());
    let name = name.unwrap();
    assert_eq!(name.as_str(), "my-spec");
  }

  #[test]
  fn test_spec_name_empty() {
    let result = SpecName::parse(String::new());
    assert!(matches!(result, Err(NameError::EmptySpecName)));
  }

  #[test]
  fn test_spec_name_whitespace() {
    let result = SpecName::parse("   ".to_string());
    assert!(matches!(result, Err(NameError::EmptySpecName)));
  }

  #[test]
  fn test_spec_name_serde_roundtrip() {
    let name = SpecName::parse("my-api-spec".to_string()).ok();
    assert!(name.is_some());
    let name = name.unwrap();

    let json = serde_json::to_string(&name).ok();
    assert!(json.is_some());
    let json = json.unwrap();

    let parsed: Result<SpecName, _> = serde_json::from_str(&json);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();

    assert_eq!(name, parsed);
  }

  // -------------------------------------------------------------------------
  // BehaviorReference tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_behavior_reference_valid() {
    let reference = BehaviorReference::parse("auth.login".to_string()).ok();
    assert!(reference.is_some());
    let reference = reference.unwrap();
    assert_eq!(reference.feature(), "auth");
    assert_eq!(reference.behavior(), "login");
    assert_eq!(reference.as_str(), "auth.login");
  }

  #[test]
  fn test_behavior_reference_missing_dot() {
    let result = BehaviorReference::parse("authlogin".to_string());
    assert!(matches!(result, Err(NameError::MissingQualifier(_))));
  }

  #[test]
  fn test_behavior_reference_multiple_dots() {
    let result = BehaviorReference::parse("a.b.c".to_string());
    assert!(matches!(result, Err(NameError::MissingQualifier(_))));
  }

  #[test]
  fn test_behavior_reference_empty_feature() {
    let result = BehaviorReference::parse(".login".to_string());
    assert!(matches!(result, Err(NameError::InvalidReferenceFormat(_))));
  }

  #[test]
  fn test_behavior_reference_empty_behavior() {
    let result = BehaviorReference::parse("auth.".to_string());
    assert!(matches!(result, Err(NameError::InvalidReferenceFormat(_))));
  }

  #[test]
  fn test_behavior_reference_from_parts() {
    let reference = BehaviorReference::from_parts("users".to_string(), "create".to_string());
    assert_eq!(reference.feature(), "users");
    assert_eq!(reference.behavior(), "create");
    assert_eq!(reference.as_str(), "users.create");
  }

  #[test]
  fn test_behavior_reference_serde_roundtrip() {
    let reference = BehaviorReference::parse("auth.login".to_string()).ok();
    assert!(reference.is_some());
    let reference = reference.unwrap();

    let json = serde_json::to_string(&reference).ok();
    assert!(json.is_some());
    let json = json.unwrap();

    let parsed: Result<BehaviorReference, _> = serde_json::from_str(&json);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();

    assert_eq!(reference, parsed);
  }

  // -------------------------------------------------------------------------
  // FeatureDependency tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_feature_dependency_valid() {
    let dep = FeatureDependency::parse("auth".to_string()).ok();
    assert!(dep.is_some());
    let dep = dep.unwrap();
    assert_eq!(dep.as_str(), "auth");
  }

  #[test]
  fn test_feature_dependency_empty() {
    let result = FeatureDependency::parse(String::new());
    assert!(matches!(result, Err(NameError::EmptyDependency)));
  }

  #[test]
  fn test_feature_dependency_whitespace() {
    let result = FeatureDependency::parse("   ".to_string());
    assert!(matches!(result, Err(NameError::EmptyDependency)));
  }

  #[test]
  fn test_feature_dependency_serde_roundtrip() {
    let dep = FeatureDependency::parse("users".to_string()).ok();
    assert!(dep.is_some());
    let dep = dep.unwrap();

    let json = serde_json::to_string(&dep).ok();
    assert!(json.is_some());
    let json = json.unwrap();

    let parsed: Result<FeatureDependency, _> = serde_json::from_str(&json);
    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();

    assert_eq!(dep, parsed);
  }

  // -------------------------------------------------------------------------
  // Display trait tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_display_behavior_name() {
    let name = BehaviorName::parse("login".to_string()).ok();
    assert!(name.is_some());
    let name = name.unwrap();
    assert_eq!(format!("{name}"), "login");
  }

  #[test]
  fn test_display_feature_name() {
    let name = FeatureName::parse("auth".to_string()).ok();
    assert!(name.is_some());
    let name = name.unwrap();
    assert_eq!(format!("{name}"), "auth");
  }

  #[test]
  fn test_display_spec_name() {
    let name = SpecName::parse("my-spec".to_string()).ok();
    assert!(name.is_some());
    let name = name.unwrap();
    assert_eq!(format!("{name}"), "my-spec");
  }

  #[test]
  fn test_display_behavior_reference() {
    let reference = BehaviorReference::parse("auth.login".to_string()).ok();
    assert!(reference.is_some());
    let reference = reference.unwrap();
    assert_eq!(format!("{reference}"), "auth.login");
  }

  #[test]
  fn test_display_feature_dependency() {
    let dep = FeatureDependency::parse("auth".to_string()).ok();
    assert!(dep.is_some());
    let dep = dep.unwrap();
    assert_eq!(format!("{dep}"), "auth");
  }

  // -------------------------------------------------------------------------
  // Error message tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_name_error_messages() {
    let err = NameError::InvalidBehaviorName("BadName".to_string());
    assert!(err.to_string().contains("BadName"));

    let err = NameError::EmptyFeatureName;
    assert!(err.to_string().contains("feature name"));

    let err = NameError::EmptySpecName;
    assert!(err.to_string().contains("spec name"));

    let err = NameError::MissingQualifier("no_dot".to_string());
    assert!(err.to_string().contains("no_dot"));
    assert!(err.to_string().contains("qualifier"));

    let err = NameError::InvalidReferenceFormat(".".to_string());
    assert!(err.to_string().contains("invalid format"));

    let err = NameError::EmptyDependency;
    assert!(err.to_string().contains("dependency"));
  }
}
