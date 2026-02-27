use crate::intent::types::Behavior;

use super::patterns::{CRITICAL_SEVERITY_KEYWORDS, HIGH_SEVERITY_KEYWORDS};
use super::types::{Effect, EffectSeverity, EffectType};

pub(super) fn combined_behavior_text(behavior: &Behavior) -> String {
  format!(
    "{} {} {} {}",
    behavior.name,
    behavior.description,
    behavior.preconditions.join(" "),
    behavior.postconditions.join(" ")
  )
}

pub(super) fn contains_keywords(text: &str, keywords: &[&str]) -> bool {
  let lower = text.to_lowercase();
  keywords.iter().any(|keyword| lower.contains(keyword))
}

pub(super) fn matched_keywords<'a>(text: &str, keywords: &'a [&'a str]) -> Vec<&'a str> {
  let lower = text.to_lowercase();
  keywords
    .iter()
    .filter(|keyword| lower.contains(*keyword))
    .map(|keyword| *keyword)
    .collect()
}

pub(super) fn estimate_severity(text: &str, effect_type: EffectType) -> EffectSeverity {
  if contains_keywords(text, CRITICAL_SEVERITY_KEYWORDS) {
    EffectSeverity::Critical
  } else if contains_keywords(text, HIGH_SEVERITY_KEYWORDS) {
    EffectSeverity::High
  } else {
    match effect_type {
      EffectType::StateChange => {
        let lower = text.to_lowercase();
        if ["delete", "destroy", "purge"]
          .iter()
          .any(|kw| lower.contains(kw))
        {
          EffectSeverity::High
        } else if ["create", "insert"].iter().any(|kw| lower.contains(kw)) {
          EffectSeverity::Medium
        } else {
          EffectSeverity::Low
        }
      }
      EffectType::Notification => EffectSeverity::Low,
      EffectType::Cascade => EffectSeverity::Medium,
      EffectType::RaceCondition => EffectSeverity::High,
      EffectType::RollbackRequired => EffectSeverity::Medium,
    }
  }
}

pub(super) fn suggestion_for(
  effect_type: EffectType,
  text: &str,
  severity: EffectSeverity,
) -> String {
  match effect_type {
        EffectType::StateChange => {
            let lower = text.to_lowercase();
            if ["delete", "destroy"].iter().any(|kw| lower.contains(kw)) {
                "Consider implementing soft delete or archive pattern. Ensure proper authorization and audit logging.".to_string()
            } else if ["update", "modify"].iter().any(|kw| lower.contains(kw)) {
                "Consider implementing optimistic locking or versioning. Validate all inputs before applying changes.".to_string()
            } else {
                "Validate inputs, ensure atomicity, and consider idempotency for retries.".to_string()
            }
        }
        EffectType::Notification => "Consider implementing retry logic, dead letter queues, and rate limiting. Ensure notifications are idempotent.".to_string(),
        EffectType::Cascade => "Document all cascade effects. Consider implementing saga pattern for distributed operations.".to_string(),
        EffectType::RaceCondition => {
            if severity == EffectSeverity::Critical || severity == EffectSeverity::High {
                "Implement proper locking mechanisms (pessimistic or optimistic). Consider using database transactions with appropriate isolation levels.".to_string()
            } else {
                "Consider using atomic operations or compare-and-swap patterns.".to_string()
            }
        }
        EffectType::RollbackRequired => "Implement compensation actions for rollback scenarios. Consider using saga pattern for distributed transactions.".to_string(),
    }
}

pub(super) fn create_effect(
  effect_type: EffectType,
  behavior_name: &str,
  description: String,
  context_text: &str,
) -> Effect {
  let severity = estimate_severity(context_text, effect_type);
  let suggestion = suggestion_for(effect_type, context_text, severity);
  Effect::new(
    effect_type,
    format!("{effect_type} detected in '{behavior_name}': {description}"),
    severity,
    suggestion,
    behavior_name.to_string(),
  )
}

pub(super) fn detect_by_keywords(
  behavior: &Behavior,
  effect_type: EffectType,
  keywords: &[&str],
  message_prefix: &str,
) -> Vec<Effect> {
  let text = combined_behavior_text(behavior);
  if !contains_keywords(&text, keywords) {
    Vec::new()
  } else {
    let description = format!(
      "{message_prefix}: {}",
      matched_keywords(&text, keywords).join(", ")
    );
    vec![create_effect(
      effect_type,
      &behavior.name,
      description,
      &text,
    )]
  }
}
