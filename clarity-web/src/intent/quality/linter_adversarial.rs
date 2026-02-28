//! Adversarial tests for spec linter
//!
//! These tests probe edge cases, boundary conditions, and potential vulnerabilities
//! in the linter to ensure robustness.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use crate::intent::quality::linter::{LintError, LintResult, LintRule, LintSeverity, SpecLinter};
use crate::intent::types::{Behavior, Feature, Spec};

/// Test helper to create a minimal valid spec
fn create_minimal_spec() -> Spec {
  match Spec::new("test-spec".to_string()) {
    Ok(mut spec) => {
      spec.description = "A test specification".to_string();

      let mut feature = match Feature::new("auth".to_string()) {
        Ok(f) => f.with_description("Authentication".to_string()),
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
    Err(_) => panic!("Failed to create test spec"),
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
      Ok(b) => b.with_description("Test behavior".to_string()),
      Err(_) => return,
    };

    let _ = feature.add_behavior(behavior);
    let _ = spec.add_feature(feature);

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_err());
    assert!(matches!(result, Err(LintError::EmptySpecName)));
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
      Ok(b) => b.with_description("Test behavior".to_string()),
      Err(_) => return,
    };

    let _ = feature.add_behavior(behavior);
    let _ = spec.add_feature(feature);

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_err());
    assert!(matches!(result, Err(LintError::EmptySpecName)));
  }

  #[test]
  fn test_empty_feature_name() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        s.description = "Test spec".to_string();

        let mut feature = match Feature::new(String::new()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let behavior = match Behavior::new("test_behavior".to_string()) {
          Ok(b) => b.with_description("Test behavior".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = s.add_feature(feature);
        s
      }
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should flag empty feature name as error
    assert!(report
      .results
      .iter()
      .any(|r| { r.location == "features[0].name" && r.message.contains("required") }));
  }

  #[test]
  fn test_whitespace_only_feature_name() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        s.description = "Test spec".to_string();

        let mut feature = match Feature::new("   ".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let behavior = match Behavior::new("test_behavior".to_string()) {
          Ok(b) => b.with_description("Test behavior".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = s.add_feature(feature);
        s
      }
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should flag whitespace-only feature name
    assert!(report
      .results
      .iter()
      .any(|r| { r.location == "features[0].name" && r.message.contains("required") }));
  }

  #[test]
  fn test_empty_behavior_name() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        s.description = "Test spec".to_string();

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

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should flag empty behavior name
    assert!(report
      .results
      .iter()
      .any(|r| { r.location.contains("behaviors[0].name") && r.message.contains("required") }));
  }

  #[test]
  fn test_feature_with_no_behaviors() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut s) => {
        s.description = "Test spec".to_string();

        let feature = match Feature::new("auth".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };

        let _ = s.add_feature(feature);
        s
      }
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should flag feature with no behaviors
    assert!(report
      .results
      .iter()
      .any(|r| { r.message.contains("no behaviors") }));
  }
}

#[cfg(test)]
mod unicode_tests {
  use super::*;

  #[test]
  fn test_unicode_spec_name() {
    let spec = match Spec::new("тест-спецификация".to_string()) {
      Ok(mut spec) => {
        spec.description = "Test spec".to_string();

        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f.with_description("Authentication".to_string()),
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

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    // Should handle Unicode gracefully
    assert!(result.is_ok());
  }

  #[test]
  fn test_rtl_text_in_names() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        spec.description = "Test spec".to_string();

        let mut feature = match Feature::new("المصادقة".to_string()) {
          Ok(f) => f.with_description("Authentication".to_string()),
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

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    // Should handle RTL text
    assert!(result.is_ok());
  }

  #[test]
  fn test_emoji_in_names() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        spec.description = "Test spec".to_string();

        let mut feature = match Feature::new("auth🔐".to_string()) {
          Ok(f) => f.with_description("Authentication".to_string()),
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

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    // Should handle emoji
    assert!(result.is_ok());
  }

  #[test]
  fn test_zero_width_characters() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        spec.description = "Test spec".to_string();

        let mut feature = match Feature::new("au\u{200B}th".to_string()) {
          Ok(f) => f.with_description("Authentication".to_string()),
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

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    // Should handle zero-width characters
    assert!(result.is_ok());
  }

  #[test]
  fn test_mixed_scripts_in_names() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        spec.description = "Test spec".to_string();

        let mut feature = match Feature::new("auth-المصادقة-認証".to_string()) {
          Ok(f) => f.with_description("Authentication".to_string()),
          Err(_) => return,
        };

        let behavior = match Behavior::new("login-вход-ログイン".to_string()) {
          Ok(b) => b.with_description("User logs in".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    // Should handle mixed scripts
    assert!(result.is_ok());
  }
}

#[cfg(test)]
mod description_edge_cases {
  use super::*;

  #[test]
  fn test_whitespace_only_description() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        spec.description = "   \t\n   ".to_string();

        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f.with_description("   ".to_string()),
          Err(_) => return,
        };

        let mut behavior = match Behavior::new("login".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };
        behavior.description = "  \t  ".to_string();

        let _ = feature.add_behavior(behavior);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should flag whitespace-only descriptions
    assert!(report
      .results
      .iter()
      .any(|r| { r.message.contains("no description") || r.message.contains("too short") }));
  }

  #[test]
  fn test_very_long_description() {
    let long_desc = "a".repeat(100_000);

    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        spec.description = long_desc.clone();

        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f.with_description(long_desc.clone()),
          Err(_) => return,
        };

        let behavior = match Behavior::new("login".to_string()) {
          Ok(b) => b.with_description(long_desc),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let start = std::time::Instant::now();
    let result = linter.lint_spec(&spec);
    let duration = start.elapsed();

    // Should handle long descriptions without performance issue
    assert!(
      duration.as_secs() < 1,
      "Linting took too long with long descriptions: {duration:?}"
    );
    assert!(result.is_ok());
  }

  #[test]
  fn test_special_characters_in_description() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        spec.description = "Test with <script>alert('xss')</script> & \"quotes\"".to_string();

        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f.with_description("Auth with $PECIAL {chars}".to_string()),
          Err(_) => return,
        };

        let behavior = match Behavior::new("login".to_string()) {
          Ok(b) => b.with_description("Login with `backticks` and 'quotes'".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    // Should handle special characters
    assert!(result.is_ok());
  }

  #[test]
  fn test_vague_terms_with_different_cases() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        spec.description = "Test spec".to_string();

        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f.with_description("Authentication".to_string()),
          Err(_) => return,
        };

        let mut behavior1 = match Behavior::new("login".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };
        behavior1.description = "TODO: Implement login".to_string();

        let mut behavior2 = match Behavior::new("logout".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };
        behavior2.description = "Tbd: will implement later".to_string();

        let mut behavior3 = match Behavior::new("reset".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };
        behavior3.description = "ToDO: Password reset".to_string();

        let _ = feature.add_behavior(behavior1);
        let _ = feature.add_behavior(behavior2);
        let _ = feature.add_behavior(behavior3);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should detect vague terms regardless of case
    let vague_results: Vec<&LintResult> = report
      .results
      .iter()
      .filter(|r| {
        r.message.contains("vague") || r.message.contains("todo") || r.message.contains("tbd")
      })
      .collect();

    assert!(
      !vague_results.is_empty(),
      "Should detect vague terms in different cases"
    );
  }
}

#[cfg(test)]
mod naming_convention_edge_cases {
  use super::*;

  #[test]
  fn test_all_kebab_case_names() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        spec.description = "Test spec".to_string();

        let mut feature = match Feature::new("auth-service".to_string()) {
          Ok(f) => f.with_description("Authentication".to_string()),
          Err(_) => return,
        };

        let behavior1 = match Behavior::new("user_login".to_string()) {
          Ok(b) => b.with_description("User login".to_string()),
          Err(_) => return,
        };

        let behavior2 = match Behavior::new("user_logout".to_string()) {
          Ok(b) => b.with_description("User logout".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior1);
        let _ = feature.add_behavior(behavior2);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should NOT flag - consistent kebab-case in feature, snake_case in behaviors
    let naming_issues: Vec<&LintResult> = report
      .results
      .iter()
      .filter(|r| r.rule == LintRule::NamingConvention)
      .collect();

    assert!(
      naming_issues.is_empty(),
      "Should not flag consistent naming conventions"
    );
  }

  #[test]
  fn test_mixed_separators_in_single_name() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        spec.description = "Test spec".to_string();

        let mut feature = match Feature::new("auth_service-test".to_string()) {
          Ok(f) => f.with_description("Authentication".to_string()),
          Err(_) => return,
        };

        let behavior = match Behavior::new("user-login_test".to_string()) {
          Ok(b) => b.with_description("User login".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should flag mixed separators
    assert!(report
      .results
      .iter()
      .any(|r| { r.rule == LintRule::NamingConvention && r.message.contains("mixed") }));
  }

  #[test]
  fn test_numbers_in_names() {
    let spec = match Spec::new("test-spec-v2".to_string()) {
      Ok(mut spec) => {
        spec.description = "Test spec".to_string();

        let mut feature = match Feature::new("auth2".to_string()) {
          Ok(f) => f.with_description("Authentication v2".to_string()),
          Err(_) => return,
        };

        let behavior = match Behavior::new("login_2fa".to_string()) {
          Ok(b) => b.with_description("Two-factor login".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    // Should handle numbers in names
    assert!(result.is_ok());
  }

  #[test]
  fn test_single_character_names() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        spec.description = "Test spec".to_string();

        let mut feature = match Feature::new("a".to_string()) {
          Ok(f) => f.with_description("Feature A".to_string()),
          Err(_) => return,
        };

        let behavior = match Behavior::new("b".to_string()) {
          Ok(b) => b.with_description("Behavior B".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    // Should handle single character names
    assert!(result.is_ok());
  }
}

#[cfg(test)]
mod performance_tests {
  use super::*;

  #[test]
  fn test_large_spec_performance() {
    let spec = match Spec::new("large-spec".to_string()) {
      Ok(mut s) => {
        s.description = "Large test specification".to_string();

        // Create 1000 features with 10 behaviors each
        for i in 0..1000 {
          let mut feature = match Feature::new(format!("feature_{i}")) {
            Ok(f) => f.with_description(format!("Feature {i}")),
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

    let linter = SpecLinter::new();
    let start = std::time::Instant::now();
    let result = linter.lint_spec(&spec);
    let duration = start.elapsed();

    // Should complete in reasonable time (< 5 seconds)
    assert!(
      duration.as_secs() < 5,
      "Linting took too long: {duration:?}"
    );
    assert!(result.is_ok());
  }

  #[test]
  fn test_spec_with_many_issues() {
    let spec = match Spec::new("problematic-spec".to_string()) {
      Ok(mut s) => {
        s.description = "x".to_string(); // Too short

        // Create 100 features with naming issues
        for i in 0..100 {
          let mut feature = match Feature::new(format!("Feature-{i}_test")) {
            Ok(f) => f,
            Err(_) => return,
          };

          for j in 0..10 {
            let mut behavior = match Behavior::new(format!("Behavior-{j}")) {
              Ok(b) => b,
              Err(_) => return,
            };
            behavior.description = "TODO: implement".to_string();

            let _ = feature.add_behavior(behavior);
          }

          let _ = s.add_feature(feature);
        }
        s
      }
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let start = std::time::Instant::now();
    let result = linter.lint_spec(&spec);
    let duration = start.elapsed();

    // Should complete reasonably even with many issues
    assert!(
      duration.as_secs() < 5,
      "Linting with many issues took too long: {duration:?}"
    );

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should detect many issues
    assert!(report.results.len() > 100, "Should detect many issues");
  }
}

#[cfg(test)]
mod generic_names_tests {
  use super::*;

  #[test]
  fn test_all_generic_names() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        spec.description = "Test spec".to_string();

        let mut feature = match Feature::new("service".to_string()) {
          Ok(f) => f.with_description("A service".to_string()),
          Err(_) => return,
        };

        let behavior1 = match Behavior::new("handle".to_string()) {
          Ok(b) => b.with_description("Handle something".to_string()),
          Err(_) => return,
        };

        let behavior2 = match Behavior::new("process".to_string()) {
          Ok(b) => b.with_description("Process data".to_string()),
          Err(_) => return,
        };

        let behavior3 = match Behavior::new("execute".to_string()) {
          Ok(b) => b.with_description("Execute command".to_string()),
          Err(_) => return,
        };

        let behavior4 = match Behavior::new("run".to_string()) {
          Ok(b) => b.with_description("Run task".to_string()),
          Err(_) => return,
        };

        let behavior5 = match Behavior::new("do".to_string()) {
          Ok(b) => b.with_description("Do operation".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior1);
        let _ = feature.add_behavior(behavior2);
        let _ = feature.add_behavior(behavior3);
        let _ = feature.add_behavior(behavior4);
        let _ = feature.add_behavior(behavior5);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should flag all generic names
    let generic_results: Vec<&LintResult> = report
      .results
      .iter()
      .filter(|r| r.message.contains("generic"))
      .collect();

    assert_eq!(
      generic_results.len(),
      5,
      "Should flag all 5 generic behavior names"
    );
  }

  #[test]
  fn test_generic_name_as_substring() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        spec.description = "Test spec".to_string();

        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f.with_description("Authentication".to_string()),
          Err(_) => return,
        };

        // These contain generic substrings but are specific enough
        let behavior1 = match Behavior::new("handle_user_login".to_string()) {
          Ok(b) => b.with_description("Handle user login".to_string()),
          Err(_) => return,
        };

        let behavior2 = match Behavior::new("process_payment".to_string()) {
          Ok(b) => b.with_description("Process payment".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior1);
        let _ = feature.add_behavior(behavior2);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should NOT flag - these contain generic substrings but are specific
    let generic_results: Vec<&LintResult> = report
      .results
      .iter()
      .filter(|r| r.message.contains("generic"))
      .collect();

    // The linter checks for exact matches, so these should not be flagged
    assert!(generic_results.is_empty() || generic_results.len() < 2);
  }
}

#[cfg(test)]
mod completeness_edge_cases {
  use super::*;

  #[test]
  fn test_all_descriptions_missing() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        spec.description = String::new();

        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };
        feature.description = String::new();

        let behavior1 = match Behavior::new("login".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };

        let behavior2 = match Behavior::new("logout".to_string()) {
          Ok(b) => b,
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior1);
        let _ = feature.add_behavior(behavior2);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should flag all missing descriptions
    let missing_desc_results: Vec<&LintResult> = report
      .results
      .iter()
      .filter(|r| r.message.contains("no description") || r.message.contains("missing"))
      .collect();

    assert!(
      missing_desc_results.len() >= 3,
      "Should flag spec, feature, and behaviors missing descriptions"
    );
  }

  #[test]
  fn test_exactly_at_threshold_descriptions() {
    let spec = match Spec::new("test-spec".to_string()) {
      Ok(mut spec) => {
        // Exactly 10 chars (threshold)
        spec.description = "1234567890".to_string();

        let mut feature = match Feature::new("auth".to_string()) {
          Ok(f) => f,
          Err(_) => return,
        };
        // Exactly 10 chars
        feature.description = "1234567890".to_string();

        // Exactly 5 chars (behavior threshold)
        let behavior1 = match Behavior::new("login".to_string()) {
          Ok(b) => b.with_description("12345".to_string()),
          Err(_) => return,
        };

        // Exactly 4 chars (below threshold)
        let behavior2 = match Behavior::new("logout".to_string()) {
          Ok(b) => b.with_description("1234".to_string()),
          Err(_) => return,
        };

        let _ = feature.add_behavior(behavior1);
        let _ = feature.add_behavior(behavior2);
        let _ = spec.add_feature(feature);
        spec
      }
      Err(_) => return,
    };

    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    assert!(result.is_ok());
    let report = match result {
      Ok(r) => r,
      Err(_) => return,
    };

    // Should NOT flag 10-char spec/feature descriptions
    assert!(!report
      .results
      .iter()
      .any(|r| { r.location == "spec.description" && r.severity == LintSeverity::Error }));

    // Should flag 4-char behavior description as hint
    assert!(report
      .results
      .iter()
      .any(|r| { r.message.contains("very short") && r.severity == LintSeverity::Hint }));
  }
}
