/// Ready document generator
/// Generates implementation-ready documentation from specs
import gleam/dict
import gleam/json.{type Json}
import gleam/list
import gleam/string
import intent/types.{type Behavior, type Spec}

/// Generate a ready document from a spec
pub fn generate_ready_document(spec: Spec) -> String {
  let sections = [
    generate_header(spec),
    generate_overview(spec),
    generate_features(spec),
    generate_behaviors(spec),
    generate_invariants(spec),
    generate_verification_criteria(spec),
    generate_anti_patterns(spec),
    generate_implementation_hints(spec),
    generate_security_guidelines(spec),
  ]

  string.join(sections, "\n\n")
}

fn generate_header(spec: Spec) -> String {
  "# Ready Document: "
  <> spec.name
  <> "\n"
  <> "Version: "
  <> spec.version
  <> "\n"
  <> "Generated: "
  <> get_current_timestamp()
}

fn generate_overview(spec: Spec) -> String {
  "## Overview\n\n"
  <> "**Description:** "
  <> spec.description
  <> "\n\n"
  <> "**Audience:** "
  <> spec.audience
  <> "\n\n"
  <> "### Success Criteria\n\n"
  <> list.map(spec.success_criteria, fn(criteria) { "- " <> criteria })
  |> string.join("\n")
}

fn generate_features(spec: Spec) -> String {
  let features_list =
    list.map(spec.features, fn(feature) {
      "### " <> feature.name <> "\n\n" <> feature.description
    })
    |> string.join("\n\n")

  "## Features\n\n" <> features_list
}

fn generate_behaviors(spec: Spec) -> String {
  let behaviors_by_feature =
    list.map(spec.features, fn(feature) {
      let behaviors =
        list.map(feature.behaviors, fn(behavior) {
          generate_behavior_details(behavior)
        })
        |> string.join("\n\n")

      "### " <> feature.name <> "\n\n" <> behaviors
    })
    |> string.join("\n\n")

  "## Behaviors\n\n" <> behaviors_by_feature
}

fn generate_behavior_details(behavior: Behavior) -> String {
  let preconditions = case behavior.preconditions {
    [] -> ""
    preconditions -> {
      let pre_list =
        list.map(preconditions, fn(p) { "- " <> p })
        |> string.join("\n")
      "\n**Preconditions:**\n" <> pre_list
    }
  }

  let postconditions = case behavior.postconditions {
    [] -> ""
    postconditions -> {
      let post_list =
        list.map(postconditions, fn(p) { "- " <> p })
        |> string.join("\n")
      "\n**Postconditions:**\n" <> post_list
    }
  }

  let verifications = case behavior.verifications {
    [] -> ""
    verifications -> {
      let ver_list =
        list.map(verifications, fn(v) {
          "#### " <> v.description <> "\n\n"
          <> list.map(v.criteria, fn(c) { "- " <> c })
          |> string.join("\n")
          <> case list.is_empty(v.examples) {
            True -> ""
            False -> "\n\n**Examples:**\n" <> generate_verification_examples(v.examples)
          }
        })
        |> string.join("\n\n")
      "\n**Verifications:**\n" <> ver_list
    }
  }

  let requires = case behavior.requires {
    [] -> ""
    deps -> "\n**Dependencies:** " <> string.join(deps, ", ")
  }

  let tags = case behavior.tags {
    [] -> ""
    tags_list -> "\n**Tags:** " <> string.join(tags_list, ", ")
  }

  let notes = case string.is_empty(behavior.notes) {
    True -> ""
    False -> "\n**Notes:** " <> behavior.notes
  }

  "#### "
  <> behavior.name
  <> "\n\n"
  <> "**Intent:** "
  <> behavior.intent
  <> "\n"
  <> notes
  <> requires
  <> tags
  <> preconditions
  <> postconditions
  <> verifications
}

fn generate_verification_examples(examples: List(Json)) -> String {
  examples
  |> list.map(fn(example) {
    "```json\n" <> json.to_string(example) <> "\n```"
  })
  |> string.join("\n\n")
}

fn generate_invariants(spec: Spec) -> String {
  case list.is_empty(spec.invariants) {
    True -> ""
    False -> {
      "## Invariants\n\n"
      <> "These global rules apply to all behaviors:\n\n"
      <> list.map(spec.invariants, fn(invariant) {
        "### "
        <> invariant.name
        <> "\n\n"
        <> invariant.description
        <> "\n\n"
        <> "**Criteria:**\n"
        <> list.map(invariant.criteria, fn(c) { "- " <> c })
        |> string.join("\n")
      })
      |> string.join("\n\n")
    }
  }
}

fn generate_verification_criteria(_spec: Spec) -> String {
  "## Verification Criteria\n\n"
  <> "Each behavior should be verified against:\n\n"
  <> "1. **Preconditions**: Verify all preconditions are met before execution\n"
  <> "2. **Postconditions**: Verify all postconditions are true after execution\n"
  <> "3. **Verifications**: Execute all verification criteria specified\n"
  <> "4. **Error handling**: Verify error cases are handled correctly\n"
  <> "5. **Edge cases**: Test boundary conditions and unusual inputs\n\n"
  <> "### Automated Testing\n\n"
  <> "All behaviors should have automated tests that:\n"
  <> "- Verify preconditions are satisfied\n"
  <> "- Execute the behavior\n"
  <> "- Validate postconditions\n"
  <> "- Check all verification criteria\n"
  <> "- Test error conditions explicitly"
}

fn generate_anti_patterns(spec: Spec) -> String {
  "## Anti-Patterns to Avoid\n\n"
  <> list.map(spec.anti_patterns, fn(anti_pattern) {
    "### "
    <> anti_pattern.name
    <> "\n\n"
    <> anti_pattern.description
    <> "\n\n"
    <> "**Bad Example:**\n```json\n"
    <> json.to_string(anti_pattern.bad_example)
    <> "\n```\n\n"
    <> "**Good Example:**\n```json\n"
    <> json.to_string(anti_pattern.good_example)
    <> "\n```\n\n"
    <> "**Why:** "
    <> anti_pattern.why
  })
  |> string.join("\n\n")
}

fn generate_implementation_hints(spec: Spec) -> String {
  let hints = spec.ai_hints

  "## Implementation Hints\n\n"
  <> "### Suggested Stack\n\n"
  <> list.map(hints.implementation.suggested_stack, fn(item) { "- " <> item })
  |> string.join("\n")
  <> "\n\n"
  <> generate_entity_hints(hints.entities)
  <> "\n\n"
  <> generate_common_pitfalls(hints.pitfalls)
}

fn generate_entity_hints(
  entities: dict.Dict(String, types.EntityHint),
) -> String {
  case dict.size(entities) {
    0 -> ""
    _ ->
      "### Entity Models\n\n"
      <> dict.to_list(entities)
      |> list.map(fn(pair) { generate_entity_hint(pair.0, pair.1) })
      |> string.join("\n\n")
  }
}

fn generate_entity_hint(name: String, hint: types.EntityHint) -> String {
  "**"
  <> name
  <> "**\n\n"
  <> case dict.size(hint.fields) {
    0 -> "No field definitions"
    _ ->
      "Fields:\n"
      <> dict.to_list(hint.fields)
      |> list.map(fn(pair) {
        "  - **" <> pair.0 <> "**: " <> json.to_string(pair.1)
      })
      |> string.join("\n")
  }
}

fn generate_common_pitfalls(pitfalls: List(String)) -> String {
  case pitfalls {
    [] -> ""
    _ ->
      "### Common Pitfalls\n\n"
      <> list.map(pitfalls, fn(pitfall) { "- " <> pitfall })
      |> string.join("\n")
  }
}

fn generate_security_guidelines(spec: Spec) -> String {
  let security = spec.ai_hints.security

  "## Security Guidelines\n\n"
  <> "### Password Hashing\n\n"
  <> security.password_hashing
  <> "\n\n"
  <> "### JWT Configuration\n\n"
  <> "- **Algorithm:** "
  <> security.jwt_algorithm
  <> "\n"
  <> "- **Expiry:** "
  <> security.jwt_expiry
  <> "\n\n"
  <> "### Rate Limiting\n\n"
  <> security.rate_limiting
}

fn get_current_timestamp() -> String {
  // Get current timestamp in ISO 8601 format
  // This is a simplified version - in production you'd use a proper datetime library
  "2024-01-15T10:30:00Z"
}
