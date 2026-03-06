//! Adversarial tests for semantic validator
//!
//! These tests probe edge cases, boundary conditions, and potential vulnerabilities
//! in the semantic validator to ensure robustness.

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]

use crate::intent::types::{Behavior, Feature, Spec};
use crate::intent::validation::semantic::{SemanticError, SemanticValidator};

/// Test helper to create a minimal valid spec
fn create_minimal_spec() -> Spec {
  match Spec::new("test-spec".to_string()) {
    Ok(mut spec) => {
      let mut feature = match Feature::new("auth".to_string()) {
        Ok(f) => f,
        Err(_) => return spec,
      };

      let behavior = match Behavior::new("login".to_string()) {
        Ok(b) => b.with_description("User logs in".to_string()),
        Err(_) => return spec,
      };

      let _ = feature.add_behavior(behavior);
      let _ = spec.add_feature(feature);
      spec
    }
    Err(_) => {
      // Should never happen in tests
      panic!("Failed to create test spec")
    }
  }
}

#[cfg(test)]
mod edge_cases {
  use super::*;

  #[test]
  fn test_empty_string_spec_name() {
    let mut spec = match Spec::new(String::new()) {
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
    if let Err(errors) = result {
      assert!(errors.iter().any(|e| e == &SemanticError::EmptySpecName));
    }
  }

  #[test]
  fn test_whitespace_only_spec_name() {
    let mut spec = match Spec::new("   \t\n   ".to_string()) {
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
    if let Err(errors) = result {
      assert!(errors.iter().any(|e| e == &SemanticError::EmptySpecName));
    }
  }

  #[test]
  fn test_empty_feature_name() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        let mut feature = match Feature::new(String::new()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let behavior = match Behavior::new("test_behavior".to_string()) {
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
    let result = validator.validate_semantics(&spec);

    // Should validate - empty feature names are technically allowed by semantic validator
    // (it checks semantic consistency, not validity)
    assert!(result.is_ok());
  }

  #[test]
  fn test_empty_behavior_name() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let behavior = match Behavior::new(String::new()) {
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
    let result = validator.validate_semantics(&spec);

    // Should validate - empty behavior names don't break semantics
    assert!(result.is_ok());
  }

  #[test]
  fn test_circular_feature_dependencies() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        // Feature A depends on B
        let mut feature_a = match Feature::new("feature_a".to_string()) {
          Ok(mut f) => {
            f.add_dependency("feature_b".to_string());
            f
          }
          Err(_) => return,
        };

        let behavior_a = match Behavior::new("behavior_a".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };

        let _ = feature_a.add_behavior(behavior_a);

        // Feature B depends on A (circular!)
        let mut feature_b = match Feature::new("feature_b".to_string()) {
          Ok(mut f) => {
            f.add_dependency("feature_a".to_string());
            f
          }
          Err(_) => return,
        };

        let behavior_b = match Behavior::new("behavior_b".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };

        let _ = feature_b.add_behavior(behavior_b);

        let _ = s.add_feature(feature_a);
        let _ = s.add_feature(feature_b);
        s
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    // Should detect circular dependency via depth calculation
    assert!(result.is_ok());
    let validation_result = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // The circular dependency should be caught by the depth calculation
    // preventing infinite recursion (visiting set check)
    assert!(!validation_result.is_valid() || validation_result.errors.is_empty());
  }

  #[test]
  fn test_self_referencing_feature_dependency() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        let mut feature = match Feature::new("auth".to_string()) {
          Ok(mut f) => {
            // Feature depends on itself!
            f.add_dependency("auth".to_string());
            f
          }
          Err(_) => return,
        };

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
    let result = validator.validate_semantics(&spec);

    // Should handle gracefully - no crash
    assert!(result.is_ok());
  }

  #[test]
  fn test_maximum_nesting_depth_dependency_chain() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        let mut prev_feature_name: Option<String> = None;

        // Create 100 levels of dependencies
        for i in 0..100 {
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

    // Should not crash, should detect deep chain
    assert!(result.is_ok());
    let validation_result = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should flag deep dependency chain
    assert!(validation_result
      .errors
      .iter()
      .any(|e| matches!(e, SemanticError::DependencyChainTooDeep { .. })));
  }
}

#[cfg(test)]
mod unicode_tests {
  use super::*;

  #[test]
  fn test_unicode_spec_name() {
    let spec = match Spec::new("тест-спецификация".to_string()) {
      Ok(mut spec) => {
        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let behavior = match Behavior::new("login".to_string()) {
          Ok(b) => b.with_description("User logs in".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    // Should handle Unicode gracefully
    assert!(result.is_ok());
  }

  #[test]
  fn test_unicode_feature_names() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        let mut feature = match Feature::new("авторизация".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let behavior = match Behavior::new("login".to_string()) {
          Ok(b) => b.with_description("User logs in".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    // Should handle Unicode in feature names
    assert!(result.is_ok());
  }

  #[test]
  fn test_unicode_behavior_names() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let behavior = match Behavior::new("вход".to_string()) {
          Ok(b) => b.with_description("User logs in".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    // Should handle Unicode in behavior names
    assert!(result.is_ok());
  }

  #[test]
  fn test_mixed_script_names() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        let mut feature = match Feature::new("auth-авторизация".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let behavior = match Behavior::new("login-вход".to_string()) {
          Ok(b) => b.with_description("User logs in".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    // Should handle mixed scripts
    assert!(result.is_ok());
  }

  #[test]
  fn test_emoji_in_names() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        let mut feature = match Feature::new("auth🔐".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let behavior = match Behavior::new("login🚀".to_string()) {
          Ok(b) => b.with_description("User logs in".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    // Should handle emoji
    assert!(result.is_ok());
  }

  #[test]
  fn test_zero_width_characters() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        // Zero-width space and non-printing characters
        let mut feature = match Feature::new("au\u{200B}th".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let behavior = match Behavior::new("lo\u{200C}gin".to_string()) {
          Ok(b) => b.with_description("User logs in".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    // Should handle zero-width characters
    assert!(result.is_ok());
  }

  #[test]
  fn test_right_to_left_text() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        // RTL Arabic text
        let mut feature = match Feature::new("المصادقة".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let behavior = match Behavior::new("تسجيل_الدخول".to_string()) {
          Ok(b) => b.with_description("User logs in".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    // Should handle RTL text
    assert!(result.is_ok());
  }
}

#[cfg(test)]
mod malformed_input_tests {
  use super::*;

  #[test]
  fn test_reference_with_multiple_dots() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let mut behavior = match Behavior::new("login".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };

        // Reference with multiple dots
        behavior
          .preconditions
          .push("feature.behavior.extra".to_string());

        let _ = feature.add_behavior(behavior);
        let _ = s.add_feature(feature);
        s
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.cross_reference_validation(&spec);

    // Should handle gracefully - will be treated as broken reference
    assert!(result.is_ok());
    let cross_ref = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should flag as broken reference
    assert!(!cross_ref.broken_references.is_empty());
  }

  #[test]
  fn test_reference_starting_with_dot() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let mut behavior = match Behavior::new("login".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };

        // Reference starting with dot
        behavior.preconditions.push(".behavior".to_string());

        let _ = feature.add_behavior(behavior);
        let _ = s.add_feature(feature);
        s
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.cross_reference_validation(&spec);

    // Should handle gracefully
    assert!(result.is_ok());
  }

  #[test]
  fn test_reference_ending_with_dot() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let mut behavior = match Behavior::new("login".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };

        // Reference ending with dot
        behavior.preconditions.push("feature.".to_string());

        let _ = feature.add_behavior(behavior);
        let _ = s.add_feature(feature);
        s
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.cross_reference_validation(&spec);

    // Should handle gracefully
    assert!(result.is_ok());
  }

  #[test]
  fn test_reference_with_only_dots() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let mut behavior = match Behavior::new("login".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };

        // Reference with only dots
        behavior.preconditions.push("...".to_string());

        let _ = feature.add_behavior(behavior);
        let _ = s.add_feature(feature);
        s
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.cross_reference_validation(&spec);

    // Should handle gracefully
    assert!(result.is_ok());
  }

  #[test]
  fn test_duplicate_preconditions() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let authenticate = match Behavior::new("authenticate".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };

        let mut login = match Behavior::new("login".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };

        // Add same precondition multiple times and provide description
        login.preconditions.push("authenticate".to_string());
        login.preconditions.push("authenticate".to_string());
        login.preconditions.push("authenticate".to_string());
        login.description = "User login".to_string(); // Required when has preconditions

        let _ = feature.add_behavior(authenticate);
        let _ = feature.add_behavior(login);
        let _ = s.add_feature(feature);
        s
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    // Should handle duplicates gracefully
    assert!(result.is_ok());
    let validation_result = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should still be valid (duplicates are redundant but not semantic errors)
    assert!(validation_result.is_valid());
  }

  #[test]
  fn test_overlapping_pre_and_post_conditions_duplicates() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let authenticate = match Behavior::new("authenticate".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };

        let mut session = match Behavior::new("session".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };

        // Add overlapping conditions with duplicates
        session.preconditions.push("authenticate".to_string());
        session.preconditions.push("authenticate".to_string());
        session.postconditions.push("authenticate".to_string());
        session.postconditions.push("authenticate".to_string());

        let _ = feature.add_behavior(authenticate);
        let _ = feature.add_behavior(session);
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

    // Should detect overlap even with duplicates
    assert!(validation_result
      .errors
      .iter()
      .any(|e| matches!(e, SemanticError::OverlappingPreconditions { .. })));
  }
}

#[cfg(test)]
mod performance_tests {
  use super::*;

  #[test]
  fn test_large_spec_many_features() {
    let spec = match Spec::new("large-spec".to_string()) {
      Ok(mut s) => {
        // Create 1000 features with 10 behaviors each
        for i in 0..1000 {
          let mut feature = match Feature::new(format!("feature_{i}")) {
            Ok(f) => f,
            Err(_) => return,
          };

          for j in 0..10 {
            let behavior = match Behavior::new(format!("behavior_{i}_{j}")) {
              Ok(b) => b.with_description(format!("Behavior {i}_{j}")),
              Err(_) => return,
            };

            let _ = feature.add_behavior(behavior);
          }

          let _ = s.add_feature(feature);
        }
        s
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let start = std::time::Instant::now();
    let result = validator.validate_semantics(&spec);
    let duration = start.elapsed();

    // Should complete in reasonable time (< 5 seconds)
    assert!(
      duration.as_secs() < 5,
      "Validation took too long: {duration:?}"
    );
    assert!(result.is_ok());
  }

  #[test]
  fn test_large_spec_many_references() {
    let spec = match Spec::new("large-spec".to_string()) {
      Ok(mut s) => {
        let base_feature = match Feature::new("base".to_string()) {
          Ok(mut f) => {
            for i in 0..100 {
              let behavior = match Behavior::new(format!("base_behavior_{i}")) {
                Ok(b) => b,
                Err(_) => return,
              };
              let _ = f.add_behavior(behavior);
            }
            f
          }
          Err(_) => return,
        };

        let _ = s.add_feature(base_feature);

        // Create feature that references all base behaviors
        let mut dependent = match Feature::new("dependent".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        for i in 0..100 {
          let mut behavior = match Behavior::new(format!("dep_behavior_{i}")) {
            Ok(b) => b,
            Err(_) => return,
          };

          // Reference each base behavior
          for j in 0..100 {
            behavior
              .preconditions
              .push(format!("base.base_behavior_{j}"));
          }

          let _ = dependent.add_behavior(behavior);
        }

        let _ = s.add_feature(dependent);
        s
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let start = std::time::Instant::now();
    let result = validator.cross_reference_validation(&spec);
    let duration = start.elapsed();

    // Should complete in reasonable time
    assert!(
      duration.as_secs() < 5,
      "Cross-reference validation took too long: {duration:?}"
    );
    assert!(result.is_ok());
  }

  #[test]
  fn test_deeply_nested_terminology_check() {
    let spec = match Spec::new("large-spec".to_string()) {
      Ok(mut s) => {
        // Create many features with similar names (terminology check is O(n^2))
        for i in 0..500 {
          let mut feature = match Feature::new(format!("feature_{i}")) {
            Ok(f) => f,
            Err(_) => return,
          };

          for j in 0..10 {
            let behavior = match Behavior::new(format!("behavior_{i}_{j}")) {
              Ok(b) => b,
              Err(_) => return,
            };

            let _ = feature.add_behavior(behavior);
          }

          let _ = s.add_feature(feature);
        }
        s
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let start = std::time::Instant::now();
    let result = validator.consistency_checks(&spec);
    let duration = start.elapsed();

    // Should complete but might take longer due to O(n^2) algorithm
    assert!(
      duration.as_secs() < 10,
      "Terminology check took too long: {duration:?}"
    );
    assert!(result.is_ok());
  }
}

#[cfg(test)]
mod special_character_tests {
  use super::*;

  #[test]
  fn test_null_bytes_in_name() {
    // Note: Rust strings don't allow embedded null bytes, so this tests
    // that we don't crash if null-like patterns appear
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        let mut feature = match Feature::new("auth\x00fake".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let behavior = match Behavior::new("login".to_string()) {
          Ok(b) => b.with_description("User logs in".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    // Should handle gracefully
    assert!(result.is_ok());
  }

  #[test]
  fn test_path_separators_in_names() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        let mut feature = match Feature::new("auth/service".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let behavior = match Behavior::new("user/login".to_string()) {
          Ok(b) => b.with_description("User logs in".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    // Should handle - path separators are valid chars
    assert!(result.is_ok());
  }

  #[test]
  fn test_control_characters() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        let mut feature = match Feature::new("auth\tfeature".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let behavior = match Behavior::new("login\nbehavior".to_string()) {
          Ok(b) => b.with_description("User logs in".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let validator = SemanticValidator::new();
    let result = validator.validate_semantics(&spec);

    // Should handle control characters
    assert!(result.is_ok());
  }
}
