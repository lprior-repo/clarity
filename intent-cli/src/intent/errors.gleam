/// Rich error reporting with context and suggestions
/// Provides detailed feedback to help users understand and fix validation failures
import gleam/dict
import gleam/dynamic
import gleam/int
import gleam/json.{type Json}
import gleam/list
import gleam/string

/// Error with full context for better user feedback
pub type ContextualError {
  ContextualError(
    behavior: String,
    field_path: String,
    rule: String,
    expected: String,
    actual: String,
    available_fields: List(String),
    suggestions: List(String),
    explanation: String,
  )
}

/// Create a contextual error with field suggestions
pub fn field_not_found(
  behavior: String,
  field_path: String,
  available_fields: List(String),
) -> ContextualError {
  let suggestions = suggest_field_names(field_path, available_fields)
  ContextualError(
    behavior: behavior,
    field_path: field_path,
    rule: "present",
    expected: "field to exist",
    actual: "field missing",
    available_fields: available_fields,
    suggestions: suggestions,
    explanation: "Field '" <> field_path <> "' not found in response",
  )
}

/// Suggest similar field names based on Levenshtein distance
/// Helps users catch typos in field paths
fn suggest_field_names(target: String, available: List(String)) -> List(String) {
  available
  |> list.map(fn(field) { #(field, levenshtein_distance(target, field)) })
  |> list.filter(fn(pair) {
    let #(_field, dist) = pair
    dist <= 2
  })
  |> list.sort(fn(a, b) {
    let #(_af, ad) = a
    let #(_bf, bd) = b
    int.compare(ad, bd)
  })
  |> list.map(fn(pair) {
    let #(field, _dist) = pair
    field
  })
  |> list.take(3)
}

/// Calculate Levenshtein distance between two strings
/// Used for suggesting similar field names
fn levenshtein_distance(s1: String, s2: String) -> Int {
  let len1 = string.length(s1)
  let len2 = string.length(s2)

  case len1, len2 {
    0, _ -> len2
    _, 0 -> len1
    _, _ -> {
      // For simplicity, use character-level comparison
      // This is a basic approximation
      let chars1 = string.to_graphemes(s1)
      let chars2 = string.to_graphemes(s2)

      let common =
        list.filter(chars1, fn(c1) { list.contains(chars2, c1) })
        |> list.length

      len1 + len2 - 2 * common
    }
  }
}

/// Format contextual error for display
pub fn format_error(error: ContextualError) -> String {
  let field_info = "Field: '" <> error.field_path <> "'"

  let rule_info =
    "Rule: "
    <> error.rule
    <> "\n  Expected: "
    <> error.expected
    <> "\n  Actual: "
    <> error.actual

  let available_info = case error.available_fields {
    [] -> ""
    fields -> {
      "\n\nAvailable fields in response:\n  " <> string.join(fields, "\n  ")
    }
  }

  let suggestions_info = case error.suggestions {
    [] -> ""
    sugg -> {
      "\n\nDid you mean:\n  " <> string.join(sugg, "\n  ")
    }
  }

  "Behavior '"
  <> error.behavior
  <> "' validation failed:\n  "
  <> field_info
  <> "\n\n  "
  <> rule_info
  <> available_info
  <> suggestions_info
  <> "\n\n"
  <> error.explanation
}

/// Error type for validation results that collect all failures
pub type ValidationError {
  ValidationError(behavior: String, failures: List(FieldFailure))
}

pub type FieldFailure {
  FieldFailure(
    field: String,
    rule: String,
    expected: String,
    actual: String,
    explanation: String,
  )
}

/// Format validation error showing all failures at once
pub fn format_validation_error(error: ValidationError) -> String {
  let count = list.length(error.failures)
  let plural = case count {
    1 -> "failure"
    _ -> "failures"
  }

  let failure_lines =
    error.failures
    |> list.index_map(fn(failure, i) {
      let idx = i + 1
      "  "
      <> int.to_string(idx)
      <> ". Field '"
      <> failure.field
      <> "':\n"
      <> "     Rule: "
      <> failure.rule
      <> "\n"
      <> "     Expected: "
      <> failure.expected
      <> "\n"
      <> "     Actual: "
      <> failure.actual
    })
    |> string.join("\n\n")

  "Behavior '"
  <> error.behavior
  <> "' failed with "
  <> int.to_string(count)
  <> " "
  <> plural
  <> ":\n\n"
  <> failure_lines
}

/// Extract available fields from JSON object for suggestions
pub fn extract_available_fields(json: Json) -> List(String) {
  let json_str = json.to_string(json)
  // Try to decode as object
  case json.decode(json_str, dynamic.dict(dynamic.string, dynamic.dynamic)) {
    Ok(obj) ->
      dict.keys(obj)
      |> list.sort(string.compare)
    Error(_) -> []
  }
}

/// Format error message for format validation failures
pub fn format_format_error(
  field: String,
  format_name: String,
  value: String,
  reason: String,
) -> String {
  "Field '"
  <> field
  <> "':\n  Expected valid "
  <> format_name
  <> "\n  Got: "
  <> value
  <> "\n  Problem: "
  <> reason
}

/// Suggest next validation steps based on error pattern
pub fn suggest_next_steps(error_type: String) -> List(String) {
  case error_type {
    "format" -> [
      "Check that format validators parse correctly, not just regex match",
      "Add test cases with edge cases (leap years, invalid dates, etc.)",
    ]
    "missing_field" -> [
      "Verify the response structure matches the spec example",
      "Check if the field is nested deeper than expected",
      "Use 'array indexing' syntax if field is in an array",
    ]
    "interpolation" -> [
      "Ensure the variable is captured in a previous behavior",
      "Check variable name for typos",
      "Use 'intent validate' to catch undefined variables before execution",
    ]
    "circular_dependency" -> [
      "Review behavior 'requires' declarations for cycles",
      "Break circular dependencies by removing one requires link",
    ]
    _ -> [
      "Check the specification for ambiguities",
      "Run 'intent analyze' to improve spec quality",
    ]
  }
}
