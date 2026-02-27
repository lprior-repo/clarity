/// Dedicated spec validation command
/// Provides comprehensive CUE spec validation with detailed error reporting
import gleam/dict
import gleam/dynamic
import gleam/int
import gleam/json
import gleam/list
import gleam/result
import gleam/string
import intent/parser
import intent/types.{type Spec}
import shellout
import simplifile

/// Validation result type
pub type ValidationResult {
  ValidationValid
  ValidationInvalid(errors: List(ValidationError))
}

/// Validation error types with location information
pub type ValidationError {
  CueSyntaxError(line: Int, message: String)
  SchemaError(path: String, message: String)
  DuplicateBehaviorName(name: String, count: Int)
  CircularDependency(cycle: List(String))
  MissingBehaviorReference(
    behavior: String,
    reference_type: String,
    missing_name: String,
  )
  EmptyFeatureList
  EmptyBehaviorList(feature: String)
  InvalidBehaviorName(name: String, reason: String)
  MissingRequiredField(field: String)
  InvalidIdentifier(name: String, context: String)
}

/// Validation report for output
pub type ValidationReport {
  ValidationReport(
    valid: Bool,
    spec_name: String,
    total_behaviors: Int,
    total_features: Int,
    errors: List(ValidationError),
    warnings: List(String),
  )
}

/// Main validation entry point - validates a CUE spec file
pub fn validate_spec_file(spec_path: String) -> ValidationResult {
  // First check if file exists
  case simplifile.verify_is_file(spec_path) {
    Ok(False) ->
      ValidationInvalid([CueSyntaxError(0, "File not found: " <> spec_path)])
    Error(_) ->
      ValidationInvalid([CueSyntaxError(0, "Cannot access file: " <> spec_path)])
    Ok(True) -> {
      // Step 1: Validate CUE syntax
      case validate_cue_syntax(spec_path) {
        Error(e) -> ValidationInvalid([e])
        Ok(_) -> {
          // Step 2: Export and parse spec
          case load_and_parse_spec(spec_path) {
            Error(errors) -> ValidationInvalid(errors)
            Ok(spec) -> validate_spec_structure(spec)
          }
        }
      }
    }
  }
}

/// Validate CUE syntax using cue vet
fn validate_cue_syntax(spec_path: String) -> Result(Nil, ValidationError) {
  case shellout.command("cue", ["vet", spec_path], ".", []) {
    Ok(_) -> Ok(Nil)
    Error(#(_, stderr)) -> {
      // Parse CUE error to extract line number and message
      let #(line_num, message) = parse_cue_error(stderr)
      Error(CueSyntaxError(line_num, message))
    }
  }
}

/// Parse CUE error output to extract line and message
fn parse_cue_error(stderr: String) -> #(Int, String) {
  // CUE error format: "file.cue:5: error message"
  let lines = string.split(stderr, "\n")

  case lines {
    [] -> #(0, stderr)
    [first_line, ..] -> {
      case string.split(first_line, ":") {
        [_, line_str, ..rest] -> {
          case int.parse(line_str) {
            Ok(line_num) -> #(line_num, string.join(rest, ":"))
            Error(_) -> #(0, first_line)
          }
        }
        _ -> #(0, first_line)
      }
    }
  }
}

/// Load and parse spec from CUE file (exposed for summary display)
pub fn load_and_parse_spec(
  spec_path: String,
) -> Result(Spec, List(ValidationError)) {
  case shellout.command("cue", ["export", spec_path, "-e", "spec"], ".", []) {
    Ok(json_str) -> {
      case json.decode(json_str, dynamic.dynamic) {
        Ok(json_data) -> {
          case parser.decode_dynamic(json_data) {
            Ok(spec) -> Ok(spec)
            Error(decode_errors) -> {
              let errors =
                list.map(decode_errors, fn(e) {
                  SchemaError(
                    string.join(e.path, "."),
                    "Expected " <> e.expected <> " but found " <> e.found,
                  )
                })
              Error(errors)
            }
          }
        }
        Error(_) ->
          Error([SchemaError("root", "Invalid JSON output from CUE export")])
      }
    }
    Error(#(_, stderr)) -> {
      Error([CueSyntaxError(0, "CUE export failed: " <> stderr)])
    }
  }
}

/// Validate spec structure and semantics
fn validate_spec_structure(spec: Spec) -> ValidationResult {
  let mut_errors = []

  // Check for empty features list
  case spec.features {
    [] -> list.append(mut_errors, [EmptyFeatureList])
    _ -> mut_errors
  }

  // Collect all behaviors
  let all_behaviors =
    spec.features
    |> list.flat_map(fn(f) { f.behaviors })

  let behavior_names =
    all_behaviors
    |> list.map(fn(b) { b.name })

  // Validate each feature has behaviors
  let feature_errors =
    spec.features
    |> list.flat_map(fn(feature) {
      case feature.behaviors {
        [] -> [EmptyBehaviorList(feature.name)]
        _ -> []
      }
    })

  let mut_errors = list.append(mut_errors, feature_errors)

  // Check for duplicate behavior names
  let duplicate_errors = check_duplicate_behaviors(behavior_names)
  let mut_errors = list.append(mut_errors, duplicate_errors)

  // Validate behavior names match identifier pattern
  let name_errors =
    all_behaviors
    |> list.flat_map(fn(b) {
      case is_valid_identifier(b.name) {
        True -> []
        False -> [
          InvalidBehaviorName(b.name, "must match pattern [a-z][a-z0-9_-]*"),
        ]
      }
    })

  let mut_errors = list.append(mut_errors, name_errors)

  // Check for circular dependencies
  let circular_errors = check_circular_dependencies(all_behaviors)
  let mut_errors = list.append(mut_errors, circular_errors)

  // Validate behavior references in requires
  let ref_errors = validate_behavior_references(all_behaviors, behavior_names)
  let mut_errors = list.append(mut_errors, ref_errors)

  // Return result
  case list.is_empty(mut_errors) {
    True -> ValidationValid
    False -> ValidationInvalid(mut_errors)
  }
}

/// Check if string is a valid identifier
fn is_valid_identifier(name: String) -> Bool {
  case name {
    "" -> False
    _ -> {
      let first = string.first(name)
      let graphemes = string.to_graphemes(name)
      let rest = case graphemes {
        [] -> []
        [_, ..rest] -> rest
      }

      case first {
        Ok(first_char) -> {
          let is_lowercase_letter =
            "abcdefghijklmnopqrstuvwxyz"
            |> string.to_graphemes()
            |> list.contains(first_char)

          case is_lowercase_letter {
            False -> False
            True -> {
              // Check rest contains only valid chars
              rest == []
              || rest
              |> list.all(fn(g) {
                let valid_chars =
                  "abcdefghijklmnopqrstuvwxyz0123456789_-"
                  |> string.to_graphemes()
                list.contains(valid_chars, g)
              })
            }
          }
        }
        Error(_) -> False
      }
    }
  }
}

/// Check for duplicate behavior names
fn check_duplicate_behaviors(
  behavior_names: List(String),
) -> List(ValidationError) {
  let grouped =
    behavior_names
    |> list.group(fn(x) { x })

  grouped
  |> dict.to_list
  |> list.filter_map(fn(pair) {
    let #(name, occurrences) = pair
    let count = list.length(occurrences)

    case count > 1 {
      False -> Error(Nil)
      True -> Ok(DuplicateBehaviorName(name, count))
    }
  })
}

/// Check for circular dependencies using DFS
fn check_circular_dependencies(
  behaviors: List(types.Behavior),
) -> List(ValidationError) {
  let behavior_map =
    behaviors
    |> list.map(fn(b) { #(b.name, b.requires) })
    |> dict.from_list

  let checked = []

  behaviors
  |> list.filter_map(fn(b) {
    case list.contains(checked, b.name) {
      True -> Error(Nil)
      // Already checked
      False -> {
        case detect_cycle(b.name, [], behavior_map) {
          Ok(cycle) -> Ok(CircularDependency(cycle))
          Error(Nil) -> Error(Nil)
        }
      }
    }
  })
}

/// Detect cycle using DFS
fn detect_cycle(
  current: String,
  path: List(String),
  behavior_map: dict.Dict(String, List(String)),
) -> Result(List(String), Nil) {
  case list.contains(path, current) {
    True -> {
      // Found cycle - extract the cycle portion
      // Find index manually
      let idx =
        list.index_map(path, fn(x, i) { #(x, i) })
        |> list.find(fn(pair) { pair.0 == current })
        |> result.map(fn(pair) { pair.1 })
        |> result.unwrap(0)

      let cycle = list.drop(path, idx) |> list.append([current])
      Ok(cycle)
    }
    False ->
      case dict.get(behavior_map, current) {
        Ok(deps) -> check_deps(deps, list.append(path, [current]), behavior_map)
        Error(_) -> Error(Nil)
        // No dependencies
      }
  }
}

/// Check all dependencies for cycles
fn check_deps(
  deps: List(String),
  path: List(String),
  behavior_map: dict.Dict(String, List(String)),
) -> Result(List(String), Nil) {
  case deps {
    [] -> Error(Nil)
    [dep, ..rest] ->
      case detect_cycle(dep, path, behavior_map) {
        Ok(cycle) -> Ok(cycle)
        Error(_) -> check_deps(rest, path, behavior_map)
      }
  }
}

/// Validate that referenced behaviors exist
fn validate_behavior_references(
  behaviors: List(types.Behavior),
  all_names: List(String),
) -> List(ValidationError) {
  behaviors
  |> list.flat_map(fn(b) {
    b.requires
    |> list.filter_map(fn(required_name) {
      case list.contains(all_names, required_name) {
        True -> Error(Nil)
        False -> Ok(MissingBehaviorReference(b.name, "requires", required_name))
      }
    })
  })
}

/// Generate a validation report
pub fn generate_report(
  spec_path: String,
  result: ValidationResult,
) -> ValidationReport {
  case result {
    ValidationValid -> {
      // Load spec to get metadata
      case load_and_parse_spec(spec_path) {
        Ok(spec) -> {
          let total_behaviors =
            spec.features
            |> list.flat_map(fn(f) { f.behaviors })
            |> list.length

          ValidationReport(
            valid: True,
            spec_name: spec.name,
            total_behaviors: total_behaviors,
            total_features: list.length(spec.features),
            errors: [],
            warnings: [],
          )
        }
        Error(_) -> {
          ValidationReport(
            valid: True,
            spec_name: "Unknown",
            total_behaviors: 0,
            total_features: 0,
            errors: [],
            warnings: [],
          )
        }
      }
    }
    ValidationInvalid(errors) -> {
      // Try to load spec for metadata
      case load_and_parse_spec(spec_path) {
        Ok(spec) -> {
          let total_behaviors =
            spec.features
            |> list.flat_map(fn(f) { f.behaviors })
            |> list.length

          ValidationReport(
            valid: False,
            spec_name: spec.name,
            total_behaviors: total_behaviors,
            total_features: list.length(spec.features),
            errors: errors,
            warnings: [],
          )
        }
        Error(_) -> {
          ValidationReport(
            valid: False,
            spec_name: "Unknown",
            total_behaviors: 0,
            total_features: 0,
            errors: errors,
            warnings: [],
          )
        }
      }
    }
  }
}

/// Format validation result as human-readable text
pub fn format_validation_result(result: ValidationResult) -> String {
  case result {
    ValidationValid -> "✓ Spec is valid\n"
    ValidationInvalid(errors) -> {
      let error_lines =
        errors
        |> list.map(format_error)
        |> string.join("\n\n")

      "✗ Spec validation failed with "
      <> int.to_string(list.length(errors))
      <> " error(s):\n\n"
      <> error_lines
      <> "\n"
    }
  }
}

/// Format a single validation error
fn format_error(error: ValidationError) -> String {
  case error {
    CueSyntaxError(line, message) -> {
      "CUE Syntax Error (line " <> int.to_string(line) <> "):\n  " <> message
    }
    SchemaError(path, message) -> {
      "Schema Error at '" <> path <> "':\n  " <> message
    }
    DuplicateBehaviorName(name, count) -> {
      "Duplicate behavior name '"
      <> name
      <> "' found "
      <> int.to_string(count)
      <> " times\n  Each behavior must have a unique name"
    }
    CircularDependency(cycle) -> {
      "Circular dependency detected:\n  " <> string.join(cycle, " → ")
    }
    MissingBehaviorReference(behavior, ref_type, missing) -> {
      "Behavior '"
      <> behavior
      <> "' references non-existent "
      <> ref_type
      <> ": '"
      <> missing
      <> "'"
    }
    EmptyFeatureList -> {
      "Spec must contain at least one feature"
    }
    EmptyBehaviorList(feature) -> {
      "Feature '" <> feature <> "' must contain at least one behavior"
    }
    InvalidBehaviorName(name, reason) -> {
      "Invalid behavior name '" <> name <> "': " <> reason
    }
    MissingRequiredField(field) -> {
      "Missing required field: " <> field
    }
    InvalidIdentifier(name, context) -> {
      "Invalid identifier '" <> name <> "' in " <> context
    }
  }
}

/// Format validation result as JSON
pub fn format_validation_result_json(result: ValidationResult) -> String {
  case result {
    ValidationValid -> {
      json.object([
        #("valid", json.bool(True)),
        #("errors", json.array([], fn(_) { json.null() })),
      ])
      |> json.to_string()
    }
    ValidationInvalid(errors) -> {
      let errors_json =
        errors
        |> list.map(fn(e) {
          json.object([
            #("type", json.string(error_type_to_string(e))),
            ..error_to_json_fields(e)
          ])
        })

      json.object([
        #("valid", json.bool(False)),
        #("error_count", json.int(list.length(errors))),
        #("errors", json.array(errors_json, fn(x) { x })),
      ])
      |> json.to_string()
    }
  }
}

/// Get error type string for JSON output
fn error_type_to_string(error: ValidationError) -> String {
  case error {
    CueSyntaxError(_, _) -> "cue_syntax_error"
    SchemaError(_, _) -> "schema_error"
    DuplicateBehaviorName(_, _) -> "duplicate_behavior_name"
    CircularDependency(_) -> "circular_dependency"
    MissingBehaviorReference(_, _, _) -> "missing_behavior_reference"
    EmptyFeatureList -> "empty_feature_list"
    EmptyBehaviorList(_) -> "empty_behavior_list"
    InvalidBehaviorName(_, _) -> "invalid_behavior_name"
    MissingRequiredField(_) -> "missing_required_field"
    InvalidIdentifier(_, _) -> "invalid_identifier"
  }
}

/// Convert error to JSON fields
fn error_to_json_fields(error: ValidationError) -> List(#(String, json.Json)) {
  case error {
    CueSyntaxError(line, message) -> [
      #("line", json.int(line)),
      #("message", json.string(message)),
    ]
    SchemaError(path, message) -> [
      #("path", json.string(path)),
      #("message", json.string(message)),
    ]
    DuplicateBehaviorName(name, count) -> [
      #("name", json.string(name)),
      #("count", json.int(count)),
    ]
    CircularDependency(cycle) -> [#("cycle", json.array(cycle, json.string))]
    MissingBehaviorReference(behavior, ref_type, missing) -> [
      #("behavior", json.string(behavior)),
      #("reference_type", json.string(ref_type)),
      #("missing", json.string(missing)),
    ]
    EmptyFeatureList -> []
    EmptyBehaviorList(feature) -> [#("feature", json.string(feature))]
    InvalidBehaviorName(name, reason) -> [
      #("name", json.string(name)),
      #("reason", json.string(reason)),
    ]
    MissingRequiredField(field) -> [#("field", json.string(field))]
    InvalidIdentifier(name, context) -> [
      #("name", json.string(name)),
      #("context", json.string(context)),
    ]
  }
}

/// Helper function to find index of element in list
fn find_index_in_list(list: List(a), target: a, default: Int) -> Int {
  case list {
    [] -> default
    [first, ..rest] -> {
      case first == target {
        True -> 0
        False -> 1 + find_index_in_list(rest, target, default)
      }
    }
  }
}
