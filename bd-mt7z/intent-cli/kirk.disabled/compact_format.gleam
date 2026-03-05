/// KIRK Compact Intent Notation (CIN)
///
/// Token-efficient format for AI prompts.
/// Reduces tokens by ~50% compared to full CUE.
import gleam/dict
import gleam/int
import gleam/json
import gleam/list
import gleam/option.{None, Some}
import gleam/string
import intent/types.{
  type AntiPattern, type Behavior, type Check, type Feature, type Method,
  type Request, type Response, type Rule, type Spec, Delete, Get, Head, Options,
  Patch, Post, Put,
}

// =============================================================================
// COMPACT FORMAT GRAMMAR
// =============================================================================
//
// SPEC name version
// description
//
// F "Feature Name"
// B name "intent"
//   <- requires1, requires2          (dependencies)
//   METHOD /path {body}              (request)
//   200                              (status)
//     field: rule "why"              (checks)
//   >> var: path                     (captures)
//
// R "rule-name" when:status>=400     (global rules)
//   !body: password, secret          (must not contain)
//   +field: error.code               (must exist)
//
// A "anti-pattern"                   (anti-patterns)
//   BAD: {example}
//   GOOD: {example}
//
// =============================================================================

// =============================================================================
// TYPES
// =============================================================================

pub type CompactSpec {
  CompactSpec(
    name: String,
    version: String,
    description: String,
    features: List(CompactFeature),
    rules: List(CompactRule),
    anti_patterns: List(CompactAntiPattern),
  )
}

pub type CompactFeature {
  CompactFeature(name: String, behaviors: List(CompactBehavior))
}

pub type CompactBehavior {
  CompactBehavior(
    name: String,
    intent: String,
    requires: List(String),
    request: String,
    status: Int,
    checks: List(CompactCheck),
    captures: List(String),
  )
}

pub type CompactCheck {
  CompactCheck(field: String, rule: String, why: String)
}

pub type CompactRule {
  CompactRule(
    name: String,
    when: String,
    must_not_contain: List(String),
    must_contain: List(String),
    fields_must_exist: List(String),
    fields_must_not_exist: List(String),
  )
}

pub type CompactAntiPattern {
  CompactAntiPattern(name: String, bad_example: String, good_example: String)
}

// =============================================================================
// SPEC TO COMPACT
// =============================================================================

pub fn spec_to_compact(spec: Spec) -> CompactSpec {
  CompactSpec(
    name: spec.name,
    version: spec.version,
    description: truncate_description(spec.description),
    features: list.map(spec.features, feature_to_compact),
    rules: list.map(spec.rules, rule_to_compact),
    anti_patterns: list.map(spec.anti_patterns, anti_pattern_to_compact),
  )
}

fn truncate_description(desc: String) -> String {
  case string.length(desc) > 100 {
    True -> string.slice(desc, 0, 100) <> "..."
    False -> desc
  }
}

fn feature_to_compact(feature: Feature) -> CompactFeature {
  CompactFeature(
    name: feature.name,
    behaviors: list.map(feature.behaviors, behavior_to_compact),
  )
}

fn behavior_to_compact(b: Behavior) -> CompactBehavior {
  let request_line =
    method_to_string(b.request.method)
    <> " "
    <> b.request.path
    <> body_to_compact(option.Some(b.request.body))

  let checks =
    b.response.checks
    |> dict.to_list()
    |> list.map(fn(pair) {
      let #(field, check) = pair
      CompactCheck(field: field, rule: check.rule, why: check.why)
    })

  let captures =
    b.captures
    |> dict.to_list()
    |> list.map(fn(pair) {
      let #(var, path) = pair
      var <> ": " <> path
    })

  CompactBehavior(
    name: b.name,
    intent: b.intent,
    requires: b.requires,
    request: request_line,
    status: b.response.status,
    checks: checks,
    captures: captures,
  )
}

fn body_to_compact(body: option.Option(json.Json)) -> String {
  case body {
    None -> ""
    Some(j) -> " " <> json.to_string(j)
  }
}

fn rule_to_compact(r: Rule) -> CompactRule {
  let w = r.when
  let status_part = case w.status {
    "" -> ""
    s -> "status:" <> s
  }
  let method_part = types.method_to_string(w.method)
  let path_part = case w.path {
    "" -> ""
    p -> "path:" <> p
  }
  let when_str = case
    [status_part, method_part, path_part]
    |> list.filter(fn(s) { !string.is_empty(s) })
  {
    [] -> "*"
    parts -> string.join(parts, ",")
  }

  CompactRule(
    name: r.name,
    when: when_str,
    must_not_contain: r.check.body_must_not_contain,
    must_contain: r.check.body_must_contain,
    fields_must_exist: r.check.fields_must_exist,
    fields_must_not_exist: r.check.fields_must_not_exist,
  )
}

fn anti_pattern_to_compact(ap: AntiPattern) -> CompactAntiPattern {
  CompactAntiPattern(
    name: ap.name,
    bad_example: json.to_string(ap.bad_example),
    good_example: json.to_string(ap.good_example),
  )
}

// =============================================================================
// FORMAT TO STRING (CIN OUTPUT)
// =============================================================================

pub fn format_compact(compact: CompactSpec) -> String {
  let header =
    "SPEC \""
    <> compact.name
    <> "\" "
    <> compact.version
    <> "\n"
    <> compact.description
    <> "\n\n"

  let features =
    compact.features
    |> list.map(format_feature)
    |> string.join("\n")

  let rules = case list.is_empty(compact.rules) {
    True -> ""
    False ->
      "\n# Rules\n"
      <> { compact.rules |> list.map(format_rule) |> string.join("\n") }
  }

  let anti_patterns = case list.is_empty(compact.anti_patterns) {
    True -> ""
    False ->
      "\n# Anti-patterns\n"
      <> {
        compact.anti_patterns
        |> list.map(format_anti_pattern)
        |> string.join("\n")
      }
  }

  header <> features <> rules <> anti_patterns
}

fn format_feature(f: CompactFeature) -> String {
  let header = "F \"" <> f.name <> "\"\n"
  let behaviors =
    f.behaviors
    |> list.map(format_behavior)
    |> string.join("\n")

  header <> behaviors
}

fn format_behavior(b: CompactBehavior) -> String {
  let header = "B " <> b.name <> " \"" <> b.intent <> "\"\n"

  let requires = case list.is_empty(b.requires) {
    True -> ""
    False -> "  <- " <> string.join(b.requires, ", ") <> "\n"
  }

  let request = "  " <> b.request <> "\n"
  let status = "  " <> int.to_string(b.status) <> "\n"

  let checks =
    b.checks
    |> list.map(fn(c) {
      let why = case string.is_empty(c.why) {
        True -> ""
        False -> " \"" <> c.why <> "\""
      }
      "    " <> c.field <> ": " <> c.rule <> why
    })
    |> string.join("\n")

  let checks_section = case list.is_empty(b.checks) {
    True -> ""
    False -> checks <> "\n"
  }

  let captures = case list.is_empty(b.captures) {
    True -> ""
    False ->
      b.captures
      |> list.map(fn(c) { "  >> " <> c })
      |> string.join("\n")
      |> fn(s) { s <> "\n" }
  }

  header <> requires <> request <> status <> checks_section <> captures
}

fn format_rule(r: CompactRule) -> String {
  let header = "R \"" <> r.name <> "\" when:" <> r.when <> "\n"

  let must_not =
    r.must_not_contain
    |> list.map(fn(s) { "  !body: " <> s })
    |> string.join("\n")

  let must =
    r.must_contain
    |> list.map(fn(s) { "  +body: " <> s })
    |> string.join("\n")

  let fields_exist =
    r.fields_must_exist
    |> list.map(fn(s) { "  +field: " <> s })
    |> string.join("\n")

  let fields_not_exist =
    r.fields_must_not_exist
    |> list.map(fn(s) { "  !field: " <> s })
    |> string.join("\n")

  header
  <> {
    [must_not, must, fields_exist, fields_not_exist]
    |> list.filter(fn(s) { !string.is_empty(s) })
    |> string.join("\n")
  }
  <> "\n"
}

fn format_anti_pattern(ap: CompactAntiPattern) -> String {
  "A \""
  <> ap.name
  <> "\"\n"
  <> "  BAD: "
  <> ap.bad_example
  <> "\n"
  <> "  GOOD: "
  <> ap.good_example
  <> "\n"
}

// =============================================================================
// HELPER: Method to string
// =============================================================================

fn method_to_string(m: Method) -> String {
  case m {
    Get -> "GET"
    Post -> "POST"
    Put -> "PUT"
    Patch -> "PATCH"
    Delete -> "DELETE"
    Head -> "HEAD"
    Options -> "OPTIONS"
  }
}

// =============================================================================
// TOKEN COUNTING (Approximate)
// =============================================================================

pub fn estimate_tokens(text: String) -> Int {
  // Rough approximation: 1 token ≈ 4 characters
  let chars = string.length(text)
  chars / 4
}

pub fn compare_token_usage(spec: Spec) -> #(Int, Int, Float) {
  // Get full JSON representation
  let full_json = json.to_string(spec_to_json(spec))
  let full_tokens = estimate_tokens(full_json)

  // Get compact representation
  let compact = spec_to_compact(spec)
  let compact_text = format_compact(compact)
  let compact_tokens = estimate_tokens(compact_text)

  // Calculate savings
  let savings = case full_tokens {
    0 -> 0.0
    _ ->
      {
        int.to_float(full_tokens - compact_tokens) /. int.to_float(full_tokens)
      }
      *. 100.0
  }

  #(full_tokens, compact_tokens, savings)
}

// Helper to convert spec to JSON (simplified)
fn spec_to_json(spec: Spec) -> json.Json {
  json.object([
    #("name", json.string(spec.name)),
    #("description", json.string(spec.description)),
    #("version", json.string(spec.version)),
    #(
      "features",
      json.array(spec.features, fn(f) {
        json.object([
          #("name", json.string(f.name)),
          #(
            "behaviors",
            json.array(f.behaviors, fn(b) {
              json.object([
                #("name", json.string(b.name)),
                #("intent", json.string(b.intent)),
                #(
                  "request",
                  json.object([
                    #("method", json.string(method_to_string(b.request.method))),
                    #("path", json.string(b.request.path)),
                  ]),
                ),
                #(
                  "response",
                  json.object([#("status", json.int(b.response.status))]),
                ),
              ])
            }),
          ),
        ])
      }),
    ),
  ])
}

// =============================================================================
// PROTOBUF TEXT FORMAT OUTPUT
// =============================================================================

pub fn spec_to_prototext(spec: Spec) -> String {
  let header =
    "# KIRK Spec - Protobuf Text Format\n"
    <> "# Generated from CUE source of truth\n\n"

  let spec_block =
    "spec {\n"
    <> "  name: \""
    <> escape_string(spec.name)
    <> "\"\n"
    <> "  description: \""
    <> escape_string(spec.description)
    <> "\"\n"
    <> "  version: \""
    <> escape_string(spec.version)
    <> "\"\n"
    <> "  audience: \""
    <> escape_string(spec.audience)
    <> "\"\n"
    <> format_success_criteria(spec.success_criteria)
    <> format_config_proto(spec.config)
    <> format_features_proto(spec.features)
    <> format_rules_proto(spec.rules)
    <> format_anti_patterns_proto(spec.anti_patterns)
    <> "}\n"

  header <> spec_block
}

fn escape_string(s: String) -> String {
  s
  |> string.replace("\\", "\\\\")
  |> string.replace("\"", "\\\"")
  |> string.replace("\n", "\\n")
}

fn format_success_criteria(criteria: List(String)) -> String {
  criteria
  |> list.map(fn(c) { "  success_criteria: \"" <> escape_string(c) <> "\"\n" })
  |> string.join("")
}

fn format_config_proto(config: types.Config) -> String {
  "  config {\n"
  <> "    base_url: \""
  <> escape_string(config.base_url)
  <> "\"\n"
  <> "    timeout_ms: "
  <> int.to_string(config.timeout_ms)
  <> "\n"
  <> format_headers_proto(config.headers, "    ")
  <> "  }\n"
}

fn format_headers_proto(
  headers: dict.Dict(String, String),
  indent: String,
) -> String {
  headers
  |> dict.to_list()
  |> list.map(fn(pair) {
    let #(k, v) = pair
    indent
    <> "headers {\n"
    <> indent
    <> "  key: \""
    <> escape_string(k)
    <> "\"\n"
    <> indent
    <> "  value: \""
    <> escape_string(v)
    <> "\"\n"
    <> indent
    <> "}\n"
  })
  |> string.join("")
}

fn format_features_proto(features: List(Feature)) -> String {
  features
  |> list.map(fn(f) {
    "  features {\n"
    <> "    name: \""
    <> escape_string(f.name)
    <> "\"\n"
    <> "    description: \""
    <> escape_string(f.description)
    <> "\"\n"
    <> format_behaviors_proto(f.behaviors)
    <> "  }\n"
  })
  |> string.join("")
}

fn format_behaviors_proto(behaviors: List(Behavior)) -> String {
  behaviors
  |> list.map(fn(b) {
    "    behaviors {\n"
    <> "      name: \""
    <> escape_string(b.name)
    <> "\"\n"
    <> "      intent: \""
    <> escape_string(b.intent)
    <> "\"\n"
    <> format_requires_proto(b.requires)
    <> format_request_proto(b.request)
    <> format_response_proto(b.response)
    <> format_captures_proto(b.captures)
    <> "    }\n"
  })
  |> string.join("")
}

fn format_requires_proto(requires: List(String)) -> String {
  requires
  |> list.map(fn(r) { "      requires: \"" <> escape_string(r) <> "\"\n" })
  |> string.join("")
}

fn format_request_proto(req: Request) -> String {
  "      request {\n"
  <> "        method: "
  <> method_to_proto_enum(req.method)
  <> "\n"
  <> "        path: \""
  <> escape_string(req.path)
  <> "\"\n"
  <> format_headers_proto(req.headers, "        ")
  <> format_body_proto(option.Some(req.body))
  <> "      }\n"
}

fn method_to_proto_enum(m: Method) -> String {
  case m {
    Get -> "GET"
    Post -> "POST"
    Put -> "PUT"
    Patch -> "PATCH"
    Delete -> "DELETE"
    Head -> "HEAD"
    Options -> "OPTIONS"
  }
}

fn format_body_proto(body: option.Option(json.Json)) -> String {
  case body {
    None -> ""
    Some(j) -> "        body: \"" <> escape_string(json.to_string(j)) <> "\"\n"
  }
}

fn format_response_proto(resp: Response) -> String {
  "      response {\n"
  <> "        status: "
  <> int.to_string(resp.status)
  <> "\n"
  <> format_checks_proto(resp.checks)
  <> "      }\n"
}

fn format_checks_proto(checks: dict.Dict(String, Check)) -> String {
  checks
  |> dict.to_list()
  |> list.map(fn(pair) {
    let #(field, check) = pair
    "        checks {\n"
    <> "          key: \""
    <> escape_string(field)
    <> "\"\n"
    <> "          value {\n"
    <> "            rule: \""
    <> escape_string(check.rule)
    <> "\"\n"
    <> "            why: \""
    <> escape_string(check.why)
    <> "\"\n"
    <> "          }\n"
    <> "        }\n"
  })
  |> string.join("")
}

fn format_captures_proto(captures: dict.Dict(String, String)) -> String {
  captures
  |> dict.to_list()
  |> list.map(fn(pair) {
    let #(k, v) = pair
    "      captures {\n"
    <> "        key: \""
    <> escape_string(k)
    <> "\"\n"
    <> "        value: \""
    <> escape_string(v)
    <> "\"\n"
    <> "      }\n"
  })
  |> string.join("")
}

fn format_rules_proto(rules: List(Rule)) -> String {
  rules
  |> list.map(fn(r) {
    "  rules {\n"
    <> "    name: \""
    <> escape_string(r.name)
    <> "\"\n"
    <> "    description: \""
    <> escape_string(r.description)
    <> "\"\n"
    <> "  }\n"
  })
  |> string.join("")
}

fn format_anti_patterns_proto(patterns: List(AntiPattern)) -> String {
  patterns
  |> list.map(fn(ap) {
    "  anti_patterns {\n"
    <> "    name: \""
    <> escape_string(ap.name)
    <> "\"\n"
    <> "    description: \""
    <> escape_string(ap.description)
    <> "\"\n"
    <> "    why: \""
    <> escape_string(ap.why)
    <> "\"\n"
    <> "  }\n"
  })
  |> string.join("")
}
