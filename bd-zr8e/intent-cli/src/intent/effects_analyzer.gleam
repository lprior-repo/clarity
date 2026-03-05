import gleam/list
import gleam/string
import intent/types.{type Behavior, type Spec}
import intent/case_insensitive.{contains_any_ignore_case}

/// Effect types for second-order analysis
pub type EffectType {
  StateChange
  Notification
  Cascade
  RaceCondition
  RollbackRequired
}

/// Individual effect finding
pub type Effect {
  Effect(
    type_: EffectType,
    description: String,
    severity: Severity,
    suggestion: String,
  )
}

/// Severity levels
pub type Severity {
  High
  Medium
  Low
}

/// Analysis result for a single behavior
pub type BehaviorEffect {
  BehaviorEffect(behavior_name: String, effects: List(Effect))
}

/// Analysis result for entire spec
pub type SpecAnalysis {
  SpecAnalysis(spec_name: String, behavior_effects: List(BehaviorEffect))
}

/// Analyze a single behavior for second-order effects
pub fn analyze_behavior(behavior: Behavior) -> List(Effect) {
  let effects = []

  // Analyze based on behavior intent and keywords
  let intent_effects = analyze_behavior_intent(behavior)
  let effects = list.append(effects, intent_effects)

  // Check for cascade effects
  let cascade_effects = analyze_cascade_effects(behavior)
  let effects = list.append(effects, cascade_effects)

  // Check for race conditions
  let race_effects = analyze_race_conditions(behavior)
  let effects = list.append(effects, race_effects)

  // Check for rollback requirements
  let rollback_effects = analyze_rollback_needs(behavior)
  let effects = list.append(effects, rollback_effects)

  effects
}

/// Analyze behavior intent for implications
fn analyze_behavior_intent(behavior: Behavior) -> List(Effect) {
  // Check for state-changing operations
  let is_create =
    contains_any_ignore_case(behavior.name, ["create", "add", "new", "insert"])
    || contains_any_ignore_case(behavior.intent, ["create", "add", "insert"])

  let is_update =
    contains_any_ignore_case(behavior.name, ["update", "modify", "change", "edit"])
    || contains_any_ignore_case(behavior.intent, ["update", "modify", "change"])

  let is_delete =
    contains_any_ignore_case(behavior.name, ["delete", "remove", "destroy"])
    || contains_any_ignore_case(behavior.intent, ["delete", "remove", "destroy"])

  let is_read =
    contains_any_ignore_case(behavior.name, ["get", "fetch", "list", "show", "find"])
    || contains_any_ignore_case(behavior.intent, ["get", "fetch", "list", "find"])

  case True {
    _ if is_delete -> [
      Effect(
        type_: StateChange,
        description: "Removes resource from data store",
        severity: High,
        suggestion: "Add behavior to test orphaned data cleanup",
      ),
      Effect(
        type_: RollbackRequired,
        description: "Deletion should be reversible or soft-delete",
        severity: High,
        suggestion: "Add behavior to test soft-delete or restoration",
      ),
    ]
    _ if is_create || is_update -> [
      Effect(
        type_: StateChange,
        description: "Creates or modifies resource in data store",
        severity: Medium,
        suggestion: "Add behavior to test duplicate creation/updates",
      ),
      Effect(
        type_: Notification,
        description: "May trigger notification events",
        severity: Low,
        suggestion: "Add behavior to test notification failure handling",
      ),
    ]
    _ if is_read -> [
      Effect(
        type_: Cascade,
        description: "Read operations may trigger cache updates",
        severity: Low,
        suggestion: "Add behavior to test cache consistency",
      ),
    ]
    _ -> []
  }
}

/// Analyze potential cascade effects
fn analyze_cascade_effects(behavior: Behavior) -> List(Effect) {
  let is_state_change =
    contains_any_ignore_case(behavior.name, ["create", "update", "delete", "modify"])
    || contains_any_ignore_case(behavior.intent, ["create", "update", "delete"])

  case is_state_change {
    True -> [
      Effect(
        type_: Cascade,
        description: "Operation may affect related records or data",
        severity: High,
        suggestion: "Add behavior to test referential integrity and data consistency",
      ),
    ]
    False -> []
  }
}

/// Analyze potential race conditions
fn analyze_race_conditions(behavior: Behavior) -> List(Effect) {
  let is_concurrent =
    contains_any_ignore_case(behavior.name, ["create", "update", "delete", "modify"])

  case is_concurrent {
    True -> [
      Effect(
        type_: RaceCondition,
        description: "Concurrent modifications may conflict",
        severity: Medium,
        suggestion: "Add behavior to test optimistic locking or conflict resolution",
      ),
    ]
    False -> []
  }
}

/// Analyze rollback requirements
fn analyze_rollback_needs(behavior: Behavior) -> List(Effect) {
  let is_state_change =
    contains_any_ignore_case(behavior.name, ["create", "update", "delete", "modify"])

  case is_state_change {
    True -> [
      Effect(
        type_: RollbackRequired,
        description: "Operation should be reversible or compensatable",
        severity: Medium,
        suggestion: "Add compensating transaction behavior",
      ),
    ]
    False -> []
  }
}

/// Analyze entire spec
pub fn analyze_spec(spec: Spec) -> SpecAnalysis {
  // Collect all behaviors from all features
  let all_behaviors =
    list.flat_map(spec.features, fn(feature) { feature.behaviors })

  // Analyze each behavior
  let behavior_effects =
    list.map(all_behaviors, fn(behavior) {
      BehaviorEffect(
        behavior_name: behavior.name,
        effects: analyze_behavior(behavior),
      )
    })

  SpecAnalysis(spec_name: spec.name, behavior_effects: behavior_effects)
}

/// Format effects as JSON
pub fn format_effects_json(effects: List(Effect)) -> Result(String, String) {
  let json_objects =
    list.map(effects, fn(effect) {
      let type_str = effect_type_to_string(effect.type_)
      let severity_str = severity_to_string(effect.severity)

      [
        #("type", type_str),
        #("description", effect.description),
        #("severity", severity_str),
        #("suggestion", effect.suggestion),
      ]
    })

  // Convert to JSON string manually
  Ok("[" <> string.join(
    list.map(json_objects, fn(obj) {
      "{"
      <> string.join(
        list.map(obj, fn(pair) { "\"" <> pair.0 <> "\": \"" <> pair.1 <> "\"" }),
        ", ",
      )
      <> "}"
    }),
    ", ",
  ) <> "]")
}

/// Format effects for CLI display
pub fn format_effects_cli(
  behavior_name: String,
  effects: List(Effect),
) -> String {
  let header = "Analyzing: " <> behavior_name <> "\n\nSecond-Order Effects:\n"

  let effect_lines =
    list.map(effects, fn(effect) {
      let icon = effect_type_to_icon(effect.type_)
      let severity = severity_to_string(effect.severity)

      "  "
      <> icon
      <> " "
      <> effect_type_to_string(effect.type_)
      <> ": "
      <> effect.description
      <> "\n"
      <> "     Severity: "
      <> severity
      <> "\n"
      <> "     → "
      <> effect.suggestion
      <> "\n"
    })

  string.join([header, ..effect_lines], "")
}

/// Convert effect type to string
fn effect_type_to_string(type_: EffectType) -> String {
  case type_ {
    StateChange -> "state_change"
    Notification -> "notification"
    Cascade -> "cascade"
    RaceCondition -> "race_condition"
    RollbackRequired -> "rollback_required"
  }
}

/// Convert effect type to display icon
fn effect_type_to_icon(type_: EffectType) -> String {
  case type_ {
    StateChange -> "📝"
    Notification -> "📧"
    Cascade -> "🔗"
    RaceCondition -> "⚠️"
    RollbackRequired -> "🔄"
  }
}

/// Convert severity to string
fn severity_to_string(severity: Severity) -> String {
  case severity {
    High -> "high"
    Medium -> "medium"
    Low -> "low"
  }
}
