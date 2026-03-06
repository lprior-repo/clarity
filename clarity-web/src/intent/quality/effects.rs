//! Effects Analyzer (WP29) - Second-Order Effect Detection
//!
//! This module provides analysis of behaviors to detect second-order effects
//! such as state changes, notifications, cascade effects, race conditions,
//! and rollback requirements.
//!
//! ## Design Principles
//!
//! - **Zero panics**: All fallible operations return `Result<T, E>`
//! - **Pure functions**: Analysis is deterministic and side-effect free
//! - **Composable**: Uses iterator pipelines with itertools
//!
//! ## Effect Categories
//!
//! - **`StateChange`**: Create/update/delete operations on data
//! - **Notification**: Email, webhook, callback triggers
//! - **Cascade**: Related records affected by operations
//! - **`RaceCondition`**: Concurrent modification risks
//! - **`RollbackRequired`**: Operations requiring reversibility

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(
  clippy::missing_errors_doc,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro
)]

use crate::intent::types::{Behavior, Feature, Spec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

// =============================================================================
// Result Type
// =============================================================================

/// Effects analysis result type
pub type EffectsResult<T> = Result<T, EffectsError>;

// =============================================================================
// Error Types
// =============================================================================

/// Effects analysis error
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EffectsError {
  /// Behavior name is empty
  #[error("behavior name cannot be empty")]
  EmptyBehaviorName,

  /// Spec name is empty
  #[error("spec name cannot be empty")]
  EmptySpecName,

  /// Invalid behavior description
  #[error("invalid behavior description: {0}")]
  InvalidDescription(String),
}

// =============================================================================
// Effect Types
// =============================================================================

/// Second-order effect types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectType {
  /// State change effect (create/update/delete)
  StateChange,
  /// Notification effect (email, webhook, callback)
  Notification,
  /// Cascade effect (related records affected)
  Cascade,
  /// Race condition risk (concurrent modifications)
  RaceCondition,
  /// Rollback required (reversibility needed)
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

/// Effect severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EffectSeverity {
  /// Low severity - minimal impact
  Low,
  /// Medium severity - moderate impact
  Medium,
  /// High severity - significant impact
  High,
  /// Critical severity - severe impact
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

// =============================================================================
// Effect Detection Patterns
// =============================================================================

/// Keywords indicating state change operations
const STATE_CHANGE_KEYWORDS: &[&str] = &[
  "create", "update", "delete", "remove", "modify", "insert", "save", "write", "destroy",
  "archive", "restore", "purge", "reset", "clear", "set", "assign", "transfer", "move", "copy",
  "clone", "merge",
];

/// Keywords indicating notification operations
const NOTIFICATION_KEYWORDS: &[&str] = &[
  "email",
  "notify",
  "send",
  "alert",
  "webhook",
  "callback",
  "broadcast",
  "publish",
  "emit",
  "push",
  "sms",
  "message",
  "mail",
  "notification",
  "subscribe",
  "dispatch",
  "trigger",
  "announce",
  "signal",
];

/// Keywords indicating cascade operations
const CASCADE_KEYWORDS: &[&str] = &[
  "cascade",
  "propagate",
  "affect",
  "related",
  "dependent",
  "linked",
  "associated",
  "connected",
  "chain",
  "ripple",
  "downstream",
  "upstream",
  "sync",
  "synchronize",
  "replicate",
  "mirror",
];

/// Keywords indicating race condition risks
const RACE_CONDITION_KEYWORDS: &[&str] = &[
  "concurrent",
  "parallel",
  "simultaneous",
  "atomic",
  "lock",
  "mutex",
  "race",
  "thread",
  "async",
  "await",
  "concurrent",
  "conflict",
  "exclusive",
  "shared",
  "synchronize",
  "volatile",
  "interleave",
];

/// Keywords indicating rollback requirements
const ROLLBACK_KEYWORDS: &[&str] = &[
  "transaction",
  "rollback",
  "revert",
  "undo",
  "compensate",
  "recover",
  "restore",
  "backup",
  "snapshot",
  "version",
  "history",
  "audit",
  "reversible",
  "idempotent",
  "replay",
];

/// Keywords indicating high severity
const HIGH_SEVERITY_KEYWORDS: &[&str] = &[
  "critical",
  "severe",
  "urgent",
  "important",
  "essential",
  "vital",
  "irreversible",
  "permanent",
  "destructive",
  "dangerous",
  "sensitive",
  "secure",
  "encrypt",
  "auth",
  "permission",
  "admin",
  "root",
];

/// Keywords indicating critical severity
const CRITICAL_SEVERITY_KEYWORDS: &[&str] = &[
  "emergency",
  "fatal",
  "catastrophic",
  "unrecoverable",
  "corrupt",
  "breach",
  "expose",
  "leak",
  "compromise",
  "invalidate",
  "destroy",
  "wipe",
  "eliminate",
  "terminate",
  "abort",
];

// =============================================================================
// Effect Structure
// =============================================================================

/// Effect detected in a behavior
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Effect {
  /// Type of effect detected
  pub effect_type: EffectType,
  /// Human-readable description of the effect
  pub description: String,
  /// Severity level of the effect
  pub severity: EffectSeverity,
  /// Suggested mitigation or consideration
  pub suggestion: String,
  /// Name of the behavior where effect was detected
  pub source_behavior: String,
}

impl Effect {
  /// Create a new effect
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

// =============================================================================
// Effects Report
// =============================================================================

/// Summary of effects by type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EffectsSummary {
  /// Count of state change effects
  pub state_changes: usize,
  /// Count of notification effects
  pub notifications: usize,
  /// Count of cascade effects
  pub cascades: usize,
  /// Count of race condition effects
  pub race_conditions: usize,
  /// Count of rollback required effects
  pub rollbacks: usize,
  /// Total effect count
  pub total: usize,
  /// Highest severity detected
  pub max_severity: Option<EffectSeverity>,
}

impl EffectsSummary {
  /// Create a new empty summary
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Create summary from a list of effects
  #[must_use]
  pub fn from_effects(effects: &[Effect]) -> Self {
    let counts = effects.iter().fold(HashMap::new(), |mut acc, effect| {
      *acc.entry(effect.effect_type).or_insert(0) += 1;
      acc
    });

    let max_severity = effects.iter().map(|e| e.severity).max();

    Self {
      state_changes: *counts.get(&EffectType::StateChange).unwrap_or(&0),
      notifications: *counts.get(&EffectType::Notification).unwrap_or(&0),
      cascades: *counts.get(&EffectType::Cascade).unwrap_or(&0),
      race_conditions: *counts.get(&EffectType::RaceCondition).unwrap_or(&0),
      rollbacks: *counts.get(&EffectType::RollbackRequired).unwrap_or(&0),
      total: effects.len(),
      max_severity,
    }
  }
}

/// Effects report for a behavior or spec
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectsReport {
  /// Name of the analyzed entity
  pub behavior_name: String,
  /// All detected effects
  pub effects: Vec<Effect>,
  /// Summary of effects
  pub summary: EffectsSummary,
  /// Human-readable summary text
  pub summary_text: String,
}

impl EffectsReport {
  /// Create a new effects report
  #[must_use]
  pub fn new(behavior_name: String, effects: Vec<Effect>) -> Self {
    let summary = EffectsSummary::from_effects(&effects);
    let summary_text = Self::generate_summary_text(&summary, behavior_name.as_str());

    Self {
      behavior_name,
      effects,
      summary,
      summary_text,
    }
  }

  /// Generate human-readable summary text
  fn generate_summary_text(summary: &EffectsSummary, name: &str) -> String {
    if summary.total == 0 {
      return format!("No second-order effects detected in {name}");
    }

    let severity_str = summary
      .max_severity
      .as_ref()
      .map_or_else(String::new, |s| format!("Highest severity: {s}. "));

    let parts: Vec<String> = [
      (summary.state_changes > 0).then(|| format!("{} state change(s)", summary.state_changes)),
      (summary.notifications > 0).then(|| format!("{} notification(s)", summary.notifications)),
      (summary.cascades > 0).then(|| format!("{} cascade(s)", summary.cascades)),
      (summary.race_conditions > 0)
        .then(|| format!("{} race condition(s)", summary.race_conditions)),
      (summary.rollbacks > 0).then(|| format!("{} rollback(s)", summary.rollbacks)),
    ]
    .into_iter()
    .flatten()
    .collect();

    format!(
      "Found {} effect(s) in {}: {}. {}",
      summary.total,
      name,
      parts.join(", "),
      severity_str
    )
    .trim_end()
    .to_string()
  }

  /// Check if any critical effects exist
  #[must_use]
  pub fn has_critical_effects(&self) -> bool {
    self
      .effects
      .iter()
      .any(|e| e.severity == EffectSeverity::Critical)
  }

  /// Get effects by type
  #[must_use]
  pub fn effects_by_type(&self, effect_type: EffectType) -> Vec<&Effect> {
    self
      .effects
      .iter()
      .filter(|e| e.effect_type == effect_type)
      .collect()
  }

  /// Get effects by severity
  #[must_use]
  pub fn effects_by_severity(&self, severity: EffectSeverity) -> Vec<&Effect> {
    self
      .effects
      .iter()
      .filter(|e| e.severity == severity)
      .collect()
  }
}

// =============================================================================
// Spec Report
// =============================================================================

/// Comprehensive effects report for an entire spec
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecEffectsReport {
  /// Spec name
  pub spec_name: String,
  /// Reports per behavior
  pub behavior_reports: Vec<EffectsReport>,
  /// Aggregated summary
  pub total_summary: EffectsSummary,
  /// Overall summary text
  pub overall_summary: String,
}

impl SpecEffectsReport {
  /// Create a new spec effects report
  #[must_use]
  pub fn new(spec_name: String, behavior_reports: Vec<EffectsReport>) -> Self {
    let all_effects: Vec<&Effect> = behavior_reports
      .iter()
      .flat_map(|r| r.effects.iter())
      .collect();

    let total_summary = Self::compute_total_summary(&all_effects);
    let overall_summary = Self::generate_overall_summary(&total_summary, &behavior_reports);

    Self {
      spec_name,
      behavior_reports,
      total_summary,
      overall_summary,
    }
  }

  /// Compute total summary from all effects
  fn compute_total_summary(effects: &[&Effect]) -> EffectsSummary {
    let counts = effects.iter().fold(HashMap::new(), |mut acc, effect| {
      *acc.entry(effect.effect_type).or_insert(0) += 1;
      acc
    });

    let max_severity = effects.iter().map(|e| e.severity).max();

    EffectsSummary {
      state_changes: *counts.get(&EffectType::StateChange).unwrap_or(&0),
      notifications: *counts.get(&EffectType::Notification).unwrap_or(&0),
      cascades: *counts.get(&EffectType::Cascade).unwrap_or(&0),
      race_conditions: *counts.get(&EffectType::RaceCondition).unwrap_or(&0),
      rollbacks: *counts.get(&EffectType::RollbackRequired).unwrap_or(&0),
      total: effects.len(),
      max_severity,
    }
  }

  /// Generate overall summary text
  fn generate_overall_summary(summary: &EffectsSummary, reports: &[EffectsReport]) -> String {
    if summary.total == 0 {
      return format!(
        "No second-order effects detected across {} behavior(s)",
        reports.len()
      );
    }

    let behaviors_with_effects = reports.iter().filter(|r| !r.effects.is_empty()).count();

    let severity_str = summary
      .max_severity
      .as_ref()
      .map_or_else(String::new, |s| format!(" with {s} severity"));

    format!(
      "Found {} effect(s) across {} of {} behavior(s){}. \
             State changes: {}, Notifications: {}, Cascades: {}, \
             Race conditions: {}, Rollbacks: {}",
      summary.total,
      behaviors_with_effects,
      reports.len(),
      severity_str,
      summary.state_changes,
      summary.notifications,
      summary.cascades,
      summary.race_conditions,
      summary.rollbacks
    )
  }

  /// Get all critical effects across all behaviors
  #[must_use]
  pub fn critical_effects(&self) -> Vec<&Effect> {
    self
      .behavior_reports
      .iter()
      .flat_map(|r| r.effects.iter())
      .filter(|e| e.severity == EffectSeverity::Critical)
      .collect()
  }

  /// Get all high severity effects across all behaviors
  #[must_use]
  pub fn high_severity_effects(&self) -> Vec<&Effect> {
    self
      .behavior_reports
      .iter()
      .flat_map(|r| r.effects.iter())
      .filter(|e| e.severity == EffectSeverity::High)
      .collect()
  }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if text contains any keywords from a list (case-insensitive)
fn contains_keywords(text: &str, keywords: &[&str]) -> bool {
  let lower = text.to_lowercase();
  keywords.iter().any(|kw| lower.contains(kw))
}

/// Count keyword matches in text (case-insensitive)
fn count_keyword_matches(text: &str, keywords: &[&str]) -> usize {
  let lower = text.to_lowercase();
  keywords.iter().filter(|kw| lower.contains(*kw)).count()
}

/// Estimate severity based on keywords and effect type
fn estimate_severity(text: &str, effect_type: EffectType) -> EffectSeverity {
  // Check for critical keywords first
  if contains_keywords(text, CRITICAL_SEVERITY_KEYWORDS) {
    return EffectSeverity::Critical;
  }

  // Check for high severity keywords
  if contains_keywords(text, HIGH_SEVERITY_KEYWORDS) {
    return EffectSeverity::High;
  }

  // Default severity based on effect type
  match effect_type {
    EffectType::StateChange => {
      // Delete operations are higher severity
      let lower = text.to_lowercase();
      if lower.contains("delete") || lower.contains("destroy") || lower.contains("purge") {
        EffectSeverity::High
      } else if lower.contains("create") || lower.contains("insert") {
        EffectSeverity::Medium
      } else {
        EffectSeverity::Low
      }
    }
    EffectType::Notification => EffectSeverity::Low,
    EffectType::Cascade | EffectType::RollbackRequired => EffectSeverity::Medium,
    EffectType::RaceCondition => EffectSeverity::High,
  }
}

/// Generate a suggestion based on effect type and context
fn generate_suggestion(effect_type: EffectType, text: &str, severity: EffectSeverity) -> String {
  match effect_type {
        EffectType::StateChange => {
            let lower = text.to_lowercase();
            if lower.contains("delete") || lower.contains("destroy") {
                "Consider implementing soft delete or archive pattern. Ensure proper authorization and audit logging."
                    .to_string()
            } else if lower.contains("update") || lower.contains("modify") {
                "Consider implementing optimistic locking or versioning. Validate all inputs before applying changes."
                    .to_string()
            } else {
                "Validate inputs, ensure atomicity, and consider idempotency for retries."
                    .to_string()
            }
        }
        EffectType::Notification => {
            "Consider implementing retry logic, dead letter queues, and rate limiting. Ensure notifications are idempotent."
                .to_string()
        }
        EffectType::Cascade => {
            "Document all cascade effects. Consider implementing saga pattern for distributed operations."
                .to_string()
        }
        EffectType::RaceCondition => {
            match severity {
                EffectSeverity::Critical | EffectSeverity::High => {
                    "Implement proper locking mechanisms (pessimistic or optimistic). Consider using database transactions with appropriate isolation levels."
                        .to_string()
                }
                _ => {
                    "Consider using atomic operations or compare-and-swap patterns."
                        .to_string()
                }
            }
        }
        EffectType::RollbackRequired => {
            "Implement compensation actions for rollback scenarios. Consider using saga pattern for distributed transactions."
                .to_string()
        }
    }
}

/// Create an effect from detected pattern
fn create_effect(
  effect_type: EffectType,
  behavior_name: &str,
  description: &str,
  context_text: &str,
) -> Effect {
  let severity = estimate_severity(context_text, effect_type);
  let suggestion = generate_suggestion(effect_type, context_text, severity);

  Effect::new(
    effect_type,
    format!("{effect_type} detected in '{behavior_name}': {description}"),
    severity,
    suggestion,
    behavior_name.to_string(),
  )
}

// =============================================================================
// Analysis Functions
// =============================================================================

/// Detect state change effects in behavior
fn detect_state_changes(behavior: &Behavior) -> Vec<Effect> {
  let combined_text = format!(
    "{} {} {} {}",
    behavior.name,
    behavior.description,
    behavior.preconditions.join(" "),
    behavior.postconditions.join(" ")
  );

  if !contains_keywords(&combined_text, STATE_CHANGE_KEYWORDS) {
    return Vec::new();
  }

  let matched_keywords: Vec<&str> = STATE_CHANGE_KEYWORDS
    .iter()
    .filter(|kw| combined_text.to_lowercase().contains(*kw))
    .copied()
    .collect();

  let description = format!(
    "Behavior involves state modification: {}",
    matched_keywords.join(", ")
  );

  vec![create_effect(
    EffectType::StateChange,
    &behavior.name,
    &description,
    &combined_text,
  )]
}

/// Detect notification effects in behavior
fn detect_notifications(behavior: &Behavior) -> Vec<Effect> {
  let combined_text = format!(
    "{} {} {} {}",
    behavior.name,
    behavior.description,
    behavior.preconditions.join(" "),
    behavior.postconditions.join(" ")
  );

  if !contains_keywords(&combined_text, NOTIFICATION_KEYWORDS) {
    return Vec::new();
  }

  let matched_keywords: Vec<&str> = NOTIFICATION_KEYWORDS
    .iter()
    .filter(|kw| combined_text.to_lowercase().contains(*kw))
    .copied()
    .collect();

  let description = format!(
    "Behavior triggers notifications: {}",
    matched_keywords.join(", ")
  );

  vec![create_effect(
    EffectType::Notification,
    &behavior.name,
    &description,
    &combined_text,
  )]
}

/// Detect cascade effects in behavior
fn detect_cascades(behavior: &Behavior) -> Vec<Effect> {
  let combined_text = format!(
    "{} {} {} {}",
    behavior.name,
    behavior.description,
    behavior.preconditions.join(" "),
    behavior.postconditions.join(" ")
  );

  if !contains_keywords(&combined_text, CASCADE_KEYWORDS) {
    return Vec::new();
  }

  let matched_keywords: Vec<&str> = CASCADE_KEYWORDS
    .iter()
    .filter(|kw| combined_text.to_lowercase().contains(*kw))
    .copied()
    .collect();

  let description = format!(
    "Behavior may cascade to related entities: {}",
    matched_keywords.join(", ")
  );

  vec![create_effect(
    EffectType::Cascade,
    &behavior.name,
    &description,
    &combined_text,
  )]
}

/// Detect race condition risks in behavior
fn detect_race_conditions(behavior: &Behavior) -> Vec<Effect> {
  let combined_text = format!(
    "{} {} {} {}",
    behavior.name,
    behavior.description,
    behavior.preconditions.join(" "),
    behavior.postconditions.join(" ")
  );

  if !contains_keywords(&combined_text, RACE_CONDITION_KEYWORDS) {
    return Vec::new();
  }

  let matched_keywords: Vec<&str> = RACE_CONDITION_KEYWORDS
    .iter()
    .filter(|kw| combined_text.to_lowercase().contains(*kw))
    .copied()
    .collect();

  let description = format!(
    "Behavior has concurrent access patterns: {}",
    matched_keywords.join(", ")
  );

  vec![create_effect(
    EffectType::RaceCondition,
    &behavior.name,
    &description,
    &combined_text,
  )]
}

/// Detect rollback requirements in behavior
fn detect_rollback_requirements(behavior: &Behavior) -> Vec<Effect> {
  let combined_text = format!(
    "{} {} {} {}",
    behavior.name,
    behavior.description,
    behavior.preconditions.join(" "),
    behavior.postconditions.join(" ")
  );

  if !contains_keywords(&combined_text, ROLLBACK_KEYWORDS) {
    return Vec::new();
  }

  let matched_keywords: Vec<&str> = ROLLBACK_KEYWORDS
    .iter()
    .filter(|kw| combined_text.to_lowercase().contains(*kw))
    .copied()
    .collect();

  let description = format!(
    "Behavior requires rollback capability: {}",
    matched_keywords.join(", ")
  );

  vec![create_effect(
    EffectType::RollbackRequired,
    &behavior.name,
    &description,
    &combined_text,
  )]
}

// =============================================================================
// Public API
// =============================================================================

/// Analyze a single behavior for second-order effects
///
/// Examines the behavior's name, description, preconditions, and postconditions
/// to detect potential second-order effects including:
/// - State changes (create/update/delete operations)
/// - Notifications (email, webhook, callback triggers)
/// - Cascade effects (related records affected)
/// - Race conditions (concurrent modification risks)
/// - Rollback requirements (reversibility needs)
///
/// # Arguments
///
/// * `behavior` - The behavior to analyze
///
/// # Returns
///
/// A vector of detected effects
///
/// # Example
///
/// ```
/// use clarity_web::intent::types::Behavior;
/// use clarity_web::intent::quality::effects::{analyze_behavior, EffectType};
///
/// let behavior = Behavior::new("create_user".to_string())
///     .expect("valid behavior")
///     .with_description("Create a new user and send welcome email".to_string());
///
/// let effects = analyze_behavior(&behavior);
/// assert!(!effects.is_empty());
///
/// let has_state_change = effects.iter().any(|e| e.effect_type == EffectType::StateChange);
/// let has_notification = effects.iter().any(|e| e.effect_type == EffectType::Notification);
/// assert!(has_state_change || has_notification);
/// ```
#[must_use]
pub fn analyze_behavior(behavior: &Behavior) -> Vec<Effect> {
  let mut effects = Vec::new();

  // Detect each type of effect
  effects.extend(detect_state_changes(behavior));
  effects.extend(detect_notifications(behavior));
  effects.extend(detect_cascades(behavior));
  effects.extend(detect_race_conditions(behavior));
  effects.extend(detect_rollback_requirements(behavior));

  effects
}

/// Analyze a behavior and return a detailed report
///
/// # Arguments
///
/// * `behavior` - The behavior to analyze
///
/// # Returns
///
/// An `EffectsReport` with all detected effects and summary
///
/// # Example
///
/// ```
/// use clarity_web::intent::types::Behavior;
/// use clarity_web::intent::quality::effects::analyze_behavior_report;
///
/// let behavior = Behavior::new("delete_user".to_string())
///     .expect("valid behavior")
///     .with_description("Permanently delete user and cascade to all related records".to_string());
///
/// let report = analyze_behavior_report(&behavior);
/// assert!(!report.effects.is_empty());
/// ```
#[must_use]
pub fn analyze_behavior_report(behavior: &Behavior) -> EffectsReport {
  let effects = analyze_behavior(behavior);
  EffectsReport::new(behavior.name.clone(), effects)
}

/// Analyze all behaviors in a feature for second-order effects
///
/// # Arguments
///
/// * `feature` - The feature whose behaviors to analyze
///
/// # Returns
///
/// A vector of effects reports, one per behavior
///
/// # Example
///
/// ```
/// use clarity_web::intent::types::{Feature, Behavior};
/// use clarity_web::intent::quality::effects::analyze_feature;
///
/// let mut feature = Feature::new("user_management".to_string())
///     .expect("valid feature");
///
/// let behavior = Behavior::new("create_user".to_string())
///     .expect("valid behavior")
///     .with_description("Create a new user account".to_string());
///
/// feature.add_behavior(behavior).expect("should add behavior");
///
/// let reports = analyze_feature(&feature);
/// assert_eq!(reports.len(), 1);
/// ```
#[must_use]
pub fn analyze_feature(feature: &Feature) -> Vec<EffectsReport> {
  feature
    .behaviors
    .iter()
    .map(analyze_behavior_report)
    .collect()
}

/// Analyze all behaviors in a spec for second-order effects
///
/// Examines all behaviors across all features and generates a comprehensive
/// report with aggregated statistics and per-behavior details.
///
/// # Arguments
///
/// * `spec` - The spec to analyze
///
/// # Returns
///
/// A `SpecEffectsReport` with all detected effects and summaries
///
/// # Example
///
/// ```
/// use clarity_web::intent::types::{Spec, Feature, Behavior};
/// use clarity_web::intent::quality::effects::analyze_spec;
///
/// let mut spec = Spec::new("user_system".to_string()).expect("valid spec");
///
/// let mut feature = Feature::new("auth".to_string()).expect("valid feature");
/// let behavior = Behavior::new("login".to_string())
///     .expect("valid behavior")
///     .with_description("Authenticate user and update last login timestamp".to_string());
///
/// feature.add_behavior(behavior).expect("should add behavior");
/// spec.add_feature(feature).expect("should add feature");
///
/// let report = analyze_spec(&spec);
/// assert!(!report.behavior_reports.is_empty());
/// ```
#[must_use]
pub fn analyze_spec(spec: &Spec) -> SpecEffectsReport {
  let behavior_reports: Vec<EffectsReport> = spec
    .features
    .iter()
    .flat_map(|feature| {
      feature
        .behaviors
        .iter()
        .map(analyze_behavior_report)
        .collect::<Vec<_>>()
    })
    .collect();

  SpecEffectsReport::new(spec.name.clone(), behavior_reports)
}

/// Check if a behavior has any critical effects
///
/// # Arguments
///
/// * `behavior` - The behavior to check
///
/// # Returns
///
/// `true` if the behavior has at least one critical severity effect
#[must_use]
pub fn has_critical_effects(behavior: &Behavior) -> bool {
  analyze_behavior(behavior)
    .iter()
    .any(|e| e.severity == EffectSeverity::Critical)
}

/// Check if a behavior has any high severity effects
///
/// # Arguments
///
/// * `behavior` - The behavior to check
///
/// # Returns
///
/// `true` if the behavior has at least one high severity effect
#[must_use]
pub fn has_high_severity_effects(behavior: &Behavior) -> bool {
  analyze_behavior(behavior)
    .iter()
    .any(|e| e.severity == EffectSeverity::High)
}

/// Get the maximum severity of effects in a behavior
///
/// # Arguments
///
/// * `behavior` - The behavior to analyze
///
/// # Returns
///
/// The highest severity level found, or `None` if no effects detected
#[must_use]
pub fn max_effect_severity(behavior: &Behavior) -> Option<EffectSeverity> {
  analyze_behavior(behavior).iter().map(|e| e.severity).max()
}

/// Count effects by type in a spec
///
/// # Arguments
///
/// * `spec` - The spec to analyze
///
/// # Returns
///
/// A `HashMap` mapping effect types to their counts
#[must_use]
pub fn count_effects_by_type(spec: &Spec) -> HashMap<EffectType, usize> {
  spec
    .features
    .iter()
    .flat_map(|f| f.behaviors.iter())
    .flat_map(analyze_behavior)
    .fold(HashMap::<EffectType, usize>::new(), |mut acc, effect| {
      *acc.entry(effect.effect_type).or_insert(0) += 1;
      acc
    })
}

/// Get all behaviors with a specific effect type
///
/// # Arguments
///
/// * `spec` - The spec to search
/// * `effect_type` - The effect type to filter by
///
/// # Returns
///
/// A vector of behavior names that have the specified effect type
#[must_use]
pub fn behaviors_with_effect_type(spec: &Spec, effect_type: EffectType) -> Vec<String> {
  spec
    .features
    .iter()
    .flat_map(|f| f.behaviors.iter())
    .filter(|b| {
      analyze_behavior(b)
        .iter()
        .any(|e| e.effect_type == effect_type)
    })
    .map(|b| b.name.clone())
    .collect()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::unnecessary_debug_formatting,
  clippy::match_same_arms,
  clippy::option_if_let_else,
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro,
  clippy::needless_pass_by_value
)]
mod tests {
  use super::*;

  // -------------------------------------------------------------------------
  // Helper function tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_contains_keywords_match() {
    assert!(contains_keywords(
      "Create a new user",
      STATE_CHANGE_KEYWORDS
    ));
    assert!(contains_keywords(
      "Send email notification",
      NOTIFICATION_KEYWORDS
    ));
    assert!(contains_keywords("CREATE", STATE_CHANGE_KEYWORDS)); // case-insensitive
  }

  #[test]
  fn test_contains_keywords_no_match() {
    assert!(!contains_keywords(
      "Read only operation",
      STATE_CHANGE_KEYWORDS
    ));
    assert!(!contains_keywords("No effects here", NOTIFICATION_KEYWORDS));
  }

  #[test]
  fn test_count_keyword_matches() {
    let text = "Create and update user, then send notification";
    // "create" and "update" are state change keywords
    assert_eq!(count_keyword_matches(text, STATE_CHANGE_KEYWORDS), 2);
    // "send" and "notification" are both notification keywords
    assert_eq!(count_keyword_matches(text, NOTIFICATION_KEYWORDS), 2);
  }

  #[test]
  fn test_estimate_severity_critical() {
    assert_eq!(
      estimate_severity("Emergency fatal breach", EffectType::StateChange),
      EffectSeverity::Critical
    );
    assert_eq!(
      estimate_severity("Unrecoverable corrupt data", EffectType::Cascade),
      EffectSeverity::Critical
    );
  }

  #[test]
  fn test_estimate_severity_high() {
    assert_eq!(
      estimate_severity("Critical admin operation", EffectType::StateChange),
      EffectSeverity::High
    );
    assert_eq!(
      estimate_severity("Delete user permanently", EffectType::StateChange),
      EffectSeverity::High
    );
  }

  #[test]
  fn test_estimate_severity_medium() {
    assert_eq!(
      estimate_severity("Create new record", EffectType::StateChange),
      EffectSeverity::Medium
    );
    assert_eq!(
      estimate_severity("Cascade to related", EffectType::Cascade),
      EffectSeverity::Medium
    );
  }

  #[test]
  fn test_estimate_severity_low() {
    assert_eq!(
      estimate_severity("Send notification", EffectType::Notification),
      EffectSeverity::Low
    );
    assert_eq!(
      estimate_severity("Update status", EffectType::StateChange),
      EffectSeverity::Low
    );
  }

  #[test]
  fn test_race_condition_severity() {
    // Race conditions default to high
    assert_eq!(
      estimate_severity("Concurrent access", EffectType::RaceCondition),
      EffectSeverity::High
    );
  }

  // -------------------------------------------------------------------------
  // Effect creation tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_effect_new() {
    let effect = Effect::new(
      EffectType::StateChange,
      "Test description".to_string(),
      EffectSeverity::High,
      "Test suggestion".to_string(),
      "test_behavior".to_string(),
    );

    assert_eq!(effect.effect_type, EffectType::StateChange);
    assert_eq!(effect.description, "Test description");
    assert_eq!(effect.severity, EffectSeverity::High);
    assert_eq!(effect.suggestion, "Test suggestion");
    assert_eq!(effect.source_behavior, "test_behavior");
  }

  // -------------------------------------------------------------------------
  // EffectsSummary tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_effects_summary_default() {
    let summary = EffectsSummary::new();
    assert_eq!(summary.state_changes, 0);
    assert_eq!(summary.total, 0);
    assert!(summary.max_severity.is_none());
  }

  #[test]
  fn test_effects_summary_from_effects() {
    let effects = vec![
      Effect::new(
        EffectType::StateChange,
        "desc1".to_string(),
        EffectSeverity::Medium,
        "sug1".to_string(),
        "b1".to_string(),
      ),
      Effect::new(
        EffectType::StateChange,
        "desc2".to_string(),
        EffectSeverity::High,
        "sug2".to_string(),
        "b2".to_string(),
      ),
      Effect::new(
        EffectType::Notification,
        "desc3".to_string(),
        EffectSeverity::Low,
        "sug3".to_string(),
        "b3".to_string(),
      ),
    ];

    let summary = EffectsSummary::from_effects(&effects);
    assert_eq!(summary.state_changes, 2);
    assert_eq!(summary.notifications, 1);
    assert_eq!(summary.total, 3);
    assert_eq!(summary.max_severity, Some(EffectSeverity::High));
  }

  // -------------------------------------------------------------------------
  // EffectsReport tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_effects_report_new() {
    let effects = vec![Effect::new(
      EffectType::StateChange,
      "desc".to_string(),
      EffectSeverity::Medium,
      "sug".to_string(),
      "test".to_string(),
    )];

    let report = EffectsReport::new("test_behavior".to_string(), effects);
    assert_eq!(report.behavior_name, "test_behavior");
    assert_eq!(report.effects.len(), 1);
    assert_eq!(report.summary.total, 1);
    assert!(report.summary_text.contains("test_behavior"));
  }

  #[test]
  fn test_effects_report_empty() {
    let report = EffectsReport::new("empty".to_string(), Vec::new());
    assert!(report.effects.is_empty());
    assert_eq!(report.summary.total, 0);
    assert!(report.summary_text.contains("No second-order effects"));
  }

  #[test]
  fn test_effects_report_has_critical() {
    let effects = vec![Effect::new(
      EffectType::StateChange,
      "desc".to_string(),
      EffectSeverity::Critical,
      "sug".to_string(),
      "test".to_string(),
    )];

    let report = EffectsReport::new("test".to_string(), effects);
    assert!(report.has_critical_effects());
  }

  #[test]
  fn test_effects_report_no_critical() {
    let effects = vec![Effect::new(
      EffectType::StateChange,
      "desc".to_string(),
      EffectSeverity::Low,
      "sug".to_string(),
      "test".to_string(),
    )];

    let report = EffectsReport::new("test".to_string(), effects);
    assert!(!report.has_critical_effects());
  }

  #[test]
  fn test_effects_report_filter_by_type() {
    let effects = vec![
      Effect::new(
        EffectType::StateChange,
        "desc1".to_string(),
        EffectSeverity::Low,
        "sug1".to_string(),
        "test".to_string(),
      ),
      Effect::new(
        EffectType::Notification,
        "desc2".to_string(),
        EffectSeverity::Low,
        "sug2".to_string(),
        "test".to_string(),
      ),
    ];

    let report = EffectsReport::new("test".to_string(), effects);
    let state_changes = report.effects_by_type(EffectType::StateChange);
    assert_eq!(state_changes.len(), 1);
  }

  // -------------------------------------------------------------------------
  // analyze_behavior tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_analyze_behavior_state_change_create() {
    let behavior = Behavior::new("create_user".to_string())
      .expect("valid behavior")
      .with_description("Create a new user in the database".to_string());

    let effects = analyze_behavior(&behavior);
    assert!(!effects.is_empty());

    let has_state_change = effects
      .iter()
      .any(|e| e.effect_type == EffectType::StateChange);
    assert!(has_state_change);
  }

  #[test]
  fn test_analyze_behavior_state_change_delete() {
    let behavior = Behavior::new("delete_account".to_string())
      .expect("valid behavior")
      .with_description("Permanently destroy user account".to_string());

    let effects = analyze_behavior(&behavior);

    let state_effect = effects
      .iter()
      .find(|e| e.effect_type == EffectType::StateChange);
    assert!(state_effect.is_some());

    // Delete should be high severity
    let state_effect = state_effect.expect("found state effect");
    assert!(state_effect.severity >= EffectSeverity::High);
  }

  #[test]
  fn test_analyze_behavior_notification() {
    let behavior = Behavior::new("register".to_string())
      .expect("valid behavior")
      .with_description("Register user and send welcome email notification".to_string());

    let effects = analyze_behavior(&behavior);

    let has_notification = effects
      .iter()
      .any(|e| e.effect_type == EffectType::Notification);
    assert!(has_notification);
  }

  #[test]
  fn test_analyze_behavior_cascade() {
    let behavior = Behavior::new("remove_user".to_string())
      .expect("valid behavior")
      .with_description("Remove user and cascade to all related records".to_string());

    let effects = analyze_behavior(&behavior);

    let has_cascade = effects.iter().any(|e| e.effect_type == EffectType::Cascade);
    assert!(has_cascade);
  }

  #[test]
  fn test_analyze_behavior_race_condition() {
    let behavior = Behavior::new("update_balance".to_string())
      .expect("valid behavior")
      .with_description("Update account balance with concurrent access protection".to_string());

    let effects = analyze_behavior(&behavior);

    let has_race = effects
      .iter()
      .any(|e| e.effect_type == EffectType::RaceCondition);
    assert!(has_race);
  }

  #[test]
  fn test_analyze_behavior_rollback() {
    let behavior = Behavior::new("transfer_funds".to_string())
      .expect("valid behavior")
      .with_description("Transfer funds within a transaction with rollback support".to_string());

    let effects = analyze_behavior(&behavior);

    let has_rollback = effects
      .iter()
      .any(|e| e.effect_type == EffectType::RollbackRequired);
    assert!(has_rollback);
  }

  #[test]
  fn test_analyze_behavior_multiple_effects() {
    let behavior = Behavior::new("create_order".to_string())
      .expect("valid behavior")
      .with_description(
        "Create order, update inventory, and send webhook notification".to_string(),
      );

    let effects = analyze_behavior(&behavior);

    let types: std::collections::HashSet<EffectType> =
      effects.iter().map(|e| e.effect_type).collect();

    assert!(types.contains(&EffectType::StateChange));
    assert!(types.contains(&EffectType::Notification));
  }

  #[test]
  fn test_analyze_behavior_no_effects() {
    let behavior = Behavior::new("view_profile".to_string())
      .expect("valid behavior")
      .with_description("Display user profile information".to_string());

    let effects = analyze_behavior(&behavior);
    assert!(effects.is_empty());
  }

  #[test]
  fn test_analyze_behavior_critical_severity() {
    let behavior = Behavior::new("emergency_purge".to_string())
      .expect("valid behavior")
      .with_description("Emergency unrecoverable wipe of all user data".to_string());

    let effects = analyze_behavior(&behavior);

    let has_critical = effects
      .iter()
      .any(|e| e.severity == EffectSeverity::Critical);
    assert!(has_critical);
  }

  // -------------------------------------------------------------------------
  // analyze_behavior_report tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_analyze_behavior_report() {
    let behavior = Behavior::new("create_user".to_string())
      .expect("valid behavior")
      .with_description("Create a new user".to_string());

    let report = analyze_behavior_report(&behavior);
    assert_eq!(report.behavior_name, "create_user");
    assert!(!report.effects.is_empty());
  }

  // -------------------------------------------------------------------------
  // analyze_feature tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_analyze_feature() {
    let mut feature = Feature::new("user_mgmt".to_string()).expect("valid feature");

    let b1 = Behavior::new("create_user".to_string())
      .expect("valid")
      .with_description("Create user".to_string());
    let b2 = Behavior::new("delete_user".to_string())
      .expect("valid")
      .with_description("Delete user".to_string());

    feature.add_behavior(b1).expect("should add");
    feature.add_behavior(b2).expect("should add");

    let reports = analyze_feature(&feature);
    assert_eq!(reports.len(), 2);
  }

  #[test]
  fn test_analyze_feature_empty() {
    let feature = Feature::new("empty".to_string()).expect("valid feature");
    let reports = analyze_feature(&feature);
    assert!(reports.is_empty());
  }

  // -------------------------------------------------------------------------
  // analyze_spec tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_analyze_spec() {
    let mut spec = Spec::new("test_spec".to_string()).expect("valid spec");

    let mut feature = Feature::new("auth".to_string()).expect("valid feature");
    let behavior = Behavior::new("login".to_string())
      .expect("valid")
      .with_description("Authenticate user and update last login".to_string());

    feature.add_behavior(behavior).expect("should add");
    spec.add_feature(feature).expect("should add");

    let report = analyze_spec(&spec);

    assert_eq!(report.spec_name, "test_spec");
    assert_eq!(report.behavior_reports.len(), 1);
    assert!(report.total_summary.total > 0);
  }

  #[test]
  fn test_analyze_spec_empty() {
    let spec = Spec::new("empty".to_string()).expect("valid spec");
    let report = analyze_spec(&spec);

    assert!(report.behavior_reports.is_empty());
    assert_eq!(report.total_summary.total, 0);
  }

  #[test]
  fn test_analyze_spec_multiple_features() {
    let mut spec = Spec::new("multi".to_string()).expect("valid spec");

    let mut f1 = Feature::new("users".to_string()).expect("valid");
    f1.add_behavior(
      Behavior::new("create".to_string())
        .expect("valid")
        .with_description("Create user".to_string()),
    )
    .expect("should add");

    let mut f2 = Feature::new("orders".to_string()).expect("valid");
    f2.add_behavior(
      Behavior::new("submit".to_string())
        .expect("valid")
        .with_description("Submit order and send email".to_string()),
    )
    .expect("should add");

    spec.add_feature(f1).expect("should add");
    spec.add_feature(f2).expect("should add");

    let report = analyze_spec(&spec);
    assert_eq!(report.behavior_reports.len(), 2);
  }

  // -------------------------------------------------------------------------
  // Utility function tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_has_critical_effects_true() {
    let behavior = Behavior::new("emergency_delete".to_string())
      .expect("valid")
      .with_description("Emergency unrecoverable destruction of data".to_string());

    assert!(has_critical_effects(&behavior));
  }

  #[test]
  fn test_has_critical_effects_false() {
    let behavior = Behavior::new("view".to_string())
      .expect("valid")
      .with_description("View data".to_string());

    assert!(!has_critical_effects(&behavior));
  }

  #[test]
  fn test_has_high_severity_effects_true() {
    let behavior = Behavior::new("delete".to_string())
      .expect("valid")
      .with_description("Delete user permanently".to_string());

    assert!(has_high_severity_effects(&behavior));
  }

  #[test]
  fn test_max_effect_severity_some() {
    let behavior = Behavior::new("delete".to_string())
      .expect("valid")
      .with_description("Critical delete operation".to_string());

    let severity = max_effect_severity(&behavior);
    assert!(severity.is_some());
  }

  #[test]
  fn test_max_effect_severity_none() {
    let behavior = Behavior::new("read".to_string())
      .expect("valid")
      .with_description("Read only".to_string());

    let severity = max_effect_severity(&behavior);
    assert!(severity.is_none());
  }

  #[test]
  fn test_count_effects_by_type() {
    let mut spec = Spec::new("test".to_string()).expect("valid");

    let mut feature = Feature::new("f".to_string()).expect("valid");
    feature
      .add_behavior(
        Behavior::new("create".to_string())
          .expect("valid")
          .with_description("Create and send email".to_string()),
      )
      .expect("should add");

    spec.add_feature(feature).expect("should add");

    let counts = count_effects_by_type(&spec);
    assert!(counts.contains_key(&EffectType::StateChange));
  }

  #[test]
  fn test_behaviors_with_effect_type() {
    let mut spec = Spec::new("test".to_string()).expect("valid");

    let mut feature = Feature::new("f".to_string()).expect("valid");
    feature
      .add_behavior(
        Behavior::new("create".to_string())
          .expect("valid")
          .with_description("Create user".to_string()),
      )
      .expect("should add");
    feature
      .add_behavior(
        Behavior::new("view".to_string())
          .expect("valid")
          .with_description("View user".to_string()),
      )
      .expect("should add");

    spec.add_feature(feature).expect("should add");

    let state_change_behaviors = behaviors_with_effect_type(&spec, EffectType::StateChange);
    assert!(state_change_behaviors.contains(&"create".to_string()));
    assert!(!state_change_behaviors.contains(&"view".to_string()));
  }

  // -------------------------------------------------------------------------
  // SpecEffectsReport tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_spec_effects_report_critical_effects() {
    let mut spec = Spec::new("test".to_string()).expect("valid");

    let mut feature = Feature::new("f".to_string()).expect("valid");
    feature
      .add_behavior(
        Behavior::new("emergency".to_string())
          .expect("valid")
          .with_description("Emergency unrecoverable wipe".to_string()),
      )
      .expect("should add");

    spec.add_feature(feature).expect("should add");

    let report = analyze_spec(&spec);
    let critical = report.critical_effects();
    assert!(!critical.is_empty());
  }

  #[test]
  fn test_spec_effects_report_high_severity_effects() {
    let mut spec = Spec::new("test".to_string()).expect("valid");

    let mut feature = Feature::new("f".to_string()).expect("valid");
    feature
      .add_behavior(
        Behavior::new("delete".to_string())
          .expect("valid")
          .with_description("Delete user permanently".to_string()),
      )
      .expect("should add");

    spec.add_feature(feature).expect("should add");

    let report = analyze_spec(&spec);
    let high = report.high_severity_effects();
    assert!(!high.is_empty());
  }

  // -------------------------------------------------------------------------
  // EffectType Display tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_effect_type_display() {
    assert_eq!(format!("{}", EffectType::StateChange), "State Change");
    assert_eq!(format!("{}", EffectType::Notification), "Notification");
    assert_eq!(format!("{}", EffectType::Cascade), "Cascade");
    assert_eq!(format!("{}", EffectType::RaceCondition), "Race Condition");
    assert_eq!(
      format!("{}", EffectType::RollbackRequired),
      "Rollback Required"
    );
  }

  // -------------------------------------------------------------------------
  // EffectSeverity Display tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_effect_severity_display() {
    assert_eq!(format!("{}", EffectSeverity::Low), "Low");
    assert_eq!(format!("{}", EffectSeverity::Medium), "Medium");
    assert_eq!(format!("{}", EffectSeverity::High), "High");
    assert_eq!(format!("{}", EffectSeverity::Critical), "Critical");
  }

  // -------------------------------------------------------------------------
  // Serde tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_effect_serde_roundtrip() {
    let effect = Effect::new(
      EffectType::StateChange,
      "Test effect".to_string(),
      EffectSeverity::High,
      "Test suggestion".to_string(),
      "test_behavior".to_string(),
    );

    let json = serde_json::to_string(&effect).expect("should serialize");
    let parsed: Effect = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(effect, parsed);
  }

  #[test]
  fn test_effects_report_serde_roundtrip() {
    let effects = vec![Effect::new(
      EffectType::Notification,
      "Test".to_string(),
      EffectSeverity::Low,
      "Suggestion".to_string(),
      "test".to_string(),
    )];

    let report = EffectsReport::new("test".to_string(), effects);

    let json = serde_json::to_string(&report).expect("should serialize");
    let parsed: EffectsReport = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(report, parsed);
  }

  // -------------------------------------------------------------------------
  // Suggestion generation tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_suggestion_state_change_delete() {
    let behavior = Behavior::new("delete_user".to_string())
      .expect("valid")
      .with_description("Delete user permanently".to_string());

    let effects = analyze_behavior(&behavior);
    let state_effect = effects
      .iter()
      .find(|e| e.effect_type == EffectType::StateChange)
      .expect("should have state effect");

    assert!(state_effect.suggestion.contains("soft delete"));
  }

  #[test]
  fn test_suggestion_race_condition() {
    let behavior = Behavior::new("concurrent_update".to_string())
      .expect("valid")
      .with_description("Update with concurrent access".to_string());

    let effects = analyze_behavior(&behavior);
    let race_effect = effects
      .iter()
      .find(|e| e.effect_type == EffectType::RaceCondition)
      .expect("should have race effect");

    assert!(race_effect.suggestion.contains("locking"));
  }

  #[test]
  fn test_suggestion_notification() {
    let behavior = Behavior::new("notify".to_string())
      .expect("valid")
      .with_description("Send notification to user".to_string());

    let effects = analyze_behavior(&behavior);
    let notif_effect = effects
      .iter()
      .find(|e| e.effect_type == EffectType::Notification)
      .expect("should have notification effect");

    assert!(notif_effect.suggestion.contains("retry"));
  }

  // -------------------------------------------------------------------------
  // Pre/Post conditions analysis tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_analyze_behavior_with_preconditions() {
    let mut behavior = Behavior::new("transfer".to_string())
      .expect("valid")
      .with_description("Transfer funds".to_string());
    behavior.add_precondition("User must be authenticated".to_string());
    behavior.add_precondition("Concurrent transfers must be prevented".to_string());

    let effects = analyze_behavior(&behavior);
    let has_race = effects
      .iter()
      .any(|e| e.effect_type == EffectType::RaceCondition);
    assert!(has_race);
  }

  #[test]
  fn test_analyze_behavior_with_postconditions() {
    let mut behavior = Behavior::new("create".to_string())
      .expect("valid")
      .with_description("Create record".to_string());
    behavior.add_postcondition("Send webhook notification".to_string());
    behavior.add_postcondition("Cascade to related records".to_string());

    let effects = analyze_behavior(&behavior);

    let has_notification = effects
      .iter()
      .any(|e| e.effect_type == EffectType::Notification);
    let has_cascade = effects.iter().any(|e| e.effect_type == EffectType::Cascade);

    assert!(has_notification);
    assert!(has_cascade);
  }

  // -------------------------------------------------------------------------
  // Edge cases
  // -------------------------------------------------------------------------

  #[test]
  fn test_analyze_behavior_empty_description() {
    let behavior = Behavior::new("empty".to_string()).expect("valid behavior");

    let effects = analyze_behavior(&behavior);
    // Empty description should not produce effects
    assert!(effects.is_empty());
  }

  #[test]
  fn test_analyze_behavior_only_name_keywords() {
    // Behavior name contains keywords but description is empty
    let behavior = Behavior::new("create_user".to_string()).expect("valid behavior");

    let effects = analyze_behavior(&behavior);
    // Name contains "create" which is a state change keyword
    let has_state_change = effects
      .iter()
      .any(|e| e.effect_type == EffectType::StateChange);
    assert!(has_state_change);
  }
}
