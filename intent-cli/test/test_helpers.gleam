//// Shared test helpers for Intent CLI tests
//// This module provides common factory functions for creating test data

import gleam/dict
import gleam/json.{type Json}
import intent/types.{
  type AntiPattern, type Behavior, type Feature, type Invariant, type Spec,
  type Verification, AIHints, AntiPattern, Behavior, Feature,
  ImplementationHints, Invariant, SecurityHints, Spec, Verification,
}

// ============================================================================
// Behavior Factories
// ============================================================================

/// Create a minimal behavior for testing
/// Takes name and list of required behaviors (dependencies)
pub fn make_test_behavior(name: String, requires: List(String)) -> Behavior {
  Behavior(
    name: name,
    intent: "Test intent for " <> name,
    notes: "",
    requires: requires,
    tags: [],
    preconditions: [],
    postconditions: [],
    verifications: [],
  )
}

/// Create a behavior with custom preconditions, postconditions, and verifications
pub fn make_test_behavior_with_conditions(
  name: String,
  requires: List(String),
  preconditions: List(String),
  postconditions: List(String),
  verifications: List(Verification),
) -> Behavior {
  Behavior(
    name: name,
    intent: "Test intent for " <> name,
    notes: "",
    requires: requires,
    tags: [],
    preconditions: preconditions,
    postconditions: postconditions,
    verifications: verifications,
  )
}

// ============================================================================
// Feature Factories
// ============================================================================

/// Create a feature from a list of behaviors
pub fn make_test_feature(name: String, behaviors: List(Behavior)) -> Feature {
  Feature(
    name: name,
    description: "Test feature: " <> name,
    behaviors: behaviors,
  )
}

// ============================================================================
// Spec Factories
// ============================================================================

/// Create a minimal spec from a list of features
pub fn make_test_spec(features: List(Feature)) -> Spec {
  Spec(
    name: "Test Spec",
    description: "Test spec for tests",
    audience: "developers",
    version: "1.0.0",
    success_criteria: [],
    features: features,
    invariants: [],
    anti_patterns: [],
    ai_hints: AIHints(
      implementation: ImplementationHints([], "", []),
      entities: dict.new(),
      security: SecurityHints(
        password_hashing: "",
        jwt_algorithm: "",
        jwt_expiry: "",
        rate_limiting: "",
      ),
      pitfalls: [],
    ),
  )
}

/// Create a spec with a custom name
pub fn make_test_spec_with_name(name: String, features: List(Feature)) -> Spec {
  Spec(..make_test_spec(features), name: name)
}

/// Create a spec from a list of behaviors (wraps in default feature)
pub fn make_test_spec_from_behaviors(behaviors: List(Behavior)) -> Spec {
  make_test_spec([make_test_feature("Default", behaviors)])
}

/// Create a spec with custom invariants
pub fn make_test_spec_with_invariants(
  features: List(Feature),
  invariants: List(Invariant),
) -> Spec {
  Spec(
    name: "Test Spec",
    description: "Test spec for tests",
    audience: "developers",
    version: "1.0.0",
    success_criteria: [],
    features: features,
    invariants: invariants,
    anti_patterns: [],
    ai_hints: AIHints(
      implementation: ImplementationHints([], "", []),
      entities: dict.new(),
      security: SecurityHints(
        password_hashing: "",
        jwt_algorithm: "",
        jwt_expiry: "",
        rate_limiting: "",
      ),
      pitfalls: [],
    ),
  )
}

/// Create a minimal invariant for testing
pub fn make_test_invariant(name: String) -> Invariant {
  Invariant(name: name, description: "Test invariant: " <> name, criteria: [])
}

/// Create an invariant with criteria
pub fn make_test_invariant_with_criteria(
  name: String,
  description: String,
  criteria: List(String),
) -> Invariant {
  Invariant(name: name, description: description, criteria: criteria)
}

// ============================================================================
// Anti-Pattern Factory
// ============================================================================

/// Create a minimal anti-pattern for testing
pub fn make_test_anti_pattern(name: String) -> AntiPattern {
  AntiPattern(
    name: name,
    description: "Test anti-pattern: " <> name,
    bad_example: json.null(),
    good_example: json.null(),
    why: "This is an anti-pattern",
  )
}
