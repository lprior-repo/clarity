#![allow(clippy::expect_used)]

use crate::intent::quality::effects::{
  analyze_behavior, analyze_spec, behaviors_with_effect_type, count_effects_by_type,
  EffectSeverity, EffectType,
};
use crate::intent::types::{Behavior, Feature, Spec};

#[test]
fn analyze_behavior_detects_state_and_notification() {
  let behavior = Behavior::new("create_user".to_string())
    .expect("test setup")
    .with_description("Create user and send welcome email".to_string());
  let effects = analyze_behavior(&behavior);
  assert!(effects
    .iter()
    .any(|effect| effect.effect_type == EffectType::StateChange));
  assert!(effects
    .iter()
    .any(|effect| effect.effect_type == EffectType::Notification));
}

#[test]
fn severity_detects_critical() {
  let behavior = Behavior::new("purge".to_string())
    .expect("test setup")
    .with_description("Emergency unrecoverable wipe".to_string());
  let effects = analyze_behavior(&behavior);
  assert!(effects
    .iter()
    .any(|effect| effect.severity == EffectSeverity::Critical));
}

#[test]
fn spec_aggregations_work() {
  let mut feature = Feature::new("users".to_string()).expect("test setup");
  feature
    .add_behavior(
      Behavior::new("create".to_string())
        .expect("test setup")
        .with_description("Create user and send email".to_string()),
    )
    .expect("test setup");
  feature
    .add_behavior(
      Behavior::new("view".to_string())
        .expect("test setup")
        .with_description("View profile".to_string()),
    )
    .expect("test setup");

  let mut spec = Spec::new("test".to_string()).expect("test setup");
  spec.add_feature(feature).expect("test setup");

  let report = analyze_spec(&spec);
  assert_eq!(report.behavior_reports.len(), 2);
  let counts = count_effects_by_type(&spec);
  assert!(counts.contains_key(&EffectType::StateChange));

  let behaviors = behaviors_with_effect_type(&spec, EffectType::StateChange);
  assert!(behaviors.contains(&"create".to_string()));
}
