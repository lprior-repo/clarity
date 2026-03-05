/// Spec linting - proactive detection of anti-patterns and quality issues
/// Checks behaviors for anti-patterns and quality issues
import gleam/dict
import gleam/dynamic
import gleam/float
import gleam/int
import gleam/json.{type Json}
import gleam/list
import gleam/string
import intent/types.{type AntiPattern, type Behavior, type Spec}

/// Linting result
pub type LintResult {
  LintValid
  LintWarnings(warnings: List(LintWarning))
}

/// A warning about the spec
pub type LintWarning {
  AntiPatternDetected(behavior: String, pattern_name: String, details: String)
  VagueVerification(behavior: String, verification: String, criteria: String)
  MissingExample(behavior: String)
  UnusedAntiPattern(pattern_name: String)
  NamingConvention(behavior: String, suggestion: String)
  DuplicateBehavior(behavior1: String, behavior2: String, similarity: String)
  MissingPreconditions(behavior: String)
  MissingPostconditions(behavior: String)
}

/// Lint a complete spec
pub fn lint_spec(spec: Spec) -> LintResult {
  let mut_warnings = []

  let behaviors =
    spec.features
    |> list.flat_map(fn(f) { f.behaviors })

  // Check for anti-patterns in verification examples
  let antipattern_warnings =
    behaviors
    |> list.flat_map(fn(behavior) {
      check_anti_patterns(behavior, spec.anti_patterns)
    })

  let mut_warnings = list.append(mut_warnings, antipattern_warnings)

  // Check for vague verifications
  let vague_warnings =
    behaviors
    |> list.flat_map(check_for_vague_verifications)

  let mut_warnings = list.append(mut_warnings, vague_warnings)

  // Check for missing examples in verifications
  let example_warnings =
    behaviors
    |> list.filter_map(fn(b) {
      case has_verification_examples(b) {
        True -> Error(Nil)
        False -> Ok(MissingExample(b.name))
      }
    })

  let mut_warnings = list.append(mut_warnings, example_warnings)

  // Check for naming conventions
  let naming_warnings =
    behaviors
    |> list.filter_map(check_naming_convention)

  let mut_warnings = list.append(mut_warnings, naming_warnings)

  // Check for duplicate behaviors
  let duplicate_warnings = check_for_duplicate_behaviors(behaviors)
  let mut_warnings = list.append(mut_warnings, duplicate_warnings)

  // Check for unused anti-patterns
  let used_patterns =
    behaviors
    |> list.flat_map(fn(b) {
      spec.anti_patterns
      |> list.filter(fn(ap) {
        has_verification_with_json(b)
        && contains_anti_pattern_keys(b, ap)
      })
      |> list.map(fn(ap) { ap.name })
    })
    |> list.unique

  let unused_warnings =
    spec.anti_patterns
    |> list.filter_map(fn(ap) {
      let name_lower = string.lowercase(ap.name)
      case string.contains(name_lower, "unused") {
        False -> Error(Nil)
        True ->
          case list.contains(used_patterns, ap.name) {
            True -> Error(Nil)
            False -> Ok(UnusedAntiPattern(ap.name))
          }
      }
    })

  let mut_warnings = list.append(mut_warnings, unused_warnings)

  case list.is_empty(mut_warnings) {
    True -> LintValid
    False -> LintWarnings(mut_warnings)
  }
}

/// Check if behavior has verification examples
fn has_verification_examples(behavior: Behavior) -> Bool {
  list.any(behavior.verifications, fn(v) { !list.is_empty(v.examples) })
}

/// Check if behavior has any verification with JSON data
fn has_verification_with_json(behavior: Behavior) -> Bool {
  list.any(behavior.verifications, fn(v) { !list.is_empty(v.examples) })
}

/// Check for anti-patterns in a behavior's verifications
fn check_anti_patterns(
  behavior: Behavior,
  patterns: List(AntiPattern),
) -> List(LintWarning) {
  case has_verification_with_json(behavior) {
    False -> []
    True ->
      patterns
      |> list.filter_map(fn(pattern) {
        case contains_anti_pattern_keys(behavior, pattern) {
          False -> Error(Nil)
          True ->
            Ok(AntiPatternDetected(
              behavior.name,
              pattern.name,
              "Verification examples contain keys from anti-pattern: "
                <> pattern.description,
            ))
        }
      })
  }
}

/// Check if a behavior contains the bad pattern keys
fn contains_anti_pattern_keys(behavior: Behavior, pattern: AntiPattern) -> Bool {
  let bad_keys = extract_all_keys(pattern.bad_example)

  // Check all verification examples
  list.any(behavior.verifications, fn(v) {
    list.any(v.examples, fn(example) {
      let example_keys = extract_all_keys(example)
      list.any(bad_keys, fn(key) { list.contains(example_keys, key) })
    })
  })
}

/// Extract all keys from a JSON object (recursively)
fn extract_all_keys(json: Json) -> List(String) {
  let json_str = json.to_string(json)

  case json.decode(json_str, dynamic.dict(dynamic.string, dynamic.dynamic)) {
    Ok(obj) -> {
      dict.keys(obj)
    }
    Error(_) -> []
  }
}

/// Check for vague verifications in a behavior
fn check_for_vague_verifications(behavior: Behavior) -> List(LintWarning) {
  behavior.verifications
  |> list.filter_map(fn(verification) {
    let has_vague =
      list.any(verification.criteria, fn(criterion) {
        let criterion_lower = string.lowercase(criterion)

        let has_valid_keyword = string.contains(criterion_lower, "valid")
        let has_email_keyword = string.contains(criterion_lower, "email")
        let has_uuid_keyword = string.contains(criterion_lower, "uuid")
        let has_iso_keyword = string.contains(criterion_lower, "iso")
        let has_jwt_keyword = string.contains(criterion_lower, "jwt")
        let has_uri_keyword = string.contains(criterion_lower, "uri")
        let has_correct_format = string.contains(criterion_lower, "correct format")
        let has_proper_format = string.contains(criterion_lower, "proper format")

        {
          has_valid_keyword
          && !has_email_keyword
          && !has_uuid_keyword
          && !has_iso_keyword
          && !has_jwt_keyword
          && !has_uri_keyword
        }
        || has_correct_format
        || has_proper_format
      })

    case has_vague {
      False -> Error(Nil)
      True ->
        Ok(VagueVerification(
          behavior.name,
          verification.description,
          "contains vague criteria (be specific)",
        ))
    }
  })
}

/// Check naming conventions for behaviors
fn check_naming_convention(behavior: Behavior) -> Result(LintWarning, Nil) {
  // Suggest kebab-case for behavior names
  case has_invalid_name_chars(behavior.name) {
    False -> Error(Nil)
    True ->
      Ok(NamingConvention(
        behavior.name,
        "Use kebab-case for behavior names (e.g., 'create-user-by-id')",
      ))
  }
}

/// Check if a name has invalid characters (not alphanumeric, hyphen, underscore)
fn has_invalid_name_chars(name: String) -> Bool {
  string.to_graphemes(name)
  |> list.any(fn(c) {
    case string.contains("abcdefghijklmnopqrstuvwxyz0123456789-_", c) {
      True -> False
      False -> True
    }
  })
}

/// Check for duplicate or similar behaviors
fn check_for_duplicate_behaviors(behaviors: List(Behavior)) -> List(LintWarning) {
  // Compare each behavior with all others
  behaviors
  |> list.index_map(fn(behavior, idx) {
    behaviors
    |> list.drop(idx + 1)
    |> list.filter_map(fn(other) {
      let similarity = calculate_behavior_similarity(behavior, other)
      case similarity >. 0.7 {
        True ->
          Ok(DuplicateBehavior(
            behavior.name,
            other.name,
            "Similar name and intent (similarity: "
              <> string.trim(float_to_string(similarity, 2))
              <> ")",
          ))
        False -> Error(Nil)
      }
    })
  })
  |> list.flatten
}

/// Calculate similarity between two behaviors
fn calculate_behavior_similarity(b1: Behavior, b2: Behavior) -> Float {
  // Check name similarity
  let name_similarity =
    calculate_string_similarity(
      string.lowercase(b1.name),
      string.lowercase(b2.name),
    )

  // Check intent similarity
  let intent_similarity =
    calculate_string_similarity(
      string.lowercase(b1.intent),
      string.lowercase(b2.intent),
    )

  // Average the two
  { name_similarity +. intent_similarity } /. 2.0
}

/// Calculate string similarity (simple Levenshtein-based approach)
fn calculate_string_similarity(s1: String, s2: String) -> Float {
  case s1 == s2 {
    True -> 1.0
    False -> {
      let len1 = string.length(s1)
      let len2 = string.length(s2)
      let max_len = int.max(len1, len2)

      case max_len {
        0 -> 1.0
        _ -> {
          let common = count_common_substrings(s1, s2)
          let float_common = int.to_float(common)
          let float_max = int.to_float(max_len)
          float_common /. float_max
        }
      }
    }
  }
}

/// Count common substrings between two strings
fn count_common_substrings(s1: String, s2: String) -> Int {
  // Simple approach: count matching characters in order
  let g1 = string.to_graphemes(s1)
  let g2 = string.to_graphemes(s2)

  list.zip(g1, g2)
  |> list.filter(fn(pair) { pair.0 == pair.1 })
  |> list.length
}

/// Convert float to string with precision
fn float_to_string(f: Float, _precision: Int) -> String {
  // Simple implementation - just convert to string
  float.to_string(f)
}

/// Format lint warnings for display
pub fn format_warnings(warnings: List(LintWarning)) -> String {
  let warning_lines =
    warnings
    |> list.map(format_warning)
    |> string.join("\n")

  "Linting found "
  <> int.to_string(list.length(warnings))
  <> " warning(s):\n\n"
  <> warning_lines
}

/// Format a single lint warning
fn format_warning(warning: LintWarning) -> String {
  case warning {
    AntiPatternDetected(behavior, pattern, details) ->
      "Behavior '"
      <> behavior
      <> "':\n"
      <> "  Anti-pattern: "
      <> pattern
      <> "\n"
      <> "  "
      <> details

    VagueVerification(behavior, verification, criteria) ->
      "Behavior '"
      <> behavior
      <> "', verification '"
      <> verification
      <> "':\n"
      <> "  "
      <> criteria

    MissingExample(behavior) ->
      "Behavior '"
      <> behavior
      <> "':\n"
      <> "  Missing verification examples (helps AI understand intent)"

    UnusedAntiPattern(pattern) ->
      "Anti-pattern '" <> pattern <> "' is not tested by any behavior"

    NamingConvention(behavior, suggestion) ->
      "Behavior '" <> behavior <> "':\n" <> "  " <> suggestion

    DuplicateBehavior(behavior1, behavior2, similarity) ->
      "Behaviors '"
      <> behavior1
      <> "' and '"
      <> behavior2
      <> "' may be duplicates:\n"
      <> "  "
      <> similarity
      <> " - consider consolidating"

    MissingPreconditions(behavior) ->
      "Behavior '" <> behavior <> "':\n" <> "  Missing preconditions"

    MissingPostconditions(behavior) ->
      "Behavior '" <> behavior <> "':\n" <> "  Missing postconditions"
  }
}
