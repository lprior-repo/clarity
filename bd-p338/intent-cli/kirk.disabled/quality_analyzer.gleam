/// KIRK Quality Analyzer
///
/// Calculates quality scores across multiple dimensions.
/// Based on empirical research from requirements engineering studies.
import gleam/dict
import gleam/float
import gleam/int
import gleam/json
import gleam/list
import gleam/string
import intent/types.{type Behavior, type Check, type Spec}

// =============================================================================
// TYPES
// =============================================================================

pub type QualityReport {
  QualityReport(
    completeness: Float,
    consistency: Float,
    testability: Float,
    clarity: Float,
    security: Float,
    overall: Float,
    issues: List(QualityIssue),
    suggestions: List(String),
  )
}

pub type QualityIssue {
  QualityIssue(field: String, issue: String, severity: Severity)
}

pub type Severity {
  Info
  Warning
  ErrorLevel
  Critical
}

// =============================================================================
// MAIN ANALYSIS FUNCTION
// =============================================================================

pub fn analyze_quality(spec: Spec) -> QualityReport {
  let completeness = calculate_completeness(spec)
  let consistency = check_consistency(spec)
  let testability = measure_testability(spec)
  let clarity = assess_clarity(spec)
  let security = evaluate_security_coverage(spec)
  let issues = collect_issues(spec)
  let suggestions = generate_suggestions(spec, issues)

  let overall =
    weighted_average([
      #(completeness, 0.2),
      #(consistency, 0.2),
      #(testability, 0.25),
      #(clarity, 0.15),
      #(security, 0.2),
    ])

  QualityReport(
    completeness: completeness,
    consistency: consistency,
    testability: testability,
    clarity: clarity,
    security: security,
    overall: overall,
    issues: issues,
    suggestions: suggestions,
  )
}

// =============================================================================
// COMPLETENESS: Are all required fields filled?
// =============================================================================

fn calculate_completeness(spec: Spec) -> Float {
  let total_fields = count_total_fields(spec)
  let filled_fields = count_filled_fields(spec)

  case total_fields {
    0 -> 100.0
    _ -> int.to_float(filled_fields) /. int.to_float(total_fields) *. 100.0
  }
}

fn count_total_fields(spec: Spec) -> Int {
  // Base spec fields: 5 required (name, desc, audience, version, success_criteria)
  let base = 5

  // Count behavior fields
  let behavior_fields =
    spec.features
    |> list.flat_map(fn(f) { f.behaviors })
    |> list.length()
    |> fn(n) { n * 8 }
  // 8 fields per behavior

  // Count check fields (each check should have rule + why)
  let check_fields =
    spec.features
    |> list.flat_map(fn(f) { f.behaviors })
    |> list.flat_map(fn(b) { dict.values(b.response.checks) })
    |> list.length()
    |> fn(n) { n * 2 }

  base + behavior_fields + check_fields
}

fn count_filled_fields(spec: Spec) -> Int {
  let base =
    bool_to_int(!string.is_empty(spec.name))
    + bool_to_int(!string.is_empty(spec.description))
    + bool_to_int(!string.is_empty(spec.audience))
    + bool_to_int(!string.is_empty(spec.version))
    + bool_to_int(!list.is_empty(spec.success_criteria))

  let behavior_filled =
    spec.features
    |> list.flat_map(fn(f) { f.behaviors })
    |> list.map(count_filled_behavior_fields)
    |> list.fold(0, fn(acc, n) { acc + n })

  let check_filled =
    spec.features
    |> list.flat_map(fn(f) { f.behaviors })
    |> list.flat_map(fn(b) { dict.values(b.response.checks) })
    |> list.map(fn(c: Check) {
      bool_to_int(!string.is_empty(c.rule))
      + bool_to_int(!string.is_empty(c.why))
    })
    |> list.fold(0, fn(acc, n) { acc + n })

  base + behavior_filled + check_filled
}

fn count_filled_behavior_fields(b: Behavior) -> Int {
  bool_to_int(!string.is_empty(b.name))
  + bool_to_int(!string.is_empty(b.intent))
  + bool_to_int(!string.is_empty(b.request.path))
  + bool_to_int(b.response.status > 0)
  + bool_to_int(!dict.is_empty(b.response.checks))
  // notes, requires, tags, captures are optional but count if present
  + bool_to_int(!string.is_empty(b.notes))
  + bool_to_int(!list.is_empty(b.requires))
  + bool_to_int(!list.is_empty(b.tags))
}

// =============================================================================
// CONSISTENCY: No conflicting rules or contradictions
// =============================================================================

fn check_consistency(spec: Spec) -> Float {
  let conflicts = find_conflicts(spec)
  let total_behaviors =
    spec.features
    |> list.flat_map(fn(f) { f.behaviors })
    |> list.length()

  case total_behaviors {
    0 -> 100.0
    _ -> {
      let conflict_penalty = int.to_float(list.length(conflicts)) *. 10.0
      float.max(0.0, 100.0 -. conflict_penalty)
    }
  }
}

fn find_conflicts(spec: Spec) -> List(String) {
  let behaviors =
    spec.features
    |> list.flat_map(fn(f) { f.behaviors })

  // Check for duplicate behavior names
  let names = list.map(behaviors, fn(b) { b.name })
  let duplicate_names = find_duplicates(names)

  // Check for conflicting status expectations on same path
  let path_conflicts = find_path_conflicts(behaviors)

  list.concat([
    list.map(duplicate_names, fn(n) { "Duplicate behavior name: " <> n }),
    path_conflicts,
  ])
}

fn find_duplicates(items: List(String)) -> List(String) {
  items
  |> list.group(fn(x) { x })
  |> dict.filter(fn(_k, v) { list.length(v) > 1 })
  |> dict.keys()
}

fn find_path_conflicts(behaviors: List(Behavior)) -> List(String) {
  // Group by method + path
  behaviors
  |> list.group(fn(b) {
    types.method_to_string(b.request.method) <> " " <> b.request.path
  })
  |> dict.to_list()
  |> list.filter_map(fn(pair) {
    let #(key, bs) = pair
    let statuses =
      bs
      |> list.map(fn(b) { b.response.status })
      |> list.unique()
    case list.length(statuses) > 1 {
      True -> Ok("Multiple status expectations for " <> key)
      False -> Error(Nil)
    }
  })
}

// =============================================================================
// TESTABILITY: Every behavior has verifiable assertions
// =============================================================================

fn measure_testability(spec: Spec) -> Float {
  let behaviors =
    spec.features
    |> list.flat_map(fn(f) { f.behaviors })

  let total = list.length(behaviors)
  let testable =
    behaviors
    |> list.filter(is_behavior_testable)
    |> list.length()

  case total {
    0 -> 100.0
    _ -> int.to_float(testable) /. int.to_float(total) *. 100.0
  }
}

fn is_behavior_testable(b: Behavior) -> Bool {
  // Must have at least status code OR checks
  b.response.status > 0 && !dict.is_empty(b.response.checks)
}

// =============================================================================
// CLARITY: Documentation and explanations present
// =============================================================================

fn assess_clarity(spec: Spec) -> Float {
  let checks_with_why = count_checks_with_why(spec)
  let total_checks = count_total_checks(spec)

  let behaviors =
    spec.features
    |> list.flat_map(fn(f) { f.behaviors })

  let intents_descriptive =
    behaviors
    |> list.filter(fn(b) { string.length(b.intent) >= 10 })
    |> list.length()

  let total_behaviors = list.length(behaviors)

  let why_score = case total_checks {
    0 -> 100.0
    _ -> int.to_float(checks_with_why) /. int.to_float(total_checks) *. 100.0
  }

  let intent_score = case total_behaviors {
    0 -> 100.0
    _ ->
      int.to_float(intents_descriptive)
      /. int.to_float(total_behaviors)
      *. 100.0
  }

  { why_score +. intent_score } /. 2.0
}

fn count_checks_with_why(spec: Spec) -> Int {
  spec.features
  |> list.flat_map(fn(f) { f.behaviors })
  |> list.flat_map(fn(b) { dict.values(b.response.checks) })
  |> list.filter(fn(c: Check) { !string.is_empty(c.why) })
  |> list.length()
}

fn count_total_checks(spec: Spec) -> Int {
  spec.features
  |> list.flat_map(fn(f) { f.behaviors })
  |> list.flat_map(fn(b) { dict.values(b.response.checks) })
  |> list.length()
}

// =============================================================================
// SECURITY: Coverage of security-related behaviors
// =============================================================================

fn evaluate_security_coverage(spec: Spec) -> Float {
  let behaviors =
    spec.features
    |> list.flat_map(fn(f) { f.behaviors })

  let security_keywords = [
    "auth", "login", "password", "token", "jwt", "unauthorized", "forbidden",
    "permission", "role", "secret", "encrypt", "hash", "xss", "injection",
    "csrf", "rate", "limit", "brute",
  ]

  // Count behaviors testing security scenarios
  let security_behaviors =
    behaviors
    |> list.filter(fn(b) {
      let name_lower = string.lowercase(b.name)
      let intent_lower = string.lowercase(b.intent)
      list.any(security_keywords, fn(kw) {
        string.contains(name_lower, kw) || string.contains(intent_lower, kw)
      })
    })
    |> list.length()

  // Count anti-patterns related to security
  let security_anti_patterns =
    spec.anti_patterns
    |> list.filter(fn(ap) {
      let name_lower = string.lowercase(ap.name)
      list.any(security_keywords, fn(kw) { string.contains(name_lower, kw) })
    })
    |> list.length()

  // Check global rules for security
  let security_rules =
    spec.rules
    |> list.filter(fn(r) {
      let body_not_contain = !list.is_empty(r.check.body_must_not_contain)
      let fields_not_exist = !list.is_empty(r.check.fields_must_not_exist)
      body_not_contain || fields_not_exist
    })
    |> list.length()

  // Score based on coverage
  let total_behaviors = list.length(behaviors)
  let behavior_ratio = case total_behaviors {
    0 -> 0.0
    _ -> int.to_float(security_behaviors) /. int.to_float(total_behaviors)
  }

  // Expected: at least 20% security behaviors, some anti-patterns, some rules
  let behavior_score = float.min(1.0, behavior_ratio /. 0.2) *. 40.0
  let anti_pattern_score =
    float.min(1.0, int.to_float(security_anti_patterns) /. 3.0) *. 30.0
  let rule_score = float.min(1.0, int.to_float(security_rules) /. 2.0) *. 30.0

  behavior_score +. anti_pattern_score +. rule_score
}

// =============================================================================
// ISSUES COLLECTION
// =============================================================================

fn collect_issues(spec: Spec) -> List(QualityIssue) {
  let missing_why_issues = find_missing_why_issues(spec)
  let short_intent_issues = find_short_intent_issues(spec)
  let missing_example_issues = find_missing_example_issues(spec)
  let empty_checks_issues = find_empty_checks_issues(spec)

  list.concat([
    missing_why_issues,
    short_intent_issues,
    missing_example_issues,
    empty_checks_issues,
  ])
}

fn find_missing_why_issues(spec: Spec) -> List(QualityIssue) {
  spec.features
  |> list.flat_map(fn(f) {
    f.behaviors
    |> list.flat_map(fn(b) {
      b.response.checks
      |> dict.to_list()
      |> list.filter_map(fn(pair) {
        let #(field, check) = pair
        case string.is_empty(check.why) {
          True ->
            Ok(QualityIssue(
              field: b.name <> ".checks." <> field <> ".why",
              issue: "Missing explanation (why field)",
              severity: Warning,
            ))
          False -> Error(Nil)
        }
      })
    })
  })
}

fn find_short_intent_issues(spec: Spec) -> List(QualityIssue) {
  spec.features
  |> list.flat_map(fn(f) { f.behaviors })
  |> list.filter_map(fn(b) {
    case string.length(b.intent) < 10 {
      True ->
        Ok(QualityIssue(
          field: b.name <> ".intent",
          issue: "Intent too short (< 10 chars)",
          severity: Warning,
        ))
      False -> Error(Nil)
    }
  })
}

fn find_missing_example_issues(spec: Spec) -> List(QualityIssue) {
  spec.features
  |> list.flat_map(fn(f) { f.behaviors })
  |> list.filter_map(fn(b) {
    case b.response.example == json.null() {
      True ->
        Ok(QualityIssue(
          field: b.name <> ".response.example",
          issue: "Missing response example",
          severity: Info,
        ))
      False -> Error(Nil)
    }
  })
}

fn find_empty_checks_issues(spec: Spec) -> List(QualityIssue) {
  spec.features
  |> list.flat_map(fn(f) { f.behaviors })
  |> list.filter_map(fn(b) {
    case dict.is_empty(b.response.checks) {
      True ->
        Ok(QualityIssue(
          field: b.name <> ".response.checks",
          issue: "No response checks defined",
          severity: ErrorLevel,
        ))
      False -> Error(Nil)
    }
  })
}

// =============================================================================
// SUGGESTIONS
// =============================================================================

fn generate_suggestions(spec: Spec, issues: List(QualityIssue)) -> List(String) {
  let behaviors =
    spec.features
    |> list.flat_map(fn(f) { f.behaviors })

  let suggestions = []

  // Suggest adding error behaviors if mostly happy paths
  let error_behaviors =
    behaviors
    |> list.filter(fn(b) { b.response.status >= 400 })
    |> list.length()

  let total = list.length(behaviors)

  let suggestions = case total > 3 && error_behaviors * 3 < total {
    True ->
      list.append(suggestions, [
        "Consider adding more error case behaviors (currently only "
        <> int.to_string(error_behaviors)
        <> " of "
        <> int.to_string(total)
        <> " test error scenarios)",
      ])
    False -> suggestions
  }

  // Suggest anti-patterns if few defined
  let suggestions = case list.length(spec.anti_patterns) < 3 {
    True ->
      list.append(suggestions, [
        "Add more anti-patterns to catch common API design mistakes",
      ])
    False -> suggestions
  }

  // Suggest global rules if few defined
  let suggestions = case list.length(spec.rules) < 2 {
    True ->
      list.append(suggestions, [
        "Add global rules for cross-cutting concerns (e.g., no sensitive data in responses)",
      ])
    False -> suggestions
  }

  // Add suggestions based on issues
  let why_issues =
    issues
    |> list.filter(fn(i) { string.contains(i.issue, "why") })
    |> list.length()

  let suggestions = case why_issues > 3 {
    True ->
      list.append(suggestions, [
        "Many checks are missing 'why' explanations - add them for clarity",
      ])
    False -> suggestions
  }

  suggestions
}

// =============================================================================
// UTILITIES
// =============================================================================

fn weighted_average(items: List(#(Float, Float))) -> Float {
  let total_weight =
    items
    |> list.map(fn(pair) { pair.1 })
    |> list.fold(0.0, fn(acc, w) { acc +. w })

  let weighted_sum =
    items
    |> list.map(fn(pair) { pair.0 *. pair.1 })
    |> list.fold(0.0, fn(acc, v) { acc +. v })

  case total_weight {
    0.0 -> 0.0
    _ -> weighted_sum /. total_weight
  }
}

fn bool_to_int(b: Bool) -> Int {
  case b {
    True -> 1
    False -> 0
  }
}

// =============================================================================
// FORMATTING
// =============================================================================

pub fn format_report(report: QualityReport) -> String {
  let header =
    "╔══════════════════════════════════════╗\n"
    <> "║       KIRK Quality Report            ║\n"
    <> "╚══════════════════════════════════════╝\n\n"

  let scores =
    "📊 Quality Scores:\n"
    <> "  Completeness:  "
    <> format_score(report.completeness)
    <> "\n"
    <> "  Consistency:   "
    <> format_score(report.consistency)
    <> "\n"
    <> "  Testability:   "
    <> format_score(report.testability)
    <> "\n"
    <> "  Clarity:       "
    <> format_score(report.clarity)
    <> "\n"
    <> "  Security:      "
    <> format_score(report.security)
    <> "\n"
    <> "  ─────────────────────────\n"
    <> "  Overall:       "
    <> format_score(report.overall)
    <> "\n\n"

  let issues_section = case list.is_empty(report.issues) {
    True -> "✅ No issues found!\n\n"
    False ->
      "⚠️  Issues ("
      <> int.to_string(list.length(report.issues))
      <> "):\n"
      <> format_issues(report.issues)
      <> "\n"
  }

  let suggestions_section = case list.is_empty(report.suggestions) {
    True -> ""
    False -> "💡 Suggestions:\n" <> format_suggestions(report.suggestions)
  }

  header <> scores <> issues_section <> suggestions_section
}

fn format_score(score: Float) -> String {
  let percentage = float.round(score) |> int.to_string()
  let bar = score_bar(score)
  bar <> " " <> percentage <> "%"
}

fn score_bar(score: Float) -> String {
  let filled = float.round(score /. 10.0) |> int.to_string() |> string.length()
  let empty = 10 - filled
  string.repeat("█", filled) <> string.repeat("░", empty)
}

fn format_issues(issues: List(QualityIssue)) -> String {
  issues
  |> list.map(fn(i) {
    let icon = case i.severity {
      Info -> "ℹ️ "
      Warning -> "⚠️ "
      ErrorLevel -> "❌"
      Critical -> "🚨"
    }
    "  " <> icon <> " " <> i.field <> ": " <> i.issue
  })
  |> string.join("\n")
}

fn format_suggestions(suggestions: List(String)) -> String {
  suggestions
  |> list.index_map(fn(s, i) { "  " <> int.to_string(i + 1) <> ". " <> s })
  |> string.join("\n")
}

pub fn severity_to_string(s: Severity) -> String {
  case s {
    Info -> "info"
    Warning -> "warning"
    ErrorLevel -> "error"
    Critical -> "critical"
  }
}
