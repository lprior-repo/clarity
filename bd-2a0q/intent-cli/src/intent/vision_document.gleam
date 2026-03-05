/// Vision Document Generator
/// Generates comprehensive vision documents from specifications
import gleam/dict
import gleam/json
import gleam/list
import gleam/string
import intent/types.{type Behavior, type Feature, type Spec}

/// Generate a vision document from a spec
pub fn generate_vision_document(spec: Spec) -> String {
  let title = "# Vision: " <> spec.name <> "\n\n"
  let overview = generate_overview(spec)
  let success_criteria = generate_success_criteria(spec)
  let features = generate_features(spec)
  let invariants = generate_invariants(spec)
  let anti_patterns = generate_anti_patterns(spec)
  let technical_considerations = generate_technical_considerations(spec)

  title
  <> overview
  <> "\n"
  <> success_criteria
  <> "\n"
  <> features
  <> "\n"
  <> invariants
  <> "\n"
  <> anti_patterns
  <> "\n"
  <> technical_considerations
}

fn generate_overview(spec: Spec) -> String {
  let version = "## Version\n\n" <> spec.version <> "\n\n"
  let description = "## Description\n\n" <> spec.description <> "\n\n"
  let audience = "## Target Audience\n\n" <> spec.audience <> "\n\n"

  "## Overview\n\n" <> version <> description <> audience
}

fn generate_success_criteria(spec: Spec) -> String {
  let criteria_items =
    spec.success_criteria
    |> list.map(fn(criteria) { "- " <> criteria })
    |> string.join("\n")

  "## Success Criteria\n\n" <> criteria_items <> "\n"
}

fn generate_features(spec: Spec) -> String {
  let features_content =
    spec.features
    |> list.map(generate_feature_section)
    |> string.join("\n\n")

  "## Features\n\n" <> features_content
}

fn generate_feature_section(feature: Feature) -> String {
  let header = "### " <> feature.name <> "\n\n"
  let description = feature.description <> "\n\n"
  let behaviors = generate_behaviors(feature.behaviors)

  header <> description <> behaviors
}

fn generate_behaviors(behaviors) -> String {
  case behaviors {
    [] -> ""
    _ -> {
      let behavior_list =
        behaviors
        |> list.map(generate_behavior_summary)
        |> string.join("\n")

      "#### Behaviors\n\n" <> behavior_list <> "\n"
    }
  }
}

fn generate_behavior_summary(behavior: Behavior) -> String {
  let name = "**" <> behavior.name <> "**"
  let intent = ": " <> behavior.intent

  let notes = case behavior.notes {
    "" -> ""
    notes -> "\n  > " <> notes
  }

  let preconditions = case behavior.preconditions {
    [] -> ""
    preconditions -> {
      let pre = string.join(preconditions, ", ")
      "\n  - Preconditions: " <> pre
    }
  }

  let postconditions = case behavior.postconditions {
    [] -> ""
    postconditions -> {
      let post = string.join(postconditions, ", ")
      "\n  - Postconditions: " <> post
    }
  }

  let requires = case behavior.requires {
    [] -> ""
    requires -> {
      let deps = string.join(requires, ", ")
      "\n  - Requires: " <> deps
    }
  }

  let tags = case behavior.tags {
    [] -> ""
    tags -> {
      let tag_list = string.join(tags, ", ")
      "\n  - Tags: " <> tag_list
    }
  }

  "- " <> name <> intent <> notes <> preconditions <> postconditions <> requires <> tags
}

fn generate_invariants(spec: Spec) -> String {
  case spec.invariants {
    [] -> ""
    invariants -> {
      let invariant_items =
        invariants
        |> list.map(fn(invariant) {
          let name = "### " <> invariant.name
          let description = invariant.description
          let criteria = string.join(invariant.criteria, "\n- ")

          name <> "\n\n" <> description <> "\n\n**Criteria:**\n- " <> criteria
        })
        |> string.join("\n\n")

      "## Global Invariants\n\n" <> invariant_items <> "\n"
    }
  }
}

fn generate_anti_patterns(spec: Spec) -> String {
  case spec.anti_patterns {
    [] -> ""
    anti_patterns -> {
      let pattern_items =
        anti_patterns
        |> list.map(fn(pattern) {
          let name = "### " <> pattern.name
          let description = pattern.description

          let bad_example =
            "\n#### ❌ Bad Example\n\n```\n"
            <> json.to_string(pattern.bad_example)
            <> "\n```\n"

          let good_example =
            "\n#### ✅ Good Example\n\n```\n"
            <> json.to_string(pattern.good_example)
            <> "\n```\n"

          let why = case pattern.why {
            "" -> ""
            why -> "\n#### Why\n\n" <> why <> "\n"
          }

          name <> "\n\n" <> description <> bad_example <> good_example <> why
        })
        |> string.join("\n\n")

      "## Anti-Patterns\n\n" <> pattern_items <> "\n"
    }
  }
}

fn generate_technical_considerations(spec: Spec) -> String {
  let stack = case spec.ai_hints.implementation.suggested_stack {
    [] -> ""
    stack -> {
      let stack_list =
        stack
        |> list.map(fn(s) { "- " <> s })
        |> string.join("\n")

      "### Suggested Stack\n\n" <> stack_list <> "\n\n"
    }
  }

  let entities = case dict.keys(spec.ai_hints.entities) {
    [] -> ""
    keys -> {
      let entity_list =
        keys
        |> list.map(fn(key) {
          let entity = dict.get(spec.ai_hints.entities, key)
          case entity {
            Ok(e) -> {
              let fields =
                e.fields
                |> dict.to_list
                |> list.map(fn(pair) {
                  "- **" <> pair.0 <> "**: " <> json.to_string(pair.1)
                })
                |> string.join("\n")

              "#### " <> key <> "\n\n" <> fields
            }
            Error(_) -> ""
          }
        })
        |> string.join("\n\n")

      "### Data Entities\n\n" <> entity_list <> "\n\n"
    }
  }

  let security = {
    let security_items =
      [
        "Password Hashing: " <> spec.ai_hints.security.password_hashing,
        "JWT Algorithm: " <> spec.ai_hints.security.jwt_algorithm,
        "JWT Expiry: " <> spec.ai_hints.security.jwt_expiry,
        "Rate Limiting: " <> spec.ai_hints.security.rate_limiting,
      ]
      |> list.filter(fn(s) { !string.contains(s, ": ") })
      |> string.join("\n")

    case security_items {
      "" -> ""
      _ -> "### Security Considerations\n\n" <> security_items <> "\n\n"
    }
  }

  let pitfalls = case spec.ai_hints.pitfalls {
    [] -> ""
    pitfalls -> {
      let pitfall_list =
        pitfalls
        |> list.map(fn(p) { "- " <> p })
        |> string.join("\n")

      "### Common Pitfalls\n\n" <> pitfall_list <> "\n"
    }
  }

  "## Technical Considerations\n\n" <> stack <> entities <> security <> pitfalls
}
