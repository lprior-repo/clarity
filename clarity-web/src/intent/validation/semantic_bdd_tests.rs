//! BDD Tests for Semantic Validator
//!
//! These tests follow the Given-When-Then pattern from Dan North's BDD framework.
//! Each test is structured as:
//!   GIVEN: A specific state/setup
//!   WHEN:  An action or condition is applied
//!   THEN:  An expected outcome is verified

#![allow(clippy::unwrap_used)] // Allowed in test code

use crate::intent::types::{Behavior, Feature, Spec};
use crate::intent::validation::semantic::{SemanticError, SemanticValidator};

/// Helper to create a valid spec with one feature and behavior (`snake_case` names)
fn create_valid_spec() -> Spec {
  let mut spec = Spec::new("test_api".to_string()).unwrap();
  let mut feature = Feature::new("test_feature".to_string()).unwrap();
  let behavior = Behavior::new("test_behavior".to_string()).unwrap();
  let _ = feature.add_behavior(behavior);
  let _ = spec.add_feature(feature);
  spec
}

mod spec_name_validation {
  use super::*;

  #[test]
  fn given_valid_spec_name_when_validated_then_passes() {
    // GIVEN: A spec with a valid name
    let spec = create_valid_spec();

    // WHEN: The spec is validated
    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    // THEN: It passes validation
    assert!(result.is_ok());
    let validation_result = result.unwrap();
    assert!(validation_result.is_valid());
  }
}

mod feature_validation {
  use super::*;

  #[test]
  fn given_spec_with_no_features_when_validated_then_returns_no_features_error() {
    // GIVEN: A spec with no features
    let spec = Spec::new("test_spec".to_string()).unwrap();

    // WHEN: The spec is validated
    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    // THEN: It returns a NoFeatures error
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
      .iter()
      .any(|e| matches!(e, SemanticError::NoFeatures)));
  }

  #[test]
  fn given_valid_feature_when_validated_then_passes() {
    // GIVEN: A spec with valid features
    let spec = create_valid_spec();

    // WHEN: The spec is validated
    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    // THEN: It passes validation
    assert!(result.is_ok());
  }
}

mod dependency_validation {
  use super::*;

  #[test]
  fn given_valid_dependency_chain_when_validated_then_passes() {
    // GIVEN: A valid dependency chain: auth -> users
    let mut spec = Spec::new("test_spec".to_string()).unwrap();

    let mut auth_feature = Feature::new("auth".to_string()).unwrap();
    let login = Behavior::new("login".to_string()).unwrap();
    let _ = auth_feature.add_behavior(login);

    let mut users_feature = Feature::new("users".to_string()).unwrap();
    users_feature.add_dependency("auth".to_string());
    let get_users = Behavior::new("get_users".to_string()).unwrap();
    let _ = users_feature.add_behavior(get_users);

    let _ = spec.add_feature(auth_feature);
    let _ = spec.add_feature(users_feature);

    // WHEN: The spec is validated
    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    // THEN: It passes validation
    assert!(result.is_ok());
  }

  #[test]
  fn given_multiple_features_with_dependencies_when_validated_then_passes() {
    // GIVEN: A spec with multiple features and dependencies
    let mut spec = Spec::new("api_spec".to_string()).unwrap();

    let mut auth = Feature::new("auth".to_string()).unwrap();
    let _ = auth.add_behavior(Behavior::new("login".to_string()).unwrap());
    let _ = auth.add_behavior(Behavior::new("logout".to_string()).unwrap());

    let mut users = Feature::new("users".to_string()).unwrap();
    users.add_dependency("auth".to_string());
    let _ = users.add_behavior(Behavior::new("create_user".to_string()).unwrap());
    let _ = users.add_behavior(Behavior::new("list_users".to_string()).unwrap());

    let mut orders = Feature::new("orders".to_string()).unwrap();
    orders.add_dependency("auth".to_string());
    orders.add_dependency("users".to_string());
    let _ = orders.add_behavior(Behavior::new("create_order".to_string()).unwrap());

    let _ = spec.add_feature(auth);
    let _ = spec.add_feature(users);
    let _ = spec.add_feature(orders);

    // WHEN: The spec is validated
    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    // THEN: It passes validation
    assert!(result.is_ok());
  }
}

mod cross_reference_validation {
  use super::*;

  #[test]
  fn given_valid_behavior_references_when_validated_then_passes() {
    // GIVEN: A spec with valid behavior references in preconditions
    let mut spec = Spec::new("test_spec".to_string()).unwrap();

    let mut feature = Feature::new("test".to_string()).unwrap();
    let mut behavior = Behavior::new("second_behavior".to_string()).unwrap();
    behavior
      .preconditions
      .push("test.first_behavior".to_string());
    let _ = feature.add_behavior(behavior);
    let _ = feature.add_behavior(Behavior::new("first_behavior".to_string()).unwrap());

    let _ = spec.add_feature(feature);

    // WHEN: Cross-reference validation is performed
    let validator = SemanticValidator::new();
    let result = validator.cross_reference_validation(&spec);

    // THEN: It passes validation
    assert!(result.is_ok());
  }
}

mod terminology_validation {
  use super::*;

  #[test]
  fn given_consistent_naming_when_validated_then_passes() {
    // GIVEN: A spec with consistent naming
    let mut spec = Spec::new("api_spec".to_string()).unwrap();

    let mut feature1 = Feature::new("user_management".to_string()).unwrap();
    let behavior1 = Behavior::new("get_user".to_string()).unwrap();
    let _ = feature1.add_behavior(behavior1);

    let mut feature2 = Feature::new("order_management".to_string()).unwrap();
    let behavior2 = Behavior::new("get_order".to_string()).unwrap();
    let _ = feature2.add_behavior(behavior2);

    let _ = spec.add_feature(feature1);
    let _ = spec.add_feature(feature2);

    // WHEN: Terminology consistency check is performed
    let validator = SemanticValidator::new();
    let result = validator.consistency_checks(&spec);

    // THEN: It passes validation (no terminology warnings)
    assert!(result.is_ok());
  }
}
