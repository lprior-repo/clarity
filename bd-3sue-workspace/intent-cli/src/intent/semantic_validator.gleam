/// Semantic validation for CUE specifications
/// Validates semantic rules beyond CUE syntax checking
import gleam/dict.{type Dict}
import gleam/dynamic
import gleam/int
import gleam/list
import gleam/option
import gleam/string

/// Maximum allowed string length (100KB as per contract)
const max_string_length = 100_000

/// Validation error with structured context
pub type ValidationError {
  ValidationError(
    field: String,
    rule: String,
    expected: String,
    actual: String,
    explanation: String,
  )
}

/// Validation result type
pub type ValidationResult {
  Valid
  Invalid(List(ValidationError))
}

/// Spec type for semantic validation
/// Note: This is a simplified version focused on validation requirements
pub type Spec {
  Spec(
    name: String,
    description: String,
    audience: String,
    version: String,
    success_criteria: List(String),
    config: Config,
    features: List(Feature),
    rules: List(Rule),
    anti_patterns: List(AntiPattern),
    ai_hints: AIHints,
  )
}

pub type Config {
  Config(base_url: String, timeout_ms: Int, headers: Dict(String, String))
}

pub type Feature {
  Feature(name: String, description: String, behaviors: List(Behavior))
}

pub type Behavior {
  Behavior(
    name: String,
    intent: String,
    notes: String,
    requires: List(String),
    tags: List(String),
    request: Request,
    response: Response,
    captures: Dict(String, String),
  )
}

pub type Request {
  Request(
    method: String,
    path: String,
    headers: Dict(String, String),
    query: Dict(String, String),
    body: option.Option(dynamic.Dynamic),
  )
}

pub type Response {
  Response(
    status: Int,
    example: option.Option(dynamic.Dynamic),
    checks: Dict(String, Check),
    headers: option.Option(Dict(String, String)),
  )
}

pub type Check {
  Check(rule: String, why: String)
}

pub type Rule {
  Rule(name: String, description: String, check: RuleCheck)
}

pub type RuleCheck {
  RuleCheck(
    body_must_not_contain: option.Option(List(String)),
    body_must_contain: option.Option(List(String)),
    fields_must_exist: option.Option(List(String)),
    fields_must_not_exist: option.Option(List(String)),
    header_must_exist: option.Option(String),
    header_must_not_exist: option.Option(String),
  )
}

pub type AntiPattern {
  AntiPattern(
    name: String,
    description: String,
    bad_example: dynamic.Dynamic,
    good_example: dynamic.Dynamic,
    why: String,
  )
}

pub type AIHints {
  AIHints(
    implementation: option.Option(dynamic.Dynamic),
    entities: option.Option(dynamic.Dynamic),
    security: option.Option(dynamic.Dynamic),
    pitfalls: option.Option(List(String)),
  )
}

/// Validate all semantic rules for a spec
pub fn validate_spec_semantics(spec: Spec) -> ValidationResult {
  let errors = []

  // Check required string fields are non-empty
  let errors = case string.is_empty(spec.name) {
    True -> [
      ValidationError(
        field: "name",
        rule: "non_empty_string",
        expected: "non-empty string",
        actual: "\"\"",
        explanation: "Spec name cannot be empty",
      ),
      ..errors
    ]
    False -> errors
  }

  let errors = case string.is_empty(spec.description) {
    True -> [
      ValidationError(
        field: "description",
        rule: "non_empty_string",
        expected: "non-empty string",
        actual: "\"\"",
        explanation: "Spec description cannot be empty",
      ),
      ..errors
    ]
    False -> errors
  }

  let errors = case string.is_empty(spec.version) {
    True -> [
      ValidationError(
        field: "version",
        rule: "non_empty_string",
        expected: "non-empty string",
        actual: "\"\"",
        explanation: "Spec version cannot be empty",
      ),
      ..errors
    ]
    False -> errors
  }

  // Check timeout_ms > 0
  let errors = case spec.config.timeout_ms > 0 {
    False -> [
      ValidationError(
        field: "config.timeout_ms",
        rule: "positive_integer",
        expected: "> 0",
        actual: int.to_string(spec.config.timeout_ms),
        explanation: "Timeout must be positive (non-zero) to prevent hangs",
      ),
      ..errors
    ]
    True -> errors
  }

  // Check features list is non-empty
  let errors = case list.is_empty(spec.features) {
    True -> [
      ValidationError(
        field: "features",
        rule: "non_empty_list",
        expected: "at least one feature",
        actual: "empty list",
        explanation: "Spec must define at least one feature",
      ),
      ..errors
    ]
    False -> errors
  }

  // Check behaviors list is non-empty for each feature
  let errors =
    list.fold(spec.features, errors, fn(acc, feature) {
      case list.is_empty(feature.behaviors) {
        True -> [
          ValidationError(
            field: "features." <> feature.name <> ".behaviors",
            rule: "non_empty_list",
            expected: "at least one behavior",
            actual: "empty list",
            explanation: "Features must define at least one behavior to test",
          ),
          ..acc
        ]
        False -> acc
      }
    })

  // Check string lengths
  let errors = case string.length(spec.name) > max_string_length {
    True -> [
      ValidationError(
        field: "name",
        rule: "max_length",
        expected: "< 100KB",
        actual: int.to_string(string.length(spec.name)) <> " bytes",
        explanation: "String length exceeds maximum allowed size",
      ),
      ..errors
    ]
    False -> errors
  }

  let errors = case string.length(spec.description) > max_string_length {
    True -> [
      ValidationError(
        field: "description",
        rule: "max_length",
        expected: "< 100KB",
        actual: int.to_string(string.length(spec.description)) <> " bytes",
        explanation: "String length exceeds maximum allowed size",
      ),
      ..errors
    ]
    False -> errors
  }

  // Check for path traversal in request paths
  let errors =
    list.fold(spec.features, errors, fn(acc, feature) {
      list.fold(feature.behaviors, acc, fn(inner_acc, behavior) {
        case string.contains(behavior.request.path, "../") {
          True -> [
            ValidationError(
              field: "features."
                <> feature.name
                <> ".behaviors."
                <> behavior.name
                <> ".request.path",
              rule: "no_path_traversal",
              expected: "path without \"../\"",
              actual: "\"" <> behavior.request.path <> "\"",
              explanation: "Path traversal sequences are not allowed for security",
            ),
            ..inner_acc
          ]
          False -> inner_acc
        }
      })
    })

  // Check for unique behavior names across all features
  let all_behaviors =
    list.fold(spec.features, [], fn(acc, feature) {
      list.append(feature.behaviors, acc)
    })

  let behavior_names = list.map(all_behaviors, fn(b) { b.name })
  let unique_names = list.unique(behavior_names)

  let errors = case list.length(behavior_names) != list.length(unique_names) {
    True -> [
      ValidationError(
        field: "behaviors",
        rule: "unique_names",
        expected: "unique behavior names across all features",
        actual: "duplicate behavior names found",
        explanation: "Behavior names must be unique across all features to enable unambiguous dependency resolution",
      ),
      ..errors
    ]
    False -> errors
  }

  // Return errors if any, otherwise Valid
  case list.is_empty(errors) {
    True -> Valid
    False -> Invalid(list.reverse(errors))
  }
}

/// Format validation errors for user display
pub fn format_validation_errors(errors: List(ValidationError)) -> String {
  let count = list.length(errors)
  let intro =
    "Spec validation failed with " <> int.to_string(count) <> " error(s):\n"

  let formatted =
    list.index_map(errors, fn(err, i) {
      let idx = i + 1
      int.to_string(idx)
      <> ". Field: '"
      <> err.field
      <> "'\n"
      <> "   Rule: "
      <> err.rule
      <> "\n"
      <> "   Expected: "
      <> err.expected
      <> "\n"
      <> "   Actual: "
      <> err.actual
      <> "\n"
      <> "   Explanation: "
      <> err.explanation
      <> "\n"
    })
    |> string.join("\n")

  intro <> formatted
}
