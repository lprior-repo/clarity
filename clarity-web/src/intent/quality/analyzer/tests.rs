#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::intent::quality::analyzer::{
  analyze_spec, calculate_ai_readiness_score, calculate_clarity_score, calculate_coverage_score,
  calculate_testability_score,
};
use crate::intent::types::{AIHints, Behavior, Feature, Spec, Verification};

fn minimal_spec() -> Spec {
  Spec::new("test-spec".to_string()).expect("test setup should create spec")
}

#[test]
fn analyze_spec_empty_has_issues() {
  let report = analyze_spec(&minimal_spec());
  assert!(report.has_issues());
  assert!(report.coverage_score < 100);
}

#[test]
fn score_functions_return_reasonable_values() {
  let mut spec = minimal_spec();
  let behavior = Behavior::new("create_user".to_string())
    .expect("test setup should create behavior")
    .with_description("Create user and handle invalid input".to_string())
    .with_verification(Verification::new(
      "unit_test".to_string(),
      "verify error cases".to_string(),
    ));

  let mut feature = Feature::new("users".to_string()).expect("test setup should create feature");

  feature
    .add_behavior(behavior)
    .expect("test setup should add behavior");
  spec
    .add_feature(feature)
    .expect("test setup should add feature");

  let mut ai_hints = AIHints::default();
  ai_hints.implementation.architecture = "clean".to_string();
  spec = spec.with_ai_hints(ai_hints);

  assert!(calculate_coverage_score(&spec) > 40);
  assert!(calculate_clarity_score(&spec) > 50);
  assert!(calculate_testability_score(&spec) > 40);
  assert!(calculate_ai_readiness_score(&spec) > 40);
}
