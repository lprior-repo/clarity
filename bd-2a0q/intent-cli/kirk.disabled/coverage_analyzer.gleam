/// KIRK Coverage Analyzer
///
/// Measures test coverage across multiple dimensions.
/// Includes OWASP Top 10 security coverage.
import gleam/dict.{type Dict}
import gleam/float
import gleam/int
import gleam/list
import gleam/string
import intent/types.{type Behavior, type Method, type Spec}

// =============================================================================
// TYPES
// =============================================================================

pub type CoverageReport {
  CoverageReport(
    methods: Dict(String, Int),
    status_codes: Dict(String, Int),
    paths: Dict(String, List(Method)),
    edge_cases: EdgeCaseCoverage,
    owasp: OWASPCoverage,
    overall_score: Float,
  )
}

pub type EdgeCaseCoverage {
  EdgeCaseCoverage(tested: List(String), suggested: List(String))
}

pub type OWASPCoverage {
  OWASPCoverage(
    categories: Dict(String, Bool),
    score: Float,
    missing: List(String),
  )
}

// OWASP Top 10 2021
const owasp_categories = [
  #(
    "A01",
    "Broken Access Control",
    ["unauthorized", "forbidden", "access", "privilege", "403", "401"],
  ),
  #(
    "A02",
    "Cryptographic Failures",
    ["password", "encrypt", "hash", "secret", "sensitive", "expose"],
  ),
  #(
    "A03",
    "Injection",
    ["injection", "sql", "xss", "script", "command", "ldap"],
  ),
  #("A04", "Insecure Design", ["validation", "business", "logic", "workflow"]),
  #(
    "A05",
    "Security Misconfiguration",
    ["default", "error", "stack", "verbose", "config"],
  ), #("A06", "Vulnerable Components", ["version", "dependency", "library"]),
  #(
    "A07",
    "Auth Failures",
    ["auth", "login", "session", "token", "jwt", "brute"],
  ), #("A08", "Data Integrity", ["integrity", "serializ", "csrf", "tamper"]),
  #("A09", "Logging Failures", ["log", "audit", "monitor", "alert"]),
  #("A10", "SSRF", ["ssrf", "redirect", "url", "fetch", "request"]),
]

// Common edge cases to test
const edge_case_patterns = [
  #("empty-list", ["empty", "no result", "zero", "[]"]),
  #("pagination-boundary", ["page", "limit", "offset", "cursor"]),
  #("max-length", ["max", "limit", "overflow", "truncat"]),
  #("unicode-input", ["unicode", "emoji", "special char", "utf"]),
  #("null-values", ["null", "nil", "none", "missing"]),
  #("duplicate", ["duplicate", "conflict", "exists", "unique"]),
  #("concurrent", ["concurrent", "race", "parallel", "lock"]),
  #("timeout", ["timeout", "slow", "hang", "long"]),
  #("large-payload", ["large", "big", "size", "upload"]),
  #("special-characters", ["special", "escape", "quote", "slash"]),
]

// =============================================================================
// MAIN ANALYSIS
// =============================================================================

pub fn analyze_coverage(spec: Spec) -> CoverageReport {
  let behaviors = get_all_behaviors(spec)

  let methods = count_methods(behaviors)
  let status_codes = count_status_codes(behaviors)
  let paths = map_paths_to_methods(behaviors)
  let edge_cases = analyze_edge_cases(behaviors)
  let owasp = analyze_owasp_coverage(spec)

  // Calculate overall score
  let method_score = calculate_method_score(methods)
  let status_score = calculate_status_score(status_codes)
  let edge_score = calculate_edge_score(edge_cases)
  let owasp_score = owasp.score

  let overall_score = {
    method_score
    *. 0.2
    +. status_score
    *. 0.2
    +. edge_score
    *. 0.2
    +. owasp_score
    *. 0.4
  }

  CoverageReport(
    methods: methods,
    status_codes: status_codes,
    paths: paths,
    edge_cases: edge_cases,
    owasp: owasp,
    overall_score: overall_score,
  )
}

fn get_all_behaviors(spec: Spec) -> List(Behavior) {
  spec.features
  |> list.flat_map(fn(f) { f.behaviors })
}

// =============================================================================
// METHOD COVERAGE
// =============================================================================

fn count_methods(behaviors: List(Behavior)) -> Dict(String, Int) {
  behaviors
  |> list.map(fn(b) { types.method_to_string(b.request.method) })
  |> list.group(fn(m) { m })
  |> dict.map_values(fn(_k, v) { list.length(v) })
}

fn calculate_method_score(methods: Dict(String, Int)) -> Float {
  // Basic methods: GET, POST, PUT, DELETE, PATCH
  let basic_methods = ["GET", "POST", "PUT", "DELETE", "PATCH"]
  let covered =
    basic_methods
    |> list.filter(fn(m) { dict.has_key(methods, m) })
    |> list.length()
  int.to_float(covered) /. int.to_float(list.length(basic_methods)) *. 100.0
}

// =============================================================================
// STATUS CODE COVERAGE
// =============================================================================

fn count_status_codes(behaviors: List(Behavior)) -> Dict(String, Int) {
  behaviors
  |> list.map(fn(b) { status_code_category(b.response.status) })
  |> list.group(fn(s) { s })
  |> dict.map_values(fn(_k, v) { list.length(v) })
}

fn status_code_category(status: Int) -> String {
  case status {
    s if s >= 200 && s < 300 -> "2xx"
    s if s >= 300 && s < 400 -> "3xx"
    s if s >= 400 && s < 500 -> "4xx"
    s if s >= 500 && s < 600 -> "5xx"
    _ -> "other"
  }
}

fn calculate_status_score(codes: Dict(String, Int)) -> Float {
  // Should have 2xx, 4xx, ideally 5xx
  let has_2xx = dict.has_key(codes, "2xx")
  let has_4xx = dict.has_key(codes, "4xx")
  let has_5xx = dict.has_key(codes, "5xx")

  let score = case True {
    _ if has_2xx && has_4xx && has_5xx -> 100.0
    _ if has_2xx && has_4xx -> 80.0
    _ if has_2xx -> 50.0
    _ -> 0.0
  }
  score
}

// =============================================================================
// PATH COVERAGE
// =============================================================================

fn map_paths_to_methods(behaviors: List(Behavior)) -> Dict(String, List(Method)) {
  behaviors
  |> list.group(fn(b) { normalize_path(b.request.path) })
  |> dict.map_values(fn(_k, bs) {
    bs
    |> list.map(fn(b) { b.request.method })
    |> list.unique()
  })
}

fn normalize_path(path: String) -> String {
  // Replace ${variable} with {param}
  path
  |> string.replace("${", "{")
  |> fn(s) {
    case string.split(s, "}") {
      [first, rest, ..] -> first <> "}" <> normalize_path(rest)
      _ -> s
    }
  }
}

// =============================================================================
// EDGE CASE COVERAGE
// =============================================================================

fn analyze_edge_cases(behaviors: List(Behavior)) -> EdgeCaseCoverage {
  let all_text =
    behaviors
    |> list.flat_map(fn(b) { [b.name, b.intent, b.notes] })
    |> list.map(string.lowercase)
    |> string.join(" ")

  // Find which edge cases are tested
  let tested =
    edge_case_patterns
    |> list.filter_map(fn(pattern) {
      let #(name, keywords) = pattern
      let is_tested =
        list.any(keywords, fn(kw) { string.contains(all_text, kw) })
      case is_tested {
        True -> Ok(name)
        False -> Error(Nil)
      }
    })

  // Suggest missing edge cases
  let suggested =
    edge_case_patterns
    |> list.filter_map(fn(pattern) {
      let #(name, keywords) = pattern
      let is_tested =
        list.any(keywords, fn(kw) { string.contains(all_text, kw) })
      case is_tested {
        True -> Error(Nil)
        False -> Ok(name)
      }
    })

  EdgeCaseCoverage(tested: tested, suggested: suggested)
}

fn calculate_edge_score(edge_cases: EdgeCaseCoverage) -> Float {
  let total = list.length(edge_case_patterns)
  let tested = list.length(edge_cases.tested)
  int.to_float(tested) /. int.to_float(total) *. 100.0
}

// =============================================================================
// OWASP TOP 10 COVERAGE
// =============================================================================

fn analyze_owasp_coverage(spec: Spec) -> OWASPCoverage {
  let behaviors = get_all_behaviors(spec)

  // Collect all searchable text
  let behavior_text =
    behaviors
    |> list.flat_map(fn(b) { [b.name, b.intent, b.notes] })
    |> list.map(string.lowercase)
    |> string.join(" ")

  let anti_pattern_text =
    spec.anti_patterns
    |> list.flat_map(fn(ap) { [ap.name, ap.description, ap.why] })
    |> list.map(string.lowercase)
    |> string.join(" ")

  let rule_text =
    spec.rules
    |> list.flat_map(fn(r) { [r.name, r.description] })
    |> list.map(string.lowercase)
    |> string.join(" ")

  let all_text = behavior_text <> " " <> anti_pattern_text <> " " <> rule_text

  // Check each OWASP category
  let coverage_list =
    owasp_categories
    |> list.map(fn(cat) {
      let #(code, _name, keywords) = cat
      let is_covered =
        list.any(keywords, fn(kw) { string.contains(all_text, kw) })
      #(code, is_covered)
    })

  let categories = dict.from_list(coverage_list)

  let covered = coverage_list |> list.filter(fn(c) { c.1 }) |> list.length()
  let total = list.length(owasp_categories)
  let score = int.to_float(covered) /. int.to_float(total) *. 100.0

  let missing =
    owasp_categories
    |> list.filter_map(fn(cat) {
      let #(code, name, keywords) = cat
      let is_covered =
        list.any(keywords, fn(kw) { string.contains(all_text, kw) })
      case is_covered {
        True -> Error(Nil)
        False -> Ok(code <> ": " <> name)
      }
    })

  OWASPCoverage(categories: categories, score: score, missing: missing)
}

// =============================================================================
// FORMATTING
// =============================================================================

pub fn format_report(report: CoverageReport) -> String {
  let header =
    "╔══════════════════════════════════════╗\n"
    <> "║      KIRK Coverage Analysis          ║\n"
    <> "╚══════════════════════════════════════╝\n\n"

  let overall =
    "📊 Overall Coverage: "
    <> int.to_string(float.round(report.overall_score))
    <> "%\n\n"

  let methods_section = format_methods(report.methods)
  let status_section = format_status_codes(report.status_codes)
  let paths_section = format_paths(report.paths)
  let edge_section = format_edge_cases(report.edge_cases)
  let owasp_section = format_owasp(report.owasp)

  header
  <> overall
  <> methods_section
  <> status_section
  <> paths_section
  <> edge_section
  <> owasp_section
}

fn format_methods(methods: Dict(String, Int)) -> String {
  let items =
    methods
    |> dict.to_list()
    |> list.map(fn(pair) {
      let #(method, count) = pair
      "  " <> method <> ": " <> int.to_string(count)
    })
    |> string.join("\n")

  "🔧 HTTP Methods:\n" <> items <> "\n\n"
}

fn format_status_codes(codes: Dict(String, Int)) -> String {
  let items =
    codes
    |> dict.to_list()
    |> list.sort(fn(a, b) { string.compare(a.0, b.0) })
    |> list.map(fn(pair) {
      let #(code, count) = pair
      let icon = case code {
        "2xx" -> "✅"
        "3xx" -> "↪️"
        "4xx" -> "⚠️"
        "5xx" -> "❌"
        _ -> "•"
      }
      "  " <> icon <> " " <> code <> ": " <> int.to_string(count)
    })
    |> string.join("\n")

  "📊 Status Codes:\n" <> items <> "\n\n"
}

fn format_paths(paths: Dict(String, List(Method))) -> String {
  let items =
    paths
    |> dict.to_list()
    |> list.map(fn(pair) {
      let #(path, methods) = pair
      let method_str =
        methods |> list.map(types.method_to_string) |> string.join(", ")
      "  " <> path <> " [" <> method_str <> "]"
    })
    |> string.join("\n")

  "🛤️  Paths ("
  <> int.to_string(dict.size(paths))
  <> "):\n"
  <> items
  <> "\n\n"
}

fn format_edge_cases(edge_cases: EdgeCaseCoverage) -> String {
  let tested_str = case list.is_empty(edge_cases.tested) {
    True -> "  (none)"
    False ->
      edge_cases.tested
      |> list.map(fn(t) { "  ✅ " <> t })
      |> string.join("\n")
  }

  let suggested_str = case list.is_empty(edge_cases.suggested) {
    True -> ""
    False ->
      "\n  Suggested:\n"
      <> {
        edge_cases.suggested
        |> list.take(5)
        |> list.map(fn(s) { "  💡 " <> s })
        |> string.join("\n")
      }
  }

  "🎯 Edge Cases:\n" <> tested_str <> suggested_str <> "\n\n"
}

fn format_owasp(owasp: OWASPCoverage) -> String {
  let score_str =
    "  Score: "
    <> int.to_string(float.round(owasp.score))
    <> "% ("
    <> int.to_string(10 - list.length(owasp.missing))
    <> "/10)\n"

  let coverage_str =
    owasp.categories
    |> dict.to_list()
    |> list.sort(fn(a, b) { string.compare(a.0, b.0) })
    |> list.map(fn(pair) {
      let #(code, covered) = pair
      let icon = case covered {
        True -> "✅"
        False -> "❌"
      }
      "  " <> icon <> " " <> code
    })
    |> string.join(" ")

  let missing_str = case list.is_empty(owasp.missing) {
    True -> ""
    False ->
      "\n  Missing:\n"
      <> {
        owasp.missing
        |> list.map(fn(m) { "    • " <> m })
        |> string.join("\n")
      }
  }

  "🔐 OWASP Top 10:\n"
  <> score_str
  <> "  "
  <> coverage_str
  <> missing_str
  <> "\n"
}
