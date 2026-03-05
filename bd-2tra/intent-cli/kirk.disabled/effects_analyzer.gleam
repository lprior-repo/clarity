/// KIRK Effects Analyzer
///
/// Second-Order Thinking: "What happens after the immediate effect?"
/// Traces consequences beyond first-order results.
import gleam/float
import gleam/int
import gleam/list
import gleam/result
import gleam/string
import intent/types.{
  type Behavior, type Spec, Delete, Get, Head, Options, Patch, Post, Put,
}

// =============================================================================
// TYPES
// =============================================================================

pub type EffectsReport {
  EffectsReport(
    behavior_effects: List(BehaviorEffects),
    orphaned_resources: List(OrphanedResource),
    cascade_warnings: List(CascadeWarning),
    state_dependencies: List(StateDependency),
    total_second_order_effects: Int,
    coverage_score: Float,
  )
}

pub type BehaviorEffects {
  BehaviorEffects(
    behavior_name: String,
    first_order: String,
    second_order: List(SecondOrderEffect),
    missing_verifications: List(String),
  )
}

pub type SecondOrderEffect {
  SecondOrderEffect(
    description: String,
    severity: EffectSeverity,
    category: EffectCategory,
    has_verification: Bool,
  )
}

pub type EffectSeverity {
  Info
  Warning
  Danger
  Critical
}

pub type EffectCategory {
  ResourceLifecycle
  DataIntegrity
  SystemState
  SecurityImplication
  PerformanceImpact
  ExternalDependency
}

pub type OrphanedResource {
  OrphanedResource(
    resource_type: String,
    caused_by: String,
    description: String,
    mitigation: String,
  )
}

pub type CascadeWarning {
  CascadeWarning(
    operation: String,
    cascades_to: List(String),
    requires_transaction: Bool,
    description: String,
  )
}

pub type StateDependency {
  StateDependency(
    behavior: String,
    depends_on: List(String),
    state_mutations: List(String),
    isolation_level: String,
  )
}

// =============================================================================
// MAIN ANALYSIS
// =============================================================================

pub fn analyze_effects(spec: Spec) -> EffectsReport {
  let behaviors = get_all_behaviors(spec)

  // Analyze each behavior for second-order effects
  let behavior_effects = list.map(behaviors, analyze_behavior_effects)

  // Detect orphaned resources from delete operations
  let orphaned = detect_orphaned_resources(behaviors)

  // Find cascading operations that affect multiple resources
  let cascades = detect_cascade_operations(behaviors)

  // Map state dependencies between behaviors
  let state_deps = analyze_state_dependencies(behaviors)

  // Calculate coverage score
  let total_effects =
    behavior_effects
    |> list.map(fn(be) { list.length(be.second_order) })
    |> list.fold(0, fn(sum, count) { sum + count })

  let verified_effects =
    behavior_effects
    |> list.flat_map(fn(be) { be.second_order })
    |> list.filter(fn(eff) { eff.has_verification })
    |> list.length()

  let coverage_score = case total_effects {
    0 -> 100.0
    _ -> int.to_float(verified_effects) /. int.to_float(total_effects) *. 100.0
  }

  EffectsReport(
    behavior_effects: behavior_effects,
    orphaned_resources: orphaned,
    cascade_warnings: cascades,
    state_dependencies: state_deps,
    total_second_order_effects: total_effects,
    coverage_score: coverage_score,
  )
}

// =============================================================================
// BEHAVIOR ANALYSIS
// =============================================================================

fn analyze_behavior_effects(behavior: Behavior) -> BehaviorEffects {
  let first_order = describe_first_order_effect(behavior)
  let second_order = infer_second_order_effects(behavior)

  // Check if second-order effects have verification behaviors
  let behavior_names_lower = behavior.requires |> list.map(string.lowercase)

  let effects_with_verification =
    second_order
    |> list.map(fn(effect) {
      let has_verification =
        check_has_verification(effect, behavior_names_lower)
      SecondOrderEffect(..effect, has_verification: has_verification)
    })

  let missing_verifications =
    effects_with_verification
    |> list.filter(fn(eff) { !eff.has_verification })
    |> list.map(fn(eff) { eff.description })

  BehaviorEffects(
    behavior_name: behavior.name,
    first_order: first_order,
    second_order: effects_with_verification,
    missing_verifications: missing_verifications,
  )
}

fn describe_first_order_effect(behavior: Behavior) -> String {
  case behavior.request.method {
    Post ->
      "Resource is created (status "
      <> int.to_string(behavior.response.status)
      <> ")"
    Get | Head ->
      "Resource is retrieved (status "
      <> int.to_string(behavior.response.status)
      <> ")"
    Put | Patch ->
      "Resource is updated (status "
      <> int.to_string(behavior.response.status)
      <> ")"
    Delete ->
      "Resource is deleted (status "
      <> int.to_string(behavior.response.status)
      <> ")"
    Options ->
      "Resource options are retrieved (status "
      <> int.to_string(behavior.response.status)
      <> ")"
  }
}

fn infer_second_order_effects(behavior: Behavior) -> List(SecondOrderEffect) {
  let method_effects = case behavior.request.method {
    Delete -> infer_delete_effects(behavior)
    Post -> infer_create_effects(behavior)
    Put | Patch -> infer_update_effects(behavior)
    Get | Head | Options -> infer_read_effects(behavior)
  }

  let name_based = infer_from_behavior_name(behavior)

  list.concat([method_effects, name_based])
}

fn infer_delete_effects(behavior: Behavior) -> List(SecondOrderEffect) {
  let base_effects = [
    SecondOrderEffect(
      description: "Related resources may become orphaned",
      severity: Warning,
      category: ResourceLifecycle,
      has_verification: False,
    ),
    SecondOrderEffect(
      description: "References to this resource must be handled",
      severity: Warning,
      category: DataIntegrity,
      has_verification: False,
    ),
    SecondOrderEffect(
      description: "Audit log entries remain but point to deleted resource",
      severity: Info,
      category: DataIntegrity,
      has_verification: False,
    ),
  ]

  // Check for user/account deletion patterns
  let is_user_delete =
    string.contains(string.lowercase(behavior.name), "user")
    || string.contains(string.lowercase(behavior.name), "account")
    || string.contains(string.lowercase(behavior.request.path), "user")

  let user_specific = case is_user_delete {
    True -> [
      SecondOrderEffect(
        description: "Active sessions must be invalidated",
        severity: Critical,
        category: SecurityImplication,
        has_verification: False,
      ),
      SecondOrderEffect(
        description: "User's owned resources need ownership transfer",
        severity: Danger,
        category: ResourceLifecycle,
        has_verification: False,
      ),
      SecondOrderEffect(
        description: "Analytics data loses user attribution",
        severity: Info,
        category: DataIntegrity,
        has_verification: False,
      ),
    ]
    False -> []
  }

  list.concat([base_effects, user_specific])
}

fn infer_create_effects(_behavior: Behavior) -> List(SecondOrderEffect) {
  [
    SecondOrderEffect(
      description: "Resource can now be retrieved via GET",
      severity: Info,
      category: ResourceLifecycle,
      has_verification: False,
    ),
    SecondOrderEffect(
      description: "Resource appears in listing endpoints",
      severity: Info,
      category: SystemState,
      has_verification: False,
    ),
    SecondOrderEffect(
      description: "Resource count/statistics are updated",
      severity: Info,
      category: SystemState,
      has_verification: False,
    ),
  ]
}

fn infer_update_effects(_behavior: Behavior) -> List(SecondOrderEffect) {
  [
    SecondOrderEffect(
      description: "Updated values are reflected in subsequent reads",
      severity: Warning,
      category: DataIntegrity,
      has_verification: False,
    ),
    SecondOrderEffect(
      description: "Cache entries may need invalidation",
      severity: Warning,
      category: PerformanceImpact,
      has_verification: False,
    ),
    SecondOrderEffect(
      description: "Dependent resources may be affected",
      severity: Info,
      category: ResourceLifecycle,
      has_verification: False,
    ),
  ]
}

fn infer_read_effects(_behavior: Behavior) -> List(SecondOrderEffect) {
  // GET operations typically have minimal second-order effects
  // but may have caching or rate-limiting implications
  [
    SecondOrderEffect(
      description: "Response may be cached",
      severity: Info,
      category: PerformanceImpact,
      has_verification: False,
    ),
  ]
}

fn infer_from_behavior_name(behavior: Behavior) -> List(SecondOrderEffect) {
  let name_lower = string.lowercase(behavior.name)
  let intent_lower = string.lowercase(behavior.intent)
  let combined = name_lower <> " " <> intent_lower

  let effects = []

  // Async/background operations
  let effects = case
    string.contains(combined, "async")
    || string.contains(combined, "background")
    || string.contains(combined, "queue")
  {
    True ->
      list.append(effects, [
        SecondOrderEffect(
          description: "Background job is queued for processing",
          severity: Warning,
          category: ExternalDependency,
          has_verification: False,
        ),
      ])
    False -> effects
  }

  // Email/notification operations
  let effects = case
    string.contains(combined, "email")
    || string.contains(combined, "notification")
    || string.contains(combined, "notify")
  {
    True ->
      list.append(effects, [
        SecondOrderEffect(
          description: "External notification system is triggered",
          severity: Warning,
          category: ExternalDependency,
          has_verification: False,
        ),
      ])
    False -> effects
  }

  // Payment/billing operations
  let effects = case
    string.contains(combined, "payment")
    || string.contains(combined, "charge")
    || string.contains(combined, "billing")
  {
    True ->
      list.append(effects, [
        SecondOrderEffect(
          description: "Financial transaction is recorded",
          severity: Critical,
          category: DataIntegrity,
          has_verification: False,
        ),
      ])
    False -> effects
  }

  // Lock/unlock operations
  let effects = case
    string.contains(combined, "lock") || string.contains(combined, "unlock")
  {
    True ->
      list.append(effects, [
        SecondOrderEffect(
          description: "Concurrent access patterns are affected",
          severity: Warning,
          category: SystemState,
          has_verification: False,
        ),
      ])
    False -> effects
  }

  effects
}

fn check_has_verification(
  effect: SecondOrderEffect,
  required_behaviors: List(String),
) -> Bool {
  // This is a simplified check - in practice, we'd need more sophisticated
  // analysis of what behaviors actually verify what effects
  case effect.category {
    ResourceLifecycle -> required_behaviors != []
    _ -> False
  }
}

// =============================================================================
// ORPHANED RESOURCES
// =============================================================================

fn detect_orphaned_resources(
  behaviors: List(Behavior),
) -> List(OrphanedResource) {
  // Find delete operations
  let delete_behaviors =
    behaviors
    |> list.filter(fn(b) { b.request.method == Delete })

  delete_behaviors
  |> list.flat_map(fn(b) {
    let resource_type = extract_resource_type(b.request.path)
    let orphan_patterns = detect_orphan_patterns(b, resource_type)
    orphan_patterns
  })
}

fn extract_resource_type(path: String) -> String {
  // Extract resource type from path like "/users/{id}" -> "user"
  path
  |> string.split("/")
  |> list.filter(fn(segment) { !string.starts_with(segment, "{") })
  |> list.filter(fn(segment) { segment != "" })
  |> list.first()
  |> result.unwrap("resource")
  |> string.lowercase()
}

fn detect_orphan_patterns(
  behavior: Behavior,
  resource_type: String,
) -> List(OrphanedResource) {
  let _name_lower = string.lowercase(behavior.name)

  case resource_type {
    "user" | "users" | "account" | "accounts" -> [
      OrphanedResource(
        resource_type: "user content",
        caused_by: behavior.name,
        description: "Posts, comments, and user-generated content",
        mitigation: "Implement cascade delete or transfer ownership",
      ),
      OrphanedResource(
        resource_type: "user sessions",
        caused_by: behavior.name,
        description: "Active authentication sessions",
        mitigation: "Invalidate all sessions before user deletion",
      ),
    ]
    "organization" | "organizations" | "team" | "teams" -> [
      OrphanedResource(
        resource_type: "members",
        caused_by: behavior.name,
        description: "Team members lose organization access",
        mitigation: "Require empty team or reassign members",
      ),
      OrphanedResource(
        resource_type: "projects",
        caused_by: behavior.name,
        description: "Projects owned by organization",
        mitigation: "Implement cascade delete or prevent if projects exist",
      ),
    ]
    _ -> []
  }
}

// =============================================================================
// CASCADE OPERATIONS
// =============================================================================

fn detect_cascade_operations(behaviors: List(Behavior)) -> List(CascadeWarning) {
  behaviors
  |> list.filter(fn(b) {
    case b.request.method {
      Delete | Put | Patch -> True
      _ -> False
    }
  })
  |> list.filter_map(fn(b) {
    let cascades = infer_cascades(b)
    case list.length(cascades) {
      0 -> Error(Nil)
      _ ->
        Ok(CascadeWarning(
          operation: b.name,
          cascades_to: cascades,
          requires_transaction: True,
          description: "This operation affects multiple resources",
        ))
    }
  })
}

fn infer_cascades(behavior: Behavior) -> List(String) {
  let name_lower = string.lowercase(behavior.name)
  let path_lower = string.lowercase(behavior.request.path)
  let combined = name_lower <> " " <> path_lower

  let cascades = []

  // User/account operations cascade to sessions
  let cascades = case
    string.contains(combined, "user") || string.contains(combined, "account")
  {
    True -> list.append(cascades, ["sessions"])
    False -> cascades
  }

  // Organization operations cascade to members and projects
  let cascades = case
    string.contains(combined, "org") || string.contains(combined, "team")
  {
    True -> list.concat([cascades, ["members", "projects"]])
    False -> cascades
  }

  cascades
}

// =============================================================================
// STATE DEPENDENCIES
// =============================================================================

fn analyze_state_dependencies(
  behaviors: List(Behavior),
) -> List(StateDependency) {
  behaviors
  |> list.filter(fn(b) { b.requires != [] })
  |> list.map(fn(b) {
    StateDependency(
      behavior: b.name,
      depends_on: b.requires,
      state_mutations: infer_state_mutations(b),
      isolation_level: infer_isolation_level(b),
    )
  })
}

fn infer_state_mutations(behavior: Behavior) -> List(String) {
  case behavior.request.method {
    Post -> ["creates new resource"]
    Put | Patch -> ["modifies existing resource"]
    Delete -> ["removes resource"]
    Get | Head | Options -> []
  }
}

fn infer_isolation_level(behavior: Behavior) -> String {
  case behavior.request.method {
    Get | Head | Options -> "READ_COMMITTED"
    Post | Put | Patch | Delete -> "SERIALIZABLE"
  }
}

// =============================================================================
// FORMATTING
// =============================================================================

pub fn format_report(report: EffectsReport) -> String {
  let header = "=== Second-Order Effects Analysis ===\n\n"

  let summary =
    "Total second-order effects identified: "
    <> int.to_string(report.total_second_order_effects)
    <> "\n"
    <> "Verification coverage: "
    <> float_to_string_1dp(report.coverage_score)
    <> "%\n\n"

  let behavior_section = case list.length(report.behavior_effects) {
    0 -> ""
    _ ->
      "--- Behavior Effects ---\n\n"
      <> {
        report.behavior_effects
        |> list.map(format_behavior_effects)
        |> string.join("\n\n")
      }
      <> "\n\n"
  }

  let orphan_section = case list.length(report.orphaned_resources) {
    0 -> ""
    _ ->
      "--- Orphaned Resources Risks ---\n\n"
      <> {
        report.orphaned_resources
        |> list.map(format_orphaned_resource)
        |> string.join("\n")
      }
      <> "\n\n"
  }

  let cascade_section = case list.length(report.cascade_warnings) {
    0 -> ""
    _ ->
      "--- Cascade Operations ---\n\n"
      <> {
        report.cascade_warnings
        |> list.map(format_cascade_warning)
        |> string.join("\n")
      }
      <> "\n\n"
  }

  let dependency_section = case list.length(report.state_dependencies) {
    0 -> ""
    _ ->
      "--- State Dependencies ---\n\n"
      <> {
        report.state_dependencies
        |> list.map(format_state_dependency)
        |> string.join("\n")
      }
      <> "\n\n"
  }

  header
  <> summary
  <> behavior_section
  <> orphan_section
  <> cascade_section
  <> dependency_section
}

fn format_behavior_effects(be: BehaviorEffects) -> String {
  let first_order =
    "BEHAVIOR: "
    <> be.behavior_name
    <> "\n"
    <> "  First Order: "
    <> be.first_order

  let second_order_str = case list.length(be.second_order) {
    0 -> "  Second Order: (none detected)"
    _ ->
      "  Second Order:\n"
      <> {
        be.second_order
        |> list.map(format_second_order_effect)
        |> string.join("\n")
      }
  }

  let missing = case list.length(be.missing_verifications) {
    0 -> ""
    _ ->
      "\n  Missing Verifications:\n"
      <> {
        be.missing_verifications
        |> list.map(fn(m) { "    - " <> m })
        |> string.join("\n")
      }
  }

  first_order <> "\n" <> second_order_str <> missing
}

fn format_second_order_effect(effect: SecondOrderEffect) -> String {
  let severity_icon = case effect.severity {
    Critical -> "[CRITICAL]"
    Danger -> "[DANGER]"
    Warning -> "[WARNING]"
    Info -> "[INFO]"
  }

  let verification = case effect.has_verification {
    True -> " ✓"
    False -> " ✗"
  }

  "    " <> severity_icon <> " " <> effect.description <> verification
}

fn format_orphaned_resource(orphan: OrphanedResource) -> String {
  "  • "
  <> orphan.resource_type
  <> " (from "
  <> orphan.caused_by
  <> ")\n"
  <> "    Problem: "
  <> orphan.description
  <> "\n"
  <> "    Fix: "
  <> orphan.mitigation
}

fn format_cascade_warning(warning: CascadeWarning) -> String {
  let transaction = case warning.requires_transaction {
    True -> " [REQUIRES TRANSACTION]"
    False -> ""
  }

  "  • "
  <> warning.operation
  <> transaction
  <> "\n"
  <> "    Cascades to: "
  <> string.join(warning.cascades_to, ", ")
  <> "\n"
  <> "    "
  <> warning.description
}

fn format_state_dependency(dep: StateDependency) -> String {
  "  • "
  <> dep.behavior
  <> "\n"
  <> "    Depends on: "
  <> string.join(dep.depends_on, ", ")
  <> "\n"
  <> "    Mutations: "
  <> string.join(dep.state_mutations, ", ")
  <> "\n"
  <> "    Isolation: "
  <> dep.isolation_level
}

fn float_to_string_1dp(f: Float) -> String {
  float.to_string(f)
}

// =============================================================================
// UTILITIES
// =============================================================================

fn get_all_behaviors(spec: Spec) -> List(Behavior) {
  spec.features
  |> list.flat_map(fn(f) { f.behaviors })
}
