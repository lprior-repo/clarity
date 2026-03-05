/// KIRK Gap Detector
///
/// Identifies missing requirements using mental models.
/// Based on empirical research: requirements gaps are top 3 cause of project failure.
import gleam/dict
import gleam/int
import gleam/list
import gleam/string
import intent/types.{type Behavior, type Spec}

// =============================================================================
// TYPES
// =============================================================================

pub type GapReport {
  GapReport(
    inversion_gaps: List(Gap),
    second_order_gaps: List(Gap),
    checklist_gaps: List(Gap),
    coverage_gaps: List(Gap),
    security_gaps: List(Gap),
    total_gaps: Int,
    severity_breakdown: SeverityBreakdown,
  )
}

pub type Gap {
  Gap(
    gap_type: GapType,
    description: String,
    severity: GapSeverity,
    suggestion: String,
    mental_model: String,
  )
}

pub type GapType {
  InversionGap
  SecondOrderGap
  ChecklistGap
  CoverageGap
  SecurityGap
}

pub type GapSeverity {
  Low
  Medium
  High
  Critical
}

pub type SeverityBreakdown {
  SeverityBreakdown(critical: Int, high: Int, medium: Int, low: Int)
}

// =============================================================================
// CHECKLISTS (Based on empirical research)
// =============================================================================

// Standard API behaviors that should exist
const api_behavior_checklist = [
  #("create", "POST", "No behavior tests resource creation"),
  #("read", "GET", "No behavior tests resource retrieval"),
  #("update", "PUT|PATCH", "No behavior tests resource update"),
  #("delete", "DELETE", "No behavior tests resource deletion"),
  #("list", "GET", "No behavior tests listing resources"),
]

// Error handling that should exist
const error_handling_checklist = [
  #(400, "No behavior tests bad request handling"),
  #(401, "No behavior tests unauthorized access"),
  #(403, "No behavior tests forbidden access"),
  #(404, "No behavior tests not found handling"),
  #(409, "No behavior tests conflict handling"),
  #(429, "No behavior tests rate limiting"),
  #(500, "No behavior tests server error handling"),
]

// Security patterns to verify
const security_checklist = [
  #("authentication", ["auth", "login", "token", "jwt"]),
  #("authorization", ["permission", "role", "access", "forbidden"]),
  #("input-validation", ["valid", "format", "sanitize", "reject"]),
  #("sensitive-data", ["password", "secret", "sensitive", "expose"]),
  #("rate-limiting", ["rate", "limit", "throttle", "429"]),
]

// =============================================================================
// MAIN ANALYSIS
// =============================================================================

pub fn detect_gaps(spec: Spec) -> GapReport {
  let behaviors = get_all_behaviors(spec)

  let inversion_gaps = find_inversion_gaps(behaviors, spec)
  let second_order_gaps = find_second_order_gaps(behaviors)
  let checklist_gaps = find_checklist_gaps(behaviors)
  let coverage_gaps = find_coverage_gaps(behaviors)
  let security_gaps = find_security_gaps(spec)

  let all_gaps =
    list.concat([
      inversion_gaps,
      second_order_gaps,
      checklist_gaps,
      coverage_gaps,
      security_gaps,
    ])

  let severity_breakdown = count_severities(all_gaps)

  GapReport(
    inversion_gaps: inversion_gaps,
    second_order_gaps: second_order_gaps,
    checklist_gaps: checklist_gaps,
    coverage_gaps: coverage_gaps,
    security_gaps: security_gaps,
    total_gaps: list.length(all_gaps),
    severity_breakdown: severity_breakdown,
  )
}

fn get_all_behaviors(spec: Spec) -> List(Behavior) {
  spec.features
  |> list.flat_map(fn(f) { f.behaviors })
}

fn count_severities(gaps: List(Gap)) -> SeverityBreakdown {
  let critical =
    gaps |> list.filter(fn(g) { g.severity == Critical }) |> list.length()
  let high = gaps |> list.filter(fn(g) { g.severity == High }) |> list.length()
  let medium =
    gaps |> list.filter(fn(g) { g.severity == Medium }) |> list.length()
  let low = gaps |> list.filter(fn(g) { g.severity == Low }) |> list.length()

  SeverityBreakdown(critical: critical, high: high, medium: medium, low: low)
}

// =============================================================================
// INVERSION GAPS
// "What failure cases are we NOT testing?"
// =============================================================================

fn find_inversion_gaps(behaviors: List(Behavior), spec: Spec) -> List(Gap) {
  // Check if we're testing failure cases
  let error_behaviors =
    behaviors
    |> list.filter(fn(b) { b.response.status >= 400 })
    |> list.length()

  let total_behaviors = list.length(behaviors)

  let ratio = case total_behaviors {
    0 -> 0.0
    _ -> int.to_float(error_behaviors) /. int.to_float(total_behaviors)
  }

  let gaps = case ratio <. 0.2 {
    True -> [
      Gap(
        gap_type: InversionGap,
        description: "Only "
          <> int.to_string(error_behaviors)
          <> " of "
          <> int.to_string(total_behaviors)
          <> " behaviors test error cases",
        severity: High,
        suggestion: "Add more error case behaviors (aim for 30%+ coverage)",
        mental_model: "Inversion",
      ),
    ]
    False -> []
  }

  // Check anti-patterns
  let anti_pattern_gaps = case list.length(spec.anti_patterns) {
    0 -> [
      Gap(
        gap_type: InversionGap,
        description: "No anti-patterns defined",
        severity: Medium,
        suggestion: "Add anti-patterns to document what NOT to do",
        mental_model: "Inversion",
      ),
    ]
    n if n < 3 -> [
      Gap(
        gap_type: InversionGap,
        description: "Only " <> int.to_string(n) <> " anti-patterns defined",
        severity: Low,
        suggestion: "Consider adding more anti-patterns for common mistakes",
        mental_model: "Inversion",
      ),
    ]
    _ -> []
  }

  list.concat([gaps, anti_pattern_gaps])
}

// =============================================================================
// SECOND-ORDER GAPS
// "What happens after this action?"
// =============================================================================

fn find_second_order_gaps(behaviors: List(Behavior)) -> List(Gap) {
  // Find mutation operations (POST, PUT, PATCH, DELETE)
  let mutations =
    behaviors
    |> list.filter(fn(b) {
      case b.request.method {
        types.Post -> True
        types.Put -> True
        types.Patch -> True
        types.Delete -> True
        _ -> False
      }
    })

  // Check if mutations have follow-up verification behaviors
  let _behavior_names = behaviors |> list.map(fn(b) { b.name })

  mutations
  |> list.filter_map(fn(m) {
    // Look for behaviors that depend on this mutation
    let has_dependent =
      behaviors
      |> list.any(fn(b) { list.contains(b.requires, m.name) })

    case has_dependent {
      True -> Error(Nil)
      False ->
        case m.request.method {
          types.Delete ->
            Ok(Gap(
              gap_type: SecondOrderGap,
              description: "Delete '"
                <> m.name
                <> "' has no verification behavior",
              severity: Medium,
              suggestion: "Add behavior to verify resource is actually deleted (GET returns 404)",
              mental_model: "Second-Order Thinking",
            ))
          types.Post ->
            Ok(Gap(
              gap_type: SecondOrderGap,
              description: "Create '" <> m.name <> "' has no follow-up behavior",
              severity: Low,
              suggestion: "Add behavior to verify created resource can be retrieved",
              mental_model: "Second-Order Thinking",
            ))
          _ -> Error(Nil)
        }
    }
  })
}

// =============================================================================
// CHECKLIST GAPS
// Standard behaviors every API should have
// =============================================================================

fn find_checklist_gaps(behaviors: List(Behavior)) -> List(Gap) {
  let methods =
    behaviors
    |> list.map(fn(b) { types.method_to_string(b.request.method) })
    |> list.unique()

  let statuses =
    behaviors
    |> list.map(fn(b) { b.response.status })
    |> list.unique()

  // Check CRUD coverage
  let crud_gaps =
    api_behavior_checklist
    |> list.filter_map(fn(item) {
      let #(name, method_pattern, message) = item
      let has_method =
        list.any(methods, fn(m) { string.contains(method_pattern, m) })
      case has_method {
        True -> Error(Nil)
        False ->
          Ok(Gap(
            gap_type: ChecklistGap,
            description: message,
            severity: Medium,
            suggestion: "Add "
              <> method_pattern
              <> " behavior for "
              <> name
              <> " operation",
            mental_model: "Checklist",
          ))
      }
    })

  // Check error handling coverage
  let error_gaps =
    error_handling_checklist
    |> list.filter_map(fn(item) {
      let #(status, message) = item
      case list.contains(statuses, status) {
        True -> Error(Nil)
        False -> {
          // Only flag important error codes as high severity
          let severity = case status {
            400 -> High
            401 -> High
            404 -> High
            403 -> Medium
            409 -> Medium
            _ -> Low
          }
          Ok(Gap(
            gap_type: ChecklistGap,
            description: message <> " (status " <> int.to_string(status) <> ")",
            severity: severity,
            suggestion: "Add behavior that expects "
              <> int.to_string(status)
              <> " status",
            mental_model: "Checklist",
          ))
        }
      }
    })

  list.concat([crud_gaps, error_gaps])
}

// =============================================================================
// COVERAGE GAPS
// Missing test coverage
// =============================================================================

fn find_coverage_gaps(behaviors: List(Behavior)) -> List(Gap) {
  // Check for behaviors without checks
  let no_checks_gaps =
    behaviors
    |> list.filter(fn(b) { dict.is_empty(b.response.checks) })
    |> list.map(fn(b) {
      Gap(
        gap_type: CoverageGap,
        description: "Behavior '" <> b.name <> "' has no response checks",
        severity: High,
        suggestion: "Add checks to validate the response",
        mental_model: "Coverage",
      )
    })

  // Check for behaviors without intent
  let no_intent_gaps =
    behaviors
    |> list.filter(fn(b) { string.length(b.intent) < 10 })
    |> list.map(fn(b) {
      Gap(
        gap_type: CoverageGap,
        description: "Behavior '" <> b.name <> "' has weak intent description",
        severity: Low,
        suggestion: "Add descriptive intent explaining what this tests",
        mental_model: "Coverage",
      )
    })

  list.concat([no_checks_gaps, no_intent_gaps])
}

// =============================================================================
// SECURITY GAPS
// Missing security testing
// =============================================================================

fn find_security_gaps(spec: Spec) -> List(Gap) {
  let behaviors = get_all_behaviors(spec)

  let all_text =
    behaviors
    |> list.flat_map(fn(b) { [b.name, b.intent, b.notes] })
    |> list.map(string.lowercase)
    |> string.join(" ")

  let anti_pattern_text =
    spec.anti_patterns
    |> list.flat_map(fn(ap) { [ap.name, ap.description] })
    |> list.map(string.lowercase)
    |> string.join(" ")

  let rule_text =
    spec.rules
    |> list.flat_map(fn(r) { [r.name, r.description] })
    |> list.map(string.lowercase)
    |> string.join(" ")

  let combined = all_text <> " " <> anti_pattern_text <> " " <> rule_text

  security_checklist
  |> list.filter_map(fn(item) {
    let #(category, keywords) = item
    let is_covered =
      list.any(keywords, fn(kw) { string.contains(combined, kw) })
    case is_covered {
      True -> Error(Nil)
      False -> {
        let severity = case category {
          "authentication" -> Critical
          "authorization" -> Critical
          "sensitive-data" -> High
          _ -> Medium
        }
        Ok(Gap(
          gap_type: SecurityGap,
          description: "No testing for " <> category,
          severity: severity,
          suggestion: "Add behaviors or rules to test " <> category,
          mental_model: "Security Checklist",
        ))
      }
    }
  })
}

// =============================================================================
// FORMATTING
// =============================================================================

pub fn format_report(report: GapReport) -> String {
  let header =
    "╔══════════════════════════════════════╗\n"
    <> "║        KIRK Gap Detection            ║\n"
    <> "║   Finding what's missing             ║\n"
    <> "╚══════════════════════════════════════╝\n\n"

  let summary = format_summary(report)
  let severity_section = format_severity_breakdown(report.severity_breakdown)

  let inversion_section =
    format_gap_section(
      "🔄 Inversion Gaps",
      report.inversion_gaps,
      "Inversion",
    )
  let second_order_section =
    format_gap_section(
      "🎯 Second-Order Gaps",
      report.second_order_gaps,
      "Second-Order",
    )
  let checklist_section =
    format_gap_section("✅ Checklist Gaps", report.checklist_gaps, "Checklist")
  let coverage_section =
    format_gap_section("📊 Coverage Gaps", report.coverage_gaps, "Coverage")
  let security_section =
    format_gap_section("🔒 Security Gaps", report.security_gaps, "Security")

  header
  <> summary
  <> severity_section
  <> inversion_section
  <> second_order_section
  <> checklist_section
  <> coverage_section
  <> security_section
}

fn format_summary(report: GapReport) -> String {
  let total = report.total_gaps
  let status = case total {
    0 -> "✅ No gaps detected!"
    n if n <= 3 -> "⚠️  " <> int.to_string(n) <> " gaps found"
    n if n <= 10 -> "❌ " <> int.to_string(n) <> " gaps need attention"
    n -> "🚨 " <> int.to_string(n) <> " gaps require immediate action"
  }
  "📋 Summary: " <> status <> "\n\n"
}

fn format_severity_breakdown(sb: SeverityBreakdown) -> String {
  "Severity Breakdown:\n"
  <> "  🚨 Critical: "
  <> int.to_string(sb.critical)
  <> "\n"
  <> "  ❌ High:     "
  <> int.to_string(sb.high)
  <> "\n"
  <> "  ⚠️  Medium:   "
  <> int.to_string(sb.medium)
  <> "\n"
  <> "  ℹ️  Low:      "
  <> int.to_string(sb.low)
  <> "\n\n"
}

fn format_gap_section(title: String, gaps: List(Gap), _model: String) -> String {
  case list.is_empty(gaps) {
    True -> title <> ": None\n\n"
    False ->
      title
      <> " ("
      <> int.to_string(list.length(gaps))
      <> "):\n"
      <> { gaps |> list.map(format_gap) |> string.join("\n") }
      <> "\n\n"
  }
}

fn format_gap(gap: Gap) -> String {
  let icon = case gap.severity {
    Critical -> "🚨"
    High -> "❌"
    Medium -> "⚠️"
    Low -> "ℹ️"
  }
  "  "
  <> icon
  <> " "
  <> gap.description
  <> "\n"
  <> "     💡 "
  <> gap.suggestion
}

pub fn gap_type_to_string(gt: GapType) -> String {
  case gt {
    InversionGap -> "inversion"
    SecondOrderGap -> "second_order"
    ChecklistGap -> "checklist"
    CoverageGap -> "coverage"
    SecurityGap -> "security"
  }
}

pub fn severity_to_string(s: GapSeverity) -> String {
  case s {
    Low -> "low"
    Medium -> "medium"
    High -> "high"
    Critical -> "critical"
  }
}
