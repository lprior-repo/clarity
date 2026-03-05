/// KIRK Inversion Checker
///
/// "Invert, always invert" - Charlie Munger/Jacobi.
/// Analyzes specs for missing failure cases.
import gleam/int
import gleam/list
import gleam/string
import intent/types.{type Behavior, type Method, type Spec, Get, Post}

// =============================================================================
// TYPES
// =============================================================================

pub type InversionReport {
  InversionReport(
    security_gaps: List(InversionGap),
    usability_gaps: List(InversionGap),
    integration_gaps: List(InversionGap),
    suggested_behaviors: List(SuggestedBehavior),
    score: Float,
  )
}

pub type InversionGap {
  InversionGap(
    category: String,
    description: String,
    severity: GapSeverity,
    what_could_fail: String,
  )
}

pub type GapSeverity {
  Low
  Medium
  High
  Critical
}

pub type SuggestedBehavior {
  SuggestedBehavior(
    name: String,
    intent: String,
    method: Method,
    path: String,
    expected_status: Int,
    category: String,
  )
}

// =============================================================================
// SECURITY INVERSIONS
// What security failures are we NOT testing?
// =============================================================================

const security_inversions = [
  #("auth-bypass", "Accessing protected resources without authentication", 401),
  #("expired-token", "Using expired authentication tokens", 401),
  #("invalid-token", "Using malformed or invalid tokens", 401),
  #("wrong-user-access", "Accessing another user's resources", 403),
  #("privilege-escalation", "Attempting admin actions as regular user", 403),
  #("sql-injection", "SQL injection in query parameters", 400),
  #("xss-payload", "XSS payloads in user-controlled fields", 400),
  #("oversized-request", "Request body exceeding size limits", 413),
  #("rate-limit-exceeded", "Exceeding rate limits", 429),
  #("brute-force-lockout", "Account lockout after failed attempts", 429),
]

const usability_inversions = [
  #("not-found", "Requesting non-existent resources", 404),
  #("invalid-format", "Sending malformed request data", 400),
  #("missing-required", "Omitting required fields", 400),
  #("invalid-type", "Wrong data types in fields", 400),
  #("empty-list", "Empty list/pagination edge case", 200),
  #("max-pagination", "Requesting beyond available pages", 200),
  #("duplicate-create", "Creating duplicate resources", 409),
  #("concurrent-modify", "Conflicting concurrent modifications", 409),
]

const integration_inversions = [
  #("idempotency", "Testing idempotency key behavior", 200),
  #("timeout-handling", "Long-running operation timeout", 504),
  #("partial-failure", "Partial success in batch operations", 207),
  #("version-mismatch", "API version compatibility", 400),
  #("content-negotiation", "Unsupported content types", 415),
  #("method-not-allowed", "Using wrong HTTP method", 405),
]

// =============================================================================
// MAIN ANALYSIS
// =============================================================================

pub fn analyze_inversions(spec: Spec) -> InversionReport {
  let behaviors = get_all_behaviors(spec)
  let paths = get_all_paths(behaviors)

  // Check each category of inversions
  let security_gaps = check_security_inversions(behaviors, paths, spec)
  let usability_gaps = check_usability_inversions(behaviors, paths)
  let integration_gaps = check_integration_inversions(behaviors, paths)

  // Generate suggested behaviors
  let suggested =
    generate_suggestions(security_gaps, usability_gaps, integration_gaps, paths)

  // Calculate score (percentage of inversions covered)
  let total_expected =
    list.length(security_inversions)
    + list.length(usability_inversions)
    + list.length(integration_inversions)

  let total_gaps =
    list.length(security_gaps)
    + list.length(usability_gaps)
    + list.length(integration_gaps)

  let score = case total_expected {
    0 -> 100.0
    _ ->
      int.to_float(total_expected - total_gaps)
      /. int.to_float(total_expected)
      *. 100.0
  }

  InversionReport(
    security_gaps: security_gaps,
    usability_gaps: usability_gaps,
    integration_gaps: integration_gaps,
    suggested_behaviors: suggested,
    score: score,
  )
}

fn get_all_behaviors(spec: Spec) -> List(Behavior) {
  spec.features
  |> list.flat_map(fn(f) { f.behaviors })
}

fn get_all_paths(behaviors: List(Behavior)) -> List(String) {
  behaviors
  |> list.map(fn(b) { b.request.path })
  |> list.unique()
}

// =============================================================================
// SECURITY INVERSION CHECKS
// =============================================================================

fn check_security_inversions(
  behaviors: List(Behavior),
  _paths: List(String),
  spec: Spec,
) -> List(InversionGap) {
  let behavior_names =
    behaviors
    |> list.map(fn(b) { string.lowercase(b.name) })

  let behavior_intents =
    behaviors
    |> list.map(fn(b) { string.lowercase(b.intent) })

  let behavior_statuses =
    behaviors
    |> list.map(fn(b) { b.response.status })

  // Check which security inversions are missing
  security_inversions
  |> list.filter_map(fn(inv) {
    let #(name, desc, expected_status) = inv
    let is_tested =
      is_inversion_covered(
        name,
        expected_status,
        behavior_names,
        behavior_intents,
        behavior_statuses,
        spec,
      )
    case is_tested {
      True -> Error(Nil)
      False ->
        Ok(InversionGap(
          category: "security",
          description: desc,
          severity: security_severity(name),
          what_could_fail: "Without testing "
            <> name
            <> ", attackers could exploit: "
            <> desc,
        ))
    }
  })
}

fn security_severity(name: String) -> GapSeverity {
  case name {
    "sql-injection" -> Critical
    "xss-payload" -> Critical
    "auth-bypass" -> Critical
    "privilege-escalation" -> High
    "wrong-user-access" -> High
    "expired-token" -> Medium
    "invalid-token" -> Medium
    "brute-force-lockout" -> Medium
    _ -> Low
  }
}

// =============================================================================
// USABILITY INVERSION CHECKS
// =============================================================================

fn check_usability_inversions(
  behaviors: List(Behavior),
  _paths: List(String),
) -> List(InversionGap) {
  let behavior_names =
    behaviors
    |> list.map(fn(b) { string.lowercase(b.name) })

  let behavior_intents =
    behaviors
    |> list.map(fn(b) { string.lowercase(b.intent) })

  let behavior_statuses =
    behaviors
    |> list.map(fn(b) { b.response.status })

  usability_inversions
  |> list.filter_map(fn(inv) {
    let #(name, desc, expected_status) = inv
    let name_parts = string.split(name, "-")
    let is_tested =
      list.any(behavior_names, fn(bn) {
        list.any(name_parts, fn(part) { string.contains(bn, part) })
      })
      || list.any(behavior_intents, fn(bi) {
        list.any(name_parts, fn(part) { string.contains(bi, part) })
      })
      || list.contains(behavior_statuses, expected_status)

    case is_tested {
      True -> Error(Nil)
      False ->
        Ok(InversionGap(
          category: "usability",
          description: desc,
          severity: usability_severity(name),
          what_could_fail: "Users may experience poor UX when: " <> desc,
        ))
    }
  })
}

fn usability_severity(name: String) -> GapSeverity {
  case name {
    "not-found" -> High
    "invalid-format" -> High
    "missing-required" -> High
    "duplicate-create" -> Medium
    _ -> Low
  }
}

// =============================================================================
// INTEGRATION INVERSION CHECKS
// =============================================================================

fn check_integration_inversions(
  behaviors: List(Behavior),
  _paths: List(String),
) -> List(InversionGap) {
  let behavior_names =
    behaviors
    |> list.map(fn(b) { string.lowercase(b.name) })

  let behavior_intents =
    behaviors
    |> list.map(fn(b) { string.lowercase(b.intent) })

  let behavior_statuses =
    behaviors
    |> list.map(fn(b) { b.response.status })

  integration_inversions
  |> list.filter_map(fn(inv) {
    let #(name, desc, expected_status) = inv
    let name_parts = string.split(name, "-")
    let is_tested =
      list.any(behavior_names, fn(bn) {
        list.any(name_parts, fn(part) { string.contains(bn, part) })
      })
      || list.any(behavior_intents, fn(bi) {
        list.any(name_parts, fn(part) { string.contains(bi, part) })
      })
      || list.contains(behavior_statuses, expected_status)

    case is_tested {
      True -> Error(Nil)
      False ->
        Ok(InversionGap(
          category: "integration",
          description: desc,
          severity: integration_severity(name),
          what_could_fail: "Integrations may break when: " <> desc,
        ))
    }
  })
}

fn integration_severity(name: String) -> GapSeverity {
  case name {
    "idempotency" -> High
    "concurrent-modify" -> High
    "timeout-handling" -> Medium
    _ -> Low
  }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

fn is_inversion_covered(
  name: String,
  expected_status: Int,
  behavior_names: List(String),
  behavior_intents: List(String),
  behavior_statuses: List(Int),
  spec: Spec,
) -> Bool {
  let name_parts = string.split(name, "-")

  // Check if any behavior tests this
  let name_match =
    list.any(behavior_names, fn(bn) {
      list.any(name_parts, fn(part) { string.contains(bn, part) })
    })

  let intent_match =
    list.any(behavior_intents, fn(bi) {
      list.any(name_parts, fn(part) { string.contains(bi, part) })
    })

  let status_match = list.contains(behavior_statuses, expected_status)

  // Check anti-patterns for security coverage
  let anti_pattern_match =
    spec.anti_patterns
    |> list.any(fn(ap) {
      let ap_name = string.lowercase(ap.name)
      list.any(name_parts, fn(part) { string.contains(ap_name, part) })
    })

  // Check global rules
  let rule_match =
    spec.rules
    |> list.any(fn(r) {
      let r_name = string.lowercase(r.name)
      list.any(name_parts, fn(part) { string.contains(r_name, part) })
    })

  name_match || intent_match || status_match || anti_pattern_match || rule_match
}

// =============================================================================
// SUGGESTION GENERATION
// =============================================================================

fn generate_suggestions(
  security_gaps: List(InversionGap),
  usability_gaps: List(InversionGap),
  integration_gaps: List(InversionGap),
  paths: List(String),
) -> List(SuggestedBehavior) {
  let primary_path = case paths {
    [p, ..] -> p
    [] -> "/resource"
  }

  let security_suggestions =
    security_gaps
    |> list.take(5)
    // Limit suggestions
    |> list.map(fn(gap) {
      let #(method, status) = gap_to_method_status(gap.description)
      SuggestedBehavior(
        name: gap_to_name(gap),
        intent: "Test: " <> gap.description,
        method: method,
        path: primary_path,
        expected_status: status,
        category: "security",
      )
    })

  let usability_suggestions =
    usability_gaps
    |> list.take(3)
    |> list.map(fn(gap) {
      let #(method, status) = gap_to_method_status(gap.description)
      SuggestedBehavior(
        name: gap_to_name(gap),
        intent: "Test: " <> gap.description,
        method: method,
        path: primary_path,
        expected_status: status,
        category: "usability",
      )
    })

  let integration_suggestions =
    integration_gaps
    |> list.take(2)
    |> list.map(fn(gap) {
      let #(method, status) = gap_to_method_status(gap.description)
      SuggestedBehavior(
        name: gap_to_name(gap),
        intent: "Test: " <> gap.description,
        method: method,
        path: primary_path,
        expected_status: status,
        category: "integration",
      )
    })

  list.concat([
    security_suggestions,
    usability_suggestions,
    integration_suggestions,
  ])
}

fn gap_to_name(gap: InversionGap) -> String {
  gap.description
  |> string.lowercase()
  |> string.replace(" ", "-")
  |> string.replace("'", "")
  |> fn(s) { string.slice(s, 0, 30) }
}

fn gap_to_method_status(description: String) -> #(Method, Int) {
  let desc = string.lowercase(description)
  case string.contains(desc, "authentication") {
    True -> #(Get, 401)
    False ->
      case string.contains(desc, "token") {
        True -> #(Get, 401)
        False ->
          case string.contains(desc, "access") {
            True -> #(Get, 403)
            False ->
              case string.contains(desc, "admin") {
                True -> #(Post, 403)
                False ->
                  case string.contains(desc, "injection") {
                    True -> #(Post, 400)
                    False ->
                      case string.contains(desc, "xss") {
                        True -> #(Post, 400)
                        False ->
                          case string.contains(desc, "not-found") {
                            True -> #(Get, 404)
                            False ->
                              case string.contains(desc, "non-existent") {
                                True -> #(Get, 404)
                                False ->
                                  case string.contains(desc, "malformed") {
                                    True -> #(Post, 400)
                                    False ->
                                      case string.contains(desc, "required") {
                                        True -> #(Post, 400)
                                        False ->
                                          case
                                            string.contains(desc, "duplicate")
                                          {
                                            True -> #(Post, 409)
                                            False ->
                                              case
                                                string.contains(desc, "rate")
                                              {
                                                True -> #(Get, 429)
                                                False ->
                                                  case
                                                    string.contains(
                                                      desc,
                                                      "timeout",
                                                    )
                                                  {
                                                    True -> #(Get, 504)
                                                    False -> #(Get, 400)
                                                  }
                                              }
                                          }
                                      }
                                  }
                              }
                          }
                      }
                  }
              }
          }
      }
  }
}

// =============================================================================
// FORMATTING
// =============================================================================

pub fn format_report(report: InversionReport) -> String {
  let header =
    "╔══════════════════════════════════════╗\n"
    <> "║      KIRK Inversion Analysis         ║\n"
    <> "║   \"What would make this fail?\"       ║\n"
    <> "╚══════════════════════════════════════╝\n\n"

  let score_line =
    "📊 Inversion Coverage: "
    <> int.to_string(float.round(report.score))
    <> "%\n\n"

  let security_section =
    format_gap_section("🔒 Security Gaps", report.security_gaps)
  let usability_section =
    format_gap_section("👤 Usability Gaps", report.usability_gaps)
  let integration_section =
    format_gap_section("🔌 Integration Gaps", report.integration_gaps)

  let suggestions_section = case list.is_empty(report.suggested_behaviors) {
    True -> ""
    False ->
      "\n💡 Suggested Behaviors to Add:\n"
      <> format_suggestions(report.suggested_behaviors)
  }

  header
  <> score_line
  <> security_section
  <> usability_section
  <> integration_section
  <> suggestions_section
}

fn format_gap_section(title: String, gaps: List(InversionGap)) -> String {
  case list.is_empty(gaps) {
    True -> title <> ": ✅ All covered!\n\n"
    False ->
      title
      <> " ("
      <> int.to_string(list.length(gaps))
      <> " missing):\n"
      <> list.map(gaps, format_gap) |> string.join("\n")
      <> "\n\n"
  }
}

fn format_gap(gap: InversionGap) -> String {
  let icon = case gap.severity {
    Critical -> "🚨"
    High -> "❌"
    Medium -> "⚠️ "
    Low -> "ℹ️ "
  }
  "  " <> icon <> " " <> gap.description
}

fn format_suggestions(suggestions: List(SuggestedBehavior)) -> String {
  suggestions
  |> list.map(fn(s) {
    "  • "
    <> s.name
    <> " ["
    <> types.method_to_string(s.method)
    <> " "
    <> int.to_string(s.expected_status)
    <> "]"
    <> "\n    "
    <> s.intent
  })
  |> string.join("\n")
}

import gleam/float

pub fn severity_to_string(s: GapSeverity) -> String {
  case s {
    Low -> "low"
    Medium -> "medium"
    High -> "high"
    Critical -> "critical"
  }
}
