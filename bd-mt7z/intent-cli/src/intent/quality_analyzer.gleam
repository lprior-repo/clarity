/// Spec quality analysis and scoring
/// Analyzes completeness, clarity, testability, and AI readiness
import gleam/int
import gleam/list
import gleam/string
import intent/case_insensitive.{contains_any_ignore_case}
import intent/types.{type Behavior, type Invariant, type Spec}

/// Quality metrics for a spec
pub type QualityReport {
  QualityReport(
    coverage_score: Int,
    clarity_score: Int,
    testability_score: Int,
    ai_readiness_score: Int,
    overall_score: Int,
    issues: List(QualityIssue),
    suggestions: List(String),
  )
}

/// Quality issues found
pub type QualityIssue {
  MissingErrorTests
  MissingAuthenticationTest
  MissingEdgeCases
  VagueRules
  NoExamples
  MissingExplanations
  UntestedInvariants
  MissingAIHints
  MissingPreconditions
  MissingPostconditions
}

/// Analyze spec quality
pub fn analyze_spec(spec: Spec) -> QualityReport {
  let behaviors =
    spec.features
    |> list.flat_map(fn(f) { f.behaviors })

  let coverage_score = calculate_coverage_score(behaviors, spec.invariants)
  let clarity_score = calculate_clarity_score(behaviors)
  let testability_score = calculate_testability_score(behaviors)
  let ai_readiness_score = calculate_ai_readiness_score(spec, behaviors)

  let overall_score = {
    let sum =
      coverage_score + clarity_score + testability_score + ai_readiness_score
    sum / 4
  }

  let issues = find_quality_issues(behaviors, spec.invariants)
  let suggestions = generate_suggestions(issues, behaviors, spec.invariants)

  QualityReport(
    coverage_score: coverage_score,
    clarity_score: clarity_score,
    testability_score: testability_score,
    ai_readiness_score: ai_readiness_score,
    overall_score: overall_score,
    issues: issues,
    suggestions: suggestions,
  )
}

/// Calculate coverage score (0-100)
/// Measures how many error cases and edge cases are tested
fn calculate_coverage_score(behaviors: List(Behavior), invariants: List(Invariant)) -> Int {
  let base = 50

  // Check for error behaviors in postconditions
  let error_behaviors =
    behaviors
    |> list.filter(fn(b) {
      list.any(b.postconditions, fn(pc) {
        string.contains(pc, "error")
        || string.contains(pc, "fail")
        || string.contains(pc, "invalid")
      })
    })
    |> list.length

  let error_bonus = int.min(50, error_behaviors * 10)

  // Check if authentication tested
  let has_auth_test =
    behaviors
    |> list.any(fn(b) {
      contains_any_ignore_case(b.name, ["auth"])
      || contains_any_ignore_case(b.intent, ["auth"])
    })

  let auth_bonus = case has_auth_test {
    True -> 10
    False -> 0
  }

  // Check for edge cases (empty, max length, invalid)
  let has_edge_cases =
    behaviors
    |> list.any(fn(b) {
      contains_any_ignore_case(b.name, ["empty", "invalid", "max"])
      || contains_any_ignore_case(b.intent, ["edge"])
    })

  let edge_bonus = case has_edge_cases {
    True -> 10
    False -> 0
  }

  // Check for invariant coverage
  let antipattern_bonus = int.min(5, list.length(invariants) * 2)

  let coverage_total =
    base + error_bonus + auth_bonus + edge_bonus + antipattern_bonus
  int.min(100, coverage_total)
}

/// Calculate clarity score (0-100)
/// Measures how well documented the spec is
fn calculate_clarity_score(behaviors: List(Behavior)) -> Int {
  let base = 60

  // Count behaviors with intent descriptions
  let with_intent =
    behaviors
    |> list.filter(fn(b) { !string.is_empty(b.intent) })
    |> list.length

  let intent_ratio = case list.length(behaviors) {
    0 -> 0
    n -> {
      let ratio = with_intent * 100
      ratio / n
    }
  }

  let intent_bonus = int.min(10, intent_ratio / 10)

  // Count behaviors with notes
  let with_notes =
    behaviors
    |> list.filter(fn(b) { !string.is_empty(b.notes) })
    |> list.length

  let notes_bonus = case list.length(behaviors) {
    0 -> 0
    n -> {
      let bonus_calc = with_notes * 10
      bonus_calc / n
    }
  }

  // Check for vague language in verifications
  let has_vague_verifications =
    behaviors
    |> list.any(fn(b) {
      list.any(b.verifications, fn(v) {
        list.any(v.criteria, fn(criterion) {
          let criterion_lower = string.lowercase(criterion)
          string.contains(criterion_lower, "valid")
          && !string.contains(criterion_lower, "email")
          && !string.contains(criterion_lower, "uuid")
          && !string.contains(criterion_lower, "iso")
        })
      })
    })

  let vague_penalty = case has_vague_verifications {
    True -> -10
    False -> 0
  }

  let clarity_total = base + intent_bonus + notes_bonus + vague_penalty
  int.max(0, int.min(100, clarity_total))
}

/// Calculate testability score (0-100)
/// Measures how well structured for execution
fn calculate_testability_score(behaviors: List(Behavior)) -> Int {
  let base = 70

  // Check for well-defined dependencies
  let with_dependencies =
    behaviors
    |> list.filter(fn(b) { !list.is_empty(b.requires) })
    |> list.length

  let deps_bonus = int.min(10, with_dependencies * 5)

  // Check for preconditions
  let with_preconditions =
    behaviors
    |> list.filter(fn(b) { !list.is_empty(b.preconditions) })
    |> list.length

  let preconditions_bonus = int.min(10, with_preconditions * 5)

  // Check for postconditions
  let with_postconditions =
    behaviors
    |> list.filter(fn(b) { !list.is_empty(b.postconditions) })
    |> list.length

  let postconditions_bonus = int.min(10, with_postconditions * 5)

  // Check for verifications with examples
  let with_examples =
    behaviors
    |> list.filter(fn(b) {
      list.any(b.verifications, fn(v) { !list.is_empty(v.examples) })
    })
    |> list.length

  let example_bonus =
    int.min(5, with_examples / int.max(1, list.length(behaviors) / 2))

  let testability_total =
    base + deps_bonus + preconditions_bonus + postconditions_bonus + example_bonus
  int.min(100, testability_total)
}

/// Calculate AI readiness score (0-100)
/// Measures how much guidance is available for AI
fn calculate_ai_readiness_score(spec: Spec, behaviors: List(Behavior)) -> Int {
  let base = 50

  // Check for AI hints (has any non-empty implementation suggestions or pitfalls)
  let has_ai_hints =
    !list.is_empty(spec.ai_hints.implementation.suggested_stack)
    || !list.is_empty(spec.ai_hints.pitfalls)

  let hints_bonus = case has_ai_hints {
    True -> 20
    False -> -10
  }

  // Count behaviors with verifications (these provide guidance)
  let with_verifications =
    behaviors
    |> list.filter(fn(b) { !list.is_empty(b.verifications) })
    |> list.length

  let verification_ratio = case list.length(behaviors) {
    0 -> 0
    n -> {
      let ratio = with_verifications * 100
      ratio / n
    }
  }

  let verification_bonus = verification_ratio / 5

  // Count examples in verifications
  let with_examples =
    behaviors
    |> list.filter(fn(b) {
      list.any(b.verifications, fn(v) { !list.is_empty(v.examples) })
    })
    |> list.length

  let example_bonus = int.min(10, with_examples * 5)

  let ai_readiness_total = base + hints_bonus + verification_bonus + example_bonus
  int.max(0, int.min(100, ai_readiness_total))
}

/// Find quality issues in spec
fn find_quality_issues(
  behaviors: List(Behavior),
  invariants: List(Invariant),
) -> List(QualityIssue) {
  let mut_issues = []

  // Check for error behaviors
  let has_error_behaviors = list.any(behaviors, fn(b) {
    list.any(b.postconditions, fn(pc) {
      string.contains(pc, "error")
      || string.contains(pc, "fail")
    })
  })

  let mut_issues = case has_error_behaviors {
    True -> mut_issues
    False -> [MissingErrorTests, ..mut_issues]
  }

  // Check for auth tests
  let has_auth_test =
    list.any(behaviors, fn(b) { contains_any_ignore_case(b.name, ["auth"]) })

  let mut_issues = case has_auth_test {
    True -> mut_issues
    False -> [MissingAuthenticationTest, ..mut_issues]
  }

  // Check for edge cases
  let has_edge_cases =
    list.any(behaviors, fn(b) {
      contains_any_ignore_case(b.name, ["empty", "invalid"])
    })

  let mut_issues = case has_edge_cases {
    True -> mut_issues
    False -> [MissingEdgeCases, ..mut_issues]
  }

  // Check for vague verifications
  let has_vague =
    list.any(behaviors, fn(b) {
      list.any(b.verifications, fn(v) {
        list.any(v.criteria, fn(criterion) {
          let criterion_lower = string.lowercase(criterion)
          string.contains(criterion_lower, "valid data")
          || string.contains(criterion_lower, "correct format")
        })
      })
    })

  let mut_issues = case has_vague {
    False -> mut_issues
    True -> [VagueRules, ..mut_issues]
  }

  // Check for examples in verifications
  let has_examples =
    list.any(behaviors, fn(b) {
      list.any(b.verifications, fn(v) { !list.is_empty(v.examples) })
    })

  let mut_issues = case has_examples {
    True -> mut_issues
    False -> [NoExamples, ..mut_issues]
  }

  // Check for preconditions
  let has_preconditions =
    list.any(behaviors, fn(b) { !list.is_empty(b.preconditions) })

  let mut_issues = case has_preconditions {
    True -> mut_issues
    False -> [MissingPreconditions, ..mut_issues]
  }

  // Check for postconditions
  let has_postconditions =
    list.any(behaviors, fn(b) { !list.is_empty(b.postconditions) })

  let mut_issues = case has_postconditions {
    True -> mut_issues
    False -> [MissingPostconditions, ..mut_issues]
  }

  // Check for untested invariants
  let has_untested_invariants = !list.is_empty(invariants)

  let mut_issues = case has_untested_invariants {
    False -> mut_issues
    True -> [UntestedInvariants, ..mut_issues]
  }

  mut_issues
}

/// Generate suggestions for improvement
fn generate_suggestions(
  issues: List(QualityIssue),
  _behaviors: List(Behavior),
  _invariants: List(Invariant),
) -> List(String) {
  []
  |> add_suggestion_if(
    list.contains(issues, MissingErrorTests),
    "Add behaviors that test error cases (failures, invalid input)",
  )
  |> add_suggestion_if(
    list.contains(issues, MissingAuthenticationTest),
    "Add test cases for authentication (missing auth, invalid token)",
  )
  |> add_suggestion_if(
    list.contains(issues, MissingEdgeCases),
    "Add edge case tests (empty values, max length, invalid input)",
  )
  |> add_suggestion_if(
    list.contains(issues, VagueRules),
    "Replace vague verifications like 'valid data' with specific validation criteria",
  )
  |> add_suggestion_if(
    list.contains(issues, NoExamples),
    "Add examples to verifications for documentation",
  )
  |> add_suggestion_if(
    list.contains(issues, MissingPreconditions),
    "Add preconditions to behaviors to specify what must be true before execution",
  )
  |> add_suggestion_if(
    list.contains(issues, MissingPostconditions),
    "Add postconditions to behaviors to specify what must be true after execution",
  )
}

/// Helper to add suggestion conditionally
fn add_suggestion_if(
  suggestions: List(String),
  condition: Bool,
  suggestion: String,
) -> List(String) {
  case condition {
    True -> [suggestion, ..suggestions]
    False -> suggestions
  }
}

/// Format quality report for display
pub fn format_report(report: QualityReport) -> String {
  let score_section =
    "Quality Score: "
    <> int.to_string(report.overall_score)
    <> "/100\n"
    <> "  Coverage: "
    <> int.to_string(report.coverage_score)
    <> "/100\n"
    <> "  Clarity: "
    <> int.to_string(report.clarity_score)
    <> "/100\n"
    <> "  Testability: "
    <> int.to_string(report.testability_score)
    <> "/100\n"
    <> "  AI Readiness: "
    <> int.to_string(report.ai_readiness_score)
    <> "/100"

  let issues_section = case list.is_empty(report.issues) {
    True -> "No quality issues found!"
    False ->
      "Quality Issues:\n"
      <> string.join(report.issues |> list.map(format_issue), "\n")
  }

  let suggestions_section = case list.is_empty(report.suggestions) {
    True -> ""
    False ->
      "\n\nSuggestions for Improvement:\n"
      <> string.join(
        report.suggestions
          |> list.index_map(fn(s, i) { int.to_string(i + 1) <> ". " <> s }),
        "\n",
      )
  }

  score_section <> "\n\n" <> issues_section <> suggestions_section
}

/// Format a quality issue
fn format_issue(issue: QualityIssue) -> String {
  case issue {
    MissingErrorTests -> "  • Missing error test cases (failures, invalid input)"
    MissingAuthenticationTest -> "  • Missing authentication tests"
    MissingEdgeCases -> "  • Missing edge case tests (empty, invalid, etc)"
    VagueRules ->
      "  • Vague verification criteria ('valid data', 'correct format')"
    NoExamples -> "  • No verification examples provided"
    MissingExplanations -> "  • Missing detailed verification descriptions"
    UntestedInvariants -> "  • Global invariants not verified in behaviors"
    MissingAIHints -> "  • No AI implementation hints provided"
    MissingPreconditions -> "  • Behaviors missing preconditions"
    MissingPostconditions -> "  • Behaviors missing postconditions"
  }
}
