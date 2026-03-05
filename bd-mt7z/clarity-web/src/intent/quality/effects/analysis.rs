use itertools::Itertools;
use std::collections::HashMap;

use crate::intent::types::{Behavior, Feature, Spec};

use super::patterns::{
  CASCADE_KEYWORDS, NOTIFICATION_KEYWORDS, RACE_CONDITION_KEYWORDS, ROLLBACK_KEYWORDS,
  STATE_CHANGE_KEYWORDS,
};
use super::rules::detect_by_keywords;
use super::types::{Effect, EffectSeverity, EffectType, EffectsReport, SpecEffectsReport};

#[must_use]
pub fn analyze_behavior(behavior: &Behavior) -> Vec<Effect> {
  [
    detect_by_keywords(
      behavior,
      EffectType::StateChange,
      STATE_CHANGE_KEYWORDS,
      "Behavior involves state modification",
    ),
    detect_by_keywords(
      behavior,
      EffectType::Notification,
      NOTIFICATION_KEYWORDS,
      "Behavior triggers notifications",
    ),
    detect_by_keywords(
      behavior,
      EffectType::Cascade,
      CASCADE_KEYWORDS,
      "Behavior may cascade to related entities",
    ),
    detect_by_keywords(
      behavior,
      EffectType::RaceCondition,
      RACE_CONDITION_KEYWORDS,
      "Behavior has concurrent access patterns",
    ),
    detect_by_keywords(
      behavior,
      EffectType::RollbackRequired,
      ROLLBACK_KEYWORDS,
      "Behavior requires rollback capability",
    ),
  ]
  .into_iter()
  .flatten()
  .collect()
}

#[must_use]
pub fn analyze_behavior_report(behavior: &Behavior) -> EffectsReport {
  EffectsReport::new(behavior.name.clone(), analyze_behavior(behavior))
}

#[must_use]
pub fn analyze_feature(feature: &Feature) -> Vec<EffectsReport> {
  feature
    .behaviors
    .iter()
    .map(analyze_behavior_report)
    .collect()
}

#[must_use]
pub fn analyze_spec(spec: &Spec) -> SpecEffectsReport {
  let behavior_reports = spec
    .features
    .iter()
    .flat_map(|feature| feature.behaviors.iter().map(analyze_behavior_report))
    .collect::<Vec<_>>();
  SpecEffectsReport::new(spec.name.clone(), behavior_reports)
}

#[must_use]
pub fn has_critical_effects(behavior: &Behavior) -> bool {
  analyze_behavior(behavior)
    .iter()
    .any(|effect| effect.severity == EffectSeverity::Critical)
}

#[must_use]
pub fn has_high_severity_effects(behavior: &Behavior) -> bool {
  analyze_behavior(behavior)
    .iter()
    .any(|effect| effect.severity == EffectSeverity::High)
}

#[must_use]
pub fn max_effect_severity(behavior: &Behavior) -> Option<EffectSeverity> {
  analyze_behavior(behavior)
    .iter()
    .map(|effect| effect.severity)
    .max()
}

#[must_use]
pub fn count_effects_by_type(spec: &Spec) -> HashMap<EffectType, usize> {
  spec
    .features
    .iter()
    .flat_map(|feature| feature.behaviors.iter())
    .flat_map(analyze_behavior)
    .counts_by(|effect| effect.effect_type)
}

#[must_use]
pub fn behaviors_with_effect_type(spec: &Spec, effect_type: EffectType) -> Vec<String> {
  spec
    .features
    .iter()
    .flat_map(|feature| feature.behaviors.iter())
    .filter(|behavior| {
      analyze_behavior(behavior)
        .iter()
        .any(|effect| effect.effect_type == effect_type)
    })
    .map(|behavior| behavior.name.clone())
    .collect()
}
