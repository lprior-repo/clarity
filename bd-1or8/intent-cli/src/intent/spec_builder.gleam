/// Spec Builder
/// Converts interview session answers into valid CUE specifications
import gleam/dict
import gleam/int
import gleam/list
import gleam/string
import intent/case_insensitive.{contains_any_ignore_case}
import intent/interview.{type Answer, type InterviewSession, type Profile}
import intent/types.{type Spec, Spec, type Behavior, Behavior, Feature, ImplementationHints, SecurityHints, AIHints}

/// Generated CUE code
pub type GeneratedCUE {
  GeneratedCUE(package: String, imports: List(String), body: String)
}

/// Build a CUE spec from a completed interview session
pub fn build_spec_from_session(session: InterviewSession) -> String {
  let features = extract_features_from_answers(session.answers)
  let behaviors =
    extract_behaviors_from_answers(session.answers, session.profile)
  let constraints = extract_constraints_from_answers(session.answers)
  let security = extract_security_requirements(session.answers)
  let non_functional = extract_non_functional_requirements(session.answers)

  // Build a minimal Spec type for CUE generation
  let spec =
    GeneratedCUE(
      package: "package api",
      imports: [],
      body: build_spec_body(
        features,
        behaviors,
        constraints,
        security,
        non_functional,
      ),
    )

  spec.package <> "\n\n" <> spec.body
}

/// Extract feature names/titles from answers
pub fn extract_features_from_answers(answers: List(Answer)) -> List(String) {
  answers
  |> list.filter(fn(answer) {
    contains_any_ignore_case(answer.question_text, ["feature", "capability"])
  })
  |> list.map(fn(answer) {
    let trimmed = string.trim(answer.response)
    case string.length(trimmed) > 0 {
      True -> trimmed
      False -> ""
    }
  })
  |> list.filter(fn(s) { s != "" })
}

/// Extract API behaviors (methods, paths, status codes)
pub fn extract_behaviors_from_answers(
  answers: List(Answer),
  _profile: Profile,
) -> String {
  let api_answers =
    list.filter(answers, fn(answer) {
      contains_any_ignore_case(answer.question_text, [
        "endpoint", "path", "method",
      ])
    })

  case api_answers {
    [] ->
      "// Define API behaviors here
behaviors: {
  // Add endpoint definitions
}"
    answers -> "// API behaviors from interview
behaviors: {
" <> {
        list.map(answers, fn(answer) { "  // " <> answer.question_text <> "
  // " <> string.trim(answer.response) })
        |> string.join("\n")
      } <> "\n}"
  }
}

/// Extract constraints from answers
pub fn extract_constraints_from_answers(answers: List(Answer)) -> List(String) {
  answers
  |> list.filter(fn(answer) {
    contains_any_ignore_case(answer.question_text, [
      "constraint", "limit", "requirement",
    ])
  })
  |> list.map(fn(answer) { string.trim(answer.response) })
  |> list.filter(fn(s) { s != "" })
}

/// Extract security requirements from answers
pub fn extract_security_requirements(answers: List(Answer)) -> String {
  let security_answers =
    list.filter(answers, fn(answer) {
      contains_any_ignore_case(answer.question_text, [
        "auth", "security", "permission",
      ])
    })

  case security_answers {
    [] ->
      "security: {
  authentication: \"todo\"
  authorization: \"todo\"
}"
    answers -> "security: {
" <> {
        list.map(answers, fn(answer) { "  // " <> answer.question_text <> "
  requirement: \"" <> string.trim(answer.response) <> "\"" })
        |> string.join("\n")
      } <> "\n}"
  }
}

/// Extract non-functional requirements (SLA, scale, monitoring)
pub fn extract_non_functional_requirements(
  answers: List(Answer),
) -> List(String) {
  answers
  |> list.filter(fn(answer) {
    contains_any_ignore_case(answer.question_text, [
      "sla", "scale", "performance", "monitoring", "latency",
    ])
  })
  |> list.map(fn(answer) { string.trim(answer.response) })
  |> list.filter(fn(s) { s != "" })
}

/// Build the main body of the spec
fn build_spec_body(
  features: List(String),
  behaviors: String,
  constraints: List(String),
  security: String,
  non_functional: List(String),
) -> String {
  let features_section = case features {
    [] ->
      "// Features
features: {
  // Add feature definitions
}"
    features -> "// Features extracted from interview
features: {
" <> {
        list.map(features, fn(feature) { "  \"" <> feature <> "\": true" })
        |> string.join("\n")
      } <> "\n}"
  }

  let constraints_section = case constraints {
    [] -> ""
    constraints -> "\n\n// Constraints and requirements
constraints: {
" <> {
        list.map(constraints, fn(constraint) { "  // " <> constraint })
        |> string.join("\n")
      } <> "\n}"
  }

  let non_functional_section = case non_functional {
    [] -> ""
    nf -> "\n\n// Non-functional requirements
nonFunctional: {
" <> {
        list.map(nf, fn(requirement) { "  // " <> requirement })
        |> string.join("\n")
      } <> "\n}"
  }

  features_section
  <> "\n\n"
  <> behaviors
  <> "\n\n"
  <> security
  <> constraints_section
  <> non_functional_section
}

/// Create a test spec with N behaviors - pure functional composition
pub fn create_test_spec(behavior_count: Int) -> Spec {
  let behaviors =
    list.range(1, behavior_count)
    |> list.map(fn(i) { make_behavior("b" <> int.to_string(i)) })
  Spec(
    name: "test",
    description: "test",
    audience: "test",
    version: "1.0.0",
    success_criteria: [],
    features: [Feature("test-feature", "test", behaviors)],
    invariants: [],
    anti_patterns: [],
    ai_hints: AIHints(
      ImplementationHints([]),
      dict.new(),
      SecurityHints("", "", "", ""),
      [],
    ),
  )
}

fn make_behavior(name: String) -> Behavior {
  Behavior(
    name:,
    intent: "test",
    notes: "",
    requires: [],
    tags: [],
    preconditions: [],
    postconditions: [],
    verifications: [],
  )
}
