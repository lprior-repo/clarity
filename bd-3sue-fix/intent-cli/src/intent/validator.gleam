/// Pre-execution static validation of specs
/// Validates declarative behavior specifications and dependencies
import gleam/dict
import gleam/dynamic
import gleam/int
import gleam/json
import gleam/list
import gleam/string
import intent/types.{type Behavior, type Spec}

/// Result of pre-execution validation
pub type ValidationResult {
  ValidationValid
  ValidationInvalid(issues: List(ValidationIssue))
}

/// Issues found during validation
pub type ValidationIssue {
  MissingDependency(behavior: String, depends_on: String)
  CircularDependency(behaviors: List(String))
  DuplicateBehaviorName(name: String, features: List(String))
  EmptyVerificationList(behavior: String)
  EmptyPreconditions(behavior: String)
  InvalidJsonInExample(behavior: String, verification_index: Int, error: String)
  EmptyBehaviorName(feature: String, behavior_index: Int)
  EmptyIntent(behavior: String)
}

/// Validate a complete spec before execution
pub fn validate_spec(spec: Spec) -> ValidationResult {
  let mut_issues = []

  // Collect all behaviors with their names
  let all_behaviors =
    spec.features
    |> list.flat_map(fn(feature) { feature.behaviors })

  let behavior_names =
    all_behaviors
    |> list.map(fn(b) { b.name })

  // Validate each behavior
  let behavior_issues =
    all_behaviors
    |> list.flat_map(fn(behavior) {
      validate_behavior(behavior, behavior_names, all_behaviors)
    })

  let mut_issues = list.append(mut_issues, behavior_issues)

  // Check for circular dependencies
  let circular_issues = check_circular_dependencies(all_behaviors)
  let mut_issues = list.append(mut_issues, circular_issues)

  // Check for duplicate behavior names
  let duplicate_issues = check_duplicate_behavior_names(spec.features)
  let mut_issues = list.append(mut_issues, duplicate_issues)

  case list.is_empty(mut_issues) {
    True -> ValidationValid
    False -> ValidationInvalid(mut_issues)
  }
}

/// Validate a single behavior
fn validate_behavior(
  behavior: Behavior,
  all_behavior_names: List(String),
  _all_behaviors: List(Behavior),
) -> List(ValidationIssue) {
  let mut_issues = []

  // Check that behavior name is not empty
  let name_issues =
    case string.is_empty(behavior.name) {
      True -> [EmptyBehaviorName("", 0)]
      False -> []
    }

  let mut_issues = list.append(mut_issues, name_issues)

  // Check that intent is not empty
  let intent_issues =
    case string.is_empty(behavior.intent) {
      True -> [EmptyIntent(behavior.name)]
      False -> []
    }

  let mut_issues = list.append(mut_issues, intent_issues)

  // Validate that all dependencies exist
  let dep_issues =
    behavior.requires
    |> list.filter_map(fn(dep_name) {
      case list.contains(all_behavior_names, dep_name) {
        True -> Error(Nil)
        False -> Ok(MissingDependency(behavior.name, dep_name))
      }
    })

  let mut_issues = list.append(mut_issues, dep_issues)

  // Validate JSON syntax in verification examples
  let json_issues = validate_verification_examples(behavior)
  let mut_issues = list.append(mut_issues, json_issues)

  mut_issues
}

/// Validate JSON syntax in verification examples
fn validate_verification_examples(
  behavior: Behavior,
) -> List(ValidationIssue) {
  behavior.verifications
  |> list.index_map(fn(verification, index) {
    verification.examples
    |> list.index_map(fn(example, example_index) {
      // Try to stringify and re-parse the JSON to validate syntax
      let json_string = json.to_string(example)

      // Try to parse it back to verify it's valid JSON
      case json.decode(json_string, dynamic.dynamic) {
        Ok(_) -> Error(Nil)
        Error(_err) ->
          Ok(InvalidJsonInExample(
            behavior.name,
            index,
            "Verification #"
              <> int.to_string(index + 1)
              <> ", example #"
              <> int.to_string(example_index + 1)
              <> ": JSON syntax validation failed",
          ))
      }
    })
  })
  |> list.concat
  |> list.filter_map(fn(result) { result })
}

/// Check for circular dependencies
fn check_circular_dependencies(
  behaviors: List(Behavior),
) -> List(ValidationIssue) {
  behaviors
  |> list.filter_map(fn(behavior) {
    case has_circular_dependency(behavior.name, [], behaviors) {
      True -> Ok(CircularDependency([behavior.name]))
      False -> Error(Nil)
    }
  })
}

/// Check if a behavior has circular dependency
fn has_circular_dependency(
  behavior_name: String,
  visited: List(String),
  all_behaviors: List(Behavior),
) -> Bool {
  has_circular_loop(behavior_name, visited, all_behaviors)
}

/// Tail-recursive circular dependency check
fn has_circular_loop(
  behavior_name: String,
  visited: List(String),
  all_behaviors: List(Behavior),
) -> Bool {
  case list.contains(visited, behavior_name) {
    True -> True
    False ->
      case list.find(all_behaviors, fn(b) { b.name == behavior_name }) {
        Error(_) -> False
        Ok(behavior) ->
          has_circular_loop_deps(
            behavior.requires,
            list.append(visited, [behavior_name]),
            all_behaviors,
          )
      }
  }
}

/// Tail-recursive helper to check all dependencies
fn has_circular_loop_deps(
  deps: List(String),
  visited: List(String),
  all_behaviors: List(Behavior),
) -> Bool {
  case deps {
    [] -> False
    [dep, ..rest] ->
      case has_circular_loop(dep, visited, all_behaviors) {
        True -> True
        False -> has_circular_loop_deps(rest, visited, all_behaviors)
      }
  }
}

/// Check for duplicate behavior names across features
fn check_duplicate_behavior_names(
  features: List(types.Feature),
) -> List(ValidationIssue) {
  // Build a list of all (behavior_name, feature_name) pairs
  let behavior_feature_pairs =
    features
    |> list.flat_map(fn(feature) {
      feature.behaviors
      |> list.map(fn(behavior) { #(behavior.name, feature.name) })
    })

  // Group by behavior name and find duplicates
  let grouped =
    behavior_feature_pairs
    |> list.group(fn(pair) { pair.0 })

  // Convert to list and filter for duplicates
  grouped
  |> dict.to_list
  |> list.filter_map(fn(entry) {
    let #(behavior_name, pairs) = entry
    let feature_names = pairs |> list.map(fn(p) { p.1 })

    case list.length(feature_names) > 1 {
      False -> Error(Nil)
      True -> Ok(DuplicateBehaviorName(behavior_name, feature_names))
    }
  })
}

/// Format validation issues for display
pub fn format_issues(issues: List(ValidationIssue)) -> String {
  let issue_lines =
    issues
    |> list.map(format_issue)
    |> string.join("\n\n")

  "Validation failed with "
  <> int.to_string(list.length(issues))
  <> " issue(s):\n\n"
  <> issue_lines
}

/// Format a single validation issue
fn format_issue(issue: ValidationIssue) -> String {
  case issue {
    MissingDependency(behavior, depends_on) ->
      "Behavior '"
      <> behavior
      <> "':\n"
      <> "  Depends on behavior '"
      <> depends_on
      <> "' which does not exist"

    CircularDependency(behaviors) ->
      "Circular dependency detected:\n"
      <> "  Behaviors: "
      <> string.join(behaviors, " -> ")

    DuplicateBehaviorName(name, features) ->
      "Duplicate behavior name '"
      <> name
      <> "' found in multiple features:\n"
      <> "  Features: "
      <> string.join(features, ", ")
      <> "\n"
      <> "  Each behavior must have a unique name across all features"

    EmptyVerificationList(behavior) ->
      "Behavior '"
      <> behavior
      <> "':\n"
      <> "  Has no verifications defined\n"
      <> "  Add at least one verification to demonstrate correct behavior"

    EmptyPreconditions(behavior) ->
      "Behavior '"
      <> behavior
      <> "':\n"
      <> "  Has no preconditions defined\n"
      <> "  Add preconditions to specify what must be true before execution"

    InvalidJsonInExample(behavior, verification_index, error) ->
      "Behavior '"
      <> behavior
      <> "', verification #"
      <> int.to_string(verification_index + 1)
      <> ":\n"
      <> "  Invalid JSON in example\n"
      <> "  Error: "
      <> error

    EmptyBehaviorName(feature, index) ->
      "Feature '"
      <> feature
      <> "', behavior #"
      <> int.to_string(index + 1)
      <> ":\n"
      <> "  Behavior name is empty\n"
      <> "  Each behavior must have a unique, non-empty name"

    EmptyIntent(behavior) ->
      "Behavior '"
      <> behavior
      <> "':\n"
      <> "  Intent field is empty\n"
      <> "  Add a clear intent description explaining what this behavior demonstrates"
  }
}
