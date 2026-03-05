use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type EffectsResult<T> = Result<T, EffectsError>;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EffectsError {
  #[error("behavior name cannot be empty")]
  EmptyBehaviorName,
  #[error("spec name cannot be empty")]
  EmptySpecName,
  #[error("invalid behavior description: {0}")]
  InvalidDescription(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectType {
  StateChange,
  Notification,
  Cascade,
  RaceCondition,
  RollbackRequired,
}

impl std::fmt::Display for EffectType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::StateChange => write!(f, "State Change"),
      Self::Notification => write!(f, "Notification"),
      Self::Cascade => write!(f, "Cascade"),
      Self::RaceCondition => write!(f, "Race Condition"),
      Self::RollbackRequired => write!(f, "Rollback Required"),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EffectSeverity {
  Low,
  Medium,
  High,
  Critical,
}

impl std::fmt::Display for EffectSeverity {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Low => write!(f, "Low"),
      Self::Medium => write!(f, "Medium"),
      Self::High => write!(f, "High"),
      Self::Critical => write!(f, "Critical"),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Effect {
  pub effect_type: EffectType,
  pub description: String,
  pub severity: EffectSeverity,
  pub suggestion: String,
  pub source_behavior: String,
}

impl Effect {
  #[must_use]
  pub const fn new(
    effect_type: EffectType,
    description: String,
    severity: EffectSeverity,
    suggestion: String,
    source_behavior: String,
  ) -> Self {
    Self {
      effect_type,
      description,
      severity,
      suggestion,
      source_behavior,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EffectsSummary {
  pub state_changes: usize,
  pub notifications: usize,
  pub cascades: usize,
  pub race_conditions: usize,
  pub rollbacks: usize,
  pub total: usize,
  pub max_severity: Option<EffectSeverity>,
}

impl EffectsSummary {
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  #[must_use]
  pub fn from_effects(effects: &[Effect]) -> Self {
    from_effect_iter(effects.iter())
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectsReport {
  pub behavior_name: String,
  pub effects: Vec<Effect>,
  pub summary: EffectsSummary,
  pub summary_text: String,
}

impl EffectsReport {
  #[must_use]
  pub fn new(behavior_name: String, effects: Vec<Effect>) -> Self {
    let summary = EffectsSummary::from_effects(&effects);
    let summary_text = summary_text_for_behavior(&summary, &behavior_name);
    Self {
      behavior_name,
      effects,
      summary,
      summary_text,
    }
  }

  #[must_use]
  pub fn has_critical_effects(&self) -> bool {
    self
      .effects
      .iter()
      .any(|effect| effect.severity == EffectSeverity::Critical)
  }

  #[must_use]
  pub fn effects_by_type(&self, effect_type: EffectType) -> Vec<&Effect> {
    self
      .effects
      .iter()
      .filter(|effect| effect.effect_type == effect_type)
      .collect()
  }

  #[must_use]
  pub fn effects_by_severity(&self, severity: EffectSeverity) -> Vec<&Effect> {
    self
      .effects
      .iter()
      .filter(|effect| effect.severity == severity)
      .collect()
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecEffectsReport {
  pub spec_name: String,
  pub behavior_reports: Vec<EffectsReport>,
  pub total_summary: EffectsSummary,
  pub overall_summary: String,
}

impl SpecEffectsReport {
  #[must_use]
  pub fn new(spec_name: String, behavior_reports: Vec<EffectsReport>) -> Self {
    let all_effects = behavior_reports
      .iter()
      .flat_map(|report| report.effects.iter());
    let total_summary = from_effect_iter(all_effects);
    let overall_summary = summary_text_for_spec(&total_summary, behavior_reports.len());
    Self {
      spec_name,
      behavior_reports,
      total_summary,
      overall_summary,
    }
  }

  #[must_use]
  pub fn critical_effects(&self) -> Vec<&Effect> {
    self
      .behavior_reports
      .iter()
      .flat_map(|report| report.effects.iter())
      .filter(|effect| effect.severity == EffectSeverity::Critical)
      .collect()
  }

  #[must_use]
  pub fn high_severity_effects(&self) -> Vec<&Effect> {
    self
      .behavior_reports
      .iter()
      .flat_map(|report| report.effects.iter())
      .filter(|effect| effect.severity == EffectSeverity::High)
      .collect()
  }
}

fn from_effect_iter<'a>(effects: impl Iterator<Item = &'a Effect> + Clone) -> EffectsSummary {
  let total = effects.clone().count();
  let max_severity = effects.clone().map(|effect| effect.severity).max();
  let state_changes = effects
    .clone()
    .filter(|effect| effect.effect_type == EffectType::StateChange)
    .count();
  let notifications = effects
    .clone()
    .filter(|effect| effect.effect_type == EffectType::Notification)
    .count();
  let cascades = effects
    .clone()
    .filter(|effect| effect.effect_type == EffectType::Cascade)
    .count();
  let race_conditions = effects
    .clone()
    .filter(|effect| effect.effect_type == EffectType::RaceCondition)
    .count();
  let rollbacks = effects
    .filter(|effect| effect.effect_type == EffectType::RollbackRequired)
    .count();
  EffectsSummary {
    state_changes,
    notifications,
    cascades,
    race_conditions,
    rollbacks,
    total,
    max_severity,
  }
}

fn summary_text_for_behavior(summary: &EffectsSummary, behavior_name: &str) -> String {
  if summary.total == 0 {
    return format!("No second-order effects detected in {behavior_name}");
  }

  let parts = [
    (summary.state_changes > 0).then_some(format!("{} state change(s)", summary.state_changes)),
    (summary.notifications > 0).then_some(format!("{} notification(s)", summary.notifications)),
    (summary.cascades > 0).then_some(format!("{} cascade(s)", summary.cascades)),
    (summary.race_conditions > 0)
      .then_some(format!("{} race condition(s)", summary.race_conditions)),
    (summary.rollbacks > 0).then_some(format!("{} rollback(s)", summary.rollbacks)),
  ]
  .into_iter()
  .flatten()
  .collect::<Vec<_>>()
  .join(", ");

  let severity = summary
    .max_severity
    .map_or_else(String::new, |level| format!(" Highest severity: {level}."));

  format!(
    "Found {} effect(s) in {behavior_name}: {parts}.{severity}",
    summary.total
  )
}

fn summary_text_for_spec(summary: &EffectsSummary, behavior_count: usize) -> String {
  if summary.total == 0 {
    format!("No second-order effects detected across {behavior_count} behavior(s)")
  } else {
    let severity = summary
      .max_severity
      .map_or_else(String::new, |level| format!(" with {level} severity"));
    format!(
            "Found {} effect(s) across {behavior_count} behavior(s){severity}. State changes: {}, Notifications: {}, Cascades: {}, Race conditions: {}, Rollbacks: {}",
            summary.total,
            summary.state_changes,
            summary.notifications,
            summary.cascades,
            summary.race_conditions,
            summary.rollbacks
        )
  }
}
