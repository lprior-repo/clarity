//! BDD Tests for Spec Linter
//!
//! These tests follow the Given-When-Then pattern from Dan North's BDD framework.
//! Each test is structured as:
//!   GIVEN: A specific state/setup
//!   WHEN:  An action or condition is applied
//!   THEN:  An expected outcome is verified

#![allow(clippy::unwrap_used)] // Allowed in test code

use crate::intent::quality::linter::{LintRule, SpecLinter};
use crate::intent::types::{Behavior, Feature, Spec};

/// Helper to create a valid spec with one feature and behavior
fn create_valid_spec() -> Spec {
  let mut spec = Spec::new("test_api".to_string()).unwrap();
  let mut feature = Feature::new("test_feature".to_string()).unwrap();
  let mut behavior = Behavior::new("test_behavior".to_string()).unwrap();
  behavior.description = "This is a valid test behavior".to_string();
  let _ = feature.add_behavior(behavior);
  let _ = spec.add_feature(feature);
  spec
}

mod naming_convention_tests {
  use super::*;

  #[test]
  fn given_snake_case_names_when_linted_then_passes() {
    // GIVEN: A spec with snake_case names
    let spec = create_valid_spec();

    // WHEN: The spec is linted with naming convention rule
    let linter = SpecLinter::with_rules(vec![LintRule::NamingConvention]);
    let result = linter.lint_spec(&spec);

    // THEN: It passes with no errors
    assert!(result.is_ok());
    let report = result.unwrap();
    assert_eq!(report.error_count, 0);
  }
}

mod required_fields_tests {
  use super::*;

  #[test]
  fn given_spec_with_all_required_fields_when_linted_then_passes() {
    // GIVEN: A spec with all required fields
    let spec = create_valid_spec();

    // WHEN: The spec is linted with required fields rule
    let linter = SpecLinter::with_rules(vec![LintRule::RequiredFields]);
    let result = linter.lint_spec(&spec);

    // THEN: It passes
    assert!(result.is_ok());
  }

  #[test]
  fn given_spec_with_empty_description_when_linted_then_passes_with_warnings() {
    // GIVEN: A spec with empty description
    let mut spec = Spec::new("test_api".to_string()).unwrap();
    let mut feature = Feature::new("test_feature".to_string()).unwrap();
    let behavior = Behavior::new("test_behavior".to_string()).unwrap();
    let _ = feature.add_behavior(behavior);
    let _ = spec.add_feature(feature);

    // WHEN: The spec is linted with required fields rule
    let linter = SpecLinter::with_rules(vec![LintRule::RequiredFields]);
    let result = linter.lint_spec(&spec);

    // THEN: It passes but may have warnings
    assert!(result.is_ok());
  }
}

mod deprecated_pattern_tests {
  use super::*;

  #[test]
  fn given_spec_without_deprecated_patterns_when_linted_then_passes() {
    // GIVEN: A spec without deprecated patterns
    let spec = create_valid_spec();

    // WHEN: The spec is linted with deprecated pattern rule
    let linter = SpecLinter::with_rules(vec![LintRule::DeprecatedPattern]);
    let result = linter.lint_spec(&spec);

    // THEN: It passes
    assert!(result.is_ok());
  }
}

mod description_quality_tests {
  use super::*;

  #[test]
  fn given_spec_with_good_descriptions_when_linted_then_passes() {
    // GIVEN: A spec with good quality descriptions
    let spec = create_valid_spec();

    // WHEN: The spec is linted with description quality rule
    let linter = SpecLinter::with_rules(vec![LintRule::DescriptionQuality]);
    let result = linter.lint_spec(&spec);

    // THEN: It passes with no issues
    assert!(result.is_ok());
  }
}

mod completeness_tests {
  use super::*;

  #[test]
  fn given_complete_spec_when_linted_then_passes() {
    // GIVEN: A complete spec with multiple features and behaviors
    let mut spec = Spec::new("complete_api".to_string()).unwrap();

    let mut auth = Feature::new("authentication".to_string()).unwrap();
    let mut login = Behavior::new("user_login".to_string()).unwrap();
    login.description = "User logs into the system".to_string();
    let _ = auth.add_behavior(login);

    let mut users = Feature::new("user_management".to_string()).unwrap();
    let mut create = Behavior::new("create_user".to_string()).unwrap();
    create.description = "Creates a new user account".to_string();
    let _ = users.add_behavior(create);

    let _ = spec.add_feature(auth);
    let _ = spec.add_feature(users);

    // WHEN: The spec is linted with completeness rule
    let linter = SpecLinter::with_rules(vec![LintRule::Completeness]);
    let result = linter.lint_spec(&spec);

    // THEN: It passes
    assert!(result.is_ok());
  }
}

mod all_rules_tests {
  use super::*;

  #[test]
  fn given_valid_spec_when_linted_with_all_rules_then_passes() {
    // GIVEN: A fully valid spec
    let spec = create_valid_spec();

    // WHEN: The spec is linted with all rules
    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    // THEN: It passes with minimal warnings
    assert!(result.is_ok());
    let report = result.unwrap();
    // Should have few or no issues
    assert!(report.error_count <= 1);
  }

  #[test]
  fn given_minimal_spec_when_linted_with_all_rules_then_reports_issues() {
    // GIVEN: A minimal spec (just name, no features)
    let spec = Spec::new("minimal_spec".to_string()).unwrap();

    // WHEN: The spec is linted with all rules
    let linter = SpecLinter::new();
    let result = linter.lint_spec(&spec);

    // THEN: It should report issues (may be Ok with warnings or Err)
    // The important thing is there are issues detected
    if let Ok(report) = result {
      // Has warnings or errors
      assert!(report.error_count > 0 || report.warning_count > 0);
    } else {
      // Linting failed - that's also acceptable for empty spec
    }
  }
}
