/// Interactive specification refinement
/// Suggests improvements based on quality analysis and linting results
import gleam/int
import gleam/list
import gleam/string
import intent/quality_analyzer.{type QualityReport}
import intent/spec_linter.{type LintResult}
import intent/types.{type Behavior, type Spec}

/// Improvement suggestion with reasoning and impact
pub type ImprovementSuggestion {
  ImprovementSuggestion(
    title: String,
    description: String,
    reasoning: String,
    impact_score: Int,
    proposed_change: ProposedChange,
  )
}

/// Type of change being proposed
pub type ProposedChange {
  AddMissingTest(behavior_name: String, test_description: String)
  RefineVagueRule(behavior_name: String, field: String, better_rule: String)
  AddResponseExample(behavior_name: String)
  RenameForClarity(old_name: String, new_name: String)
  SimplifyRule(behavior_name: String, field: String, simpler_rule: String)
  AddExplanation(behavior_name: String, field: String, explanation: String)
}

/// Improvement context combining quality and lint results
pub type ImprovementContext {
  ImprovementContext(
    quality_report: QualityReport,
    lint_result: LintResult,
    spec: Spec,
  )
}

/// Generate improvement suggestions from analysis results
pub fn suggest_improvements(
  context: ImprovementContext,
) -> List(ImprovementSuggestion) {
  let mut_suggestions = []

  // Suggestions from quality issues
  let quality_suggestions =
    suggest_from_quality_issues(context.quality_report, context.spec)
  let mut_suggestions = list.append(mut_suggestions, quality_suggestions)

  // Suggestions from linting warnings
  let lint_suggestions = suggest_from_lint_warnings(context.lint_result)
  let mut_suggestions = list.append(mut_suggestions, lint_suggestions)

  // Sort by impact score (highest first)
  mut_suggestions
  |> list.sort(fn(a, b) { int.compare(b.impact_score, a.impact_score) })
}

/// Generate suggestions based on quality analysis
fn suggest_from_quality_issues(
  report: QualityReport,
  spec: Spec,
) -> List(ImprovementSuggestion) {
  let behaviors =
    spec.features
    |> list.flat_map(fn(f) { f.behaviors })

  []
  |> append_coverage_suggestions(report, behaviors)
  |> append_clarity_suggestions(report, behaviors)
  |> append_testability_suggestions(report, behaviors)
  |> append_ai_readiness_suggestions(report, behaviors)
}

/// Add suggestions for improving coverage
fn append_coverage_suggestions(
  suggestions: List(ImprovementSuggestion),
  _report: QualityReport,
  behaviors: List(Behavior),
) -> List(ImprovementSuggestion) {
  // In v3.0, error conditions are expressed through intent descriptions
  let has_error_tests =
    list.any(behaviors, fn(b) {
      let intent_lower = string.lowercase(b.intent)
      string.contains(intent_lower, "error")
      || string.contains(intent_lower, "fail")
      || string.contains(intent_lower, "invalid")
      || string.contains(intent_lower, "not found")
    })

  case has_error_tests {
    True -> suggestions
    False -> {
      list.append(suggestions, [
        ImprovementSuggestion(
          title: "Add error case tests",
          description: "No behaviors test error conditions",
          reasoning: "Testing error cases ensures the system handles failures gracefully and helps AI understand error conditions",
          impact_score: 25,
          proposed_change: AddMissingTest(
            "test-error-not-found",
            "Test system behavior when resource is not found",
          ),
        ),
      ])
    }
  }
}

/// Add suggestions for improving clarity
fn append_clarity_suggestions(
  suggestions: List(ImprovementSuggestion),
  _report: QualityReport,
  behaviors: List(Behavior),
) -> List(ImprovementSuggestion) {
  let missing_intent =
    behaviors
    |> list.filter(fn(b) { string.is_empty(b.intent) })
    |> list.length

  case missing_intent > 0 {
    False -> suggestions
    True -> {
      list.append(suggestions, [
        ImprovementSuggestion(
          title: "Add intent descriptions",
          description: int.to_string(missing_intent)
            <> " behavior(s) lack intent descriptions",
          reasoning: "Intent descriptions explain WHY a test exists, helping both humans and AI understand the business logic",
          impact_score: 15,
          proposed_change: AddExplanation(
            "test-success",
            "intent",
            "Verify successful operation with valid input",
          ),
        ),
      ])
    }
  }
}

/// Add suggestions for improving testability
fn append_testability_suggestions(
  suggestions: List(ImprovementSuggestion),
  _report: QualityReport,
  behaviors: List(Behavior),
) -> List(ImprovementSuggestion) {
  let missing_examples =
    behaviors
    |> list.filter(fn(b) {
      list.is_empty(b.verifications) || list.all(b.verifications, fn(v) {
        list.is_empty(v.examples)
      })
    })
    |> list.length

  case missing_examples > 0 {
    False -> suggestions
    True -> {
      list.append(suggestions, [
        ImprovementSuggestion(
          title: "Add verification examples",
          description: int.to_string(missing_examples)
            <> " behavior(s) lack verification examples",
          reasoning: "Examples make the spec executable and give AI concrete data structures to work with",
          impact_score: 20,
          proposed_change: AddResponseExample("test-success"),
        ),
      ])
    }
  }
}

/// Add suggestions for improving AI readiness
fn append_ai_readiness_suggestions(
  suggestions: List(ImprovementSuggestion),
  _report: QualityReport,
  behaviors: List(Behavior),
) -> List(ImprovementSuggestion) {
  let missing_criteria =
    behaviors
    |> list.filter(fn(b) {
      list.is_empty(b.verifications) || list.any(b.verifications, fn(v) {
        list.is_empty(v.criteria)
      })
    })
    |> list.length

  case missing_criteria > 0 {
    False -> suggestions
    True -> {
      list.append(suggestions, [
        ImprovementSuggestion(
          title: "Add verification criteria",
          description: int.to_string(missing_criteria)
            <> " behavior(s) lack verification criteria",
          reasoning: "Explanations help AI understand the business logic behind each verification",
          impact_score: 18,
          proposed_change: AddExplanation(
            "test-success",
            "criteria",
            "Field must not be empty and must match expected format",
          ),
        ),
      ])
    }
  }
}

/// Generate suggestions from lint warnings
fn suggest_from_lint_warnings(
  lint_result: LintResult,
) -> List(ImprovementSuggestion) {
  case lint_result {
    spec_linter.LintValid -> []
    spec_linter.LintWarnings(warnings) -> {
      warnings
      |> list.filter_map(fn(warning) {
        case warning {
          spec_linter.VagueVerification(behavior, verification, criteria) -> {
            Ok(ImprovementSuggestion(
              title: "Clarify verification criteria",
              description: "Behavior '"
                <> behavior
                <> "', verification '"
                <> verification
                <> "' uses vague language",
              reasoning: "Specific verification criteria are more testable and easier for AI to implement",
              impact_score: 22,
              proposed_change: RefineVagueRule(
                behavior,
                criteria,
                "Add specific, testable criteria with examples",
              ),
            ))
          }

          spec_linter.MissingExample(behavior) -> {
            Ok(ImprovementSuggestion(
              title: "Add verification example",
              description: "Behavior '"
                <> behavior
                <> "' has no verification examples",
              reasoning: "Examples make specifications executable and concrete",
              impact_score: 20,
              proposed_change: AddResponseExample(behavior),
            ))
          }

          spec_linter.NamingConvention(behavior, suggestion) -> {
            Ok(ImprovementSuggestion(
              title: "Improve naming",
              description: suggestion,
              reasoning: "Consistent naming conventions make specs easier to read and navigate",
              impact_score: 10,
              proposed_change: RenameForClarity(
                behavior,
                string.replace(behavior, "_", "-"),
              ),
            ))
          }

          spec_linter.UnusedAntiPattern(pattern) -> {
            Ok(ImprovementSuggestion(
              title: "Remove unused anti-pattern",
              description: "Anti-pattern '"
                <> pattern
                <> "' is not tested by any behavior",
              reasoning: "Unused anti-patterns add clutter without providing test coverage",
              impact_score: 5,
              proposed_change: RefineVagueRule(
                pattern,
                "pattern",
                "remove from spec",
              ),
            ))
          }

          spec_linter.AntiPatternDetected(behavior, pattern, _details) -> {
            Ok(ImprovementSuggestion(
              title: "Fix anti-pattern in response",
              description: "Behavior '"
                <> behavior
                <> "' contains anti-pattern '"
                <> pattern
                <> "'",
              reasoning: "Anti-patterns represent bad responses that should not be in examples",
              impact_score: 30,
              proposed_change: RefineVagueRule(
                behavior,
                "response.example",
                "Remove keys matching anti-pattern bad_example",
              ),
            ))
          }

          spec_linter.DuplicateBehavior(behavior1, behavior2, similarity) -> {
            Ok(ImprovementSuggestion(
              title: "Consolidate duplicate behaviors",
              description: "Behaviors '"
                <> behavior1
                <> "' and '"
                <> behavior2
                <> "' are similar ("
                <> similarity
                <> ")",
              reasoning: "Duplicate behaviors increase maintenance burden and reduce clarity",
              impact_score: 25,
              proposed_change: RefineVagueRule(
                behavior1,
                "consolidation",
                "Merge with " <> behavior2 <> " or clarify differences",
              ),
            ))
          }

          spec_linter.MissingPreconditions(behavior) -> {
            Ok(ImprovementSuggestion(
              title: "Add preconditions",
              description: "Behavior '"
                <> behavior
                <> "' lacks preconditions",
              reasoning: "Preconditions clarify what must be true before execution, improving testability",
              impact_score: 20,
              proposed_change: AddExplanation(
                behavior,
                "preconditions",
                "Define what must be true before this behavior executes",
              ),
            ))
          }

          spec_linter.MissingPostconditions(behavior) -> {
            Ok(ImprovementSuggestion(
              title: "Add postconditions",
              description: "Behavior '"
                <> behavior
                <> "' lacks postconditions",
              reasoning: "Postconditions clarify what must be true after execution, improving testability",
              impact_score: 20,
              proposed_change: AddExplanation(
                behavior,
                "postconditions",
                "Define what must be true after this behavior executes",
              ),
            ))
          }
        }
      })
    }
  }
}

/// Generate a refined spec based on accepted suggestions
pub fn apply_improvements(
  spec: Spec,
  _suggestions_to_apply: List(ImprovementSuggestion),
) -> Spec {
  // For now, return spec unchanged
  // Full implementation would apply changes to behaviors, features, etc.
  spec
}

/// Format improvements for interactive display
pub fn format_improvements(suggestions: List(ImprovementSuggestion)) -> String {
  let count = list.length(suggestions)

  case count {
    0 -> "No improvements suggested - spec is well-formed!"
    _ -> {
      let header =
        "Found " <> int.to_string(count) <> " improvement suggestion(s):\\n\\n"

      let suggestion_lines =
        suggestions
        |> list.index_map(fn(suggestion, i) {
          let idx = i + 1
          let impact_bar = string.repeat("█", suggestion.impact_score / 10)
          let impact_empty =
            string.repeat("░", 10 - suggestion.impact_score / 10)

          idx
          |> int.to_string
          <> ". "
          <> suggestion.title
          <> "\\n   Impact: ["
          <> impact_bar
          <> impact_empty
          <> "] "
          <> int.to_string(suggestion.impact_score)
          <> "/100\\n   "
          <> suggestion.description
          <> "\\n   Why: "
          <> suggestion.reasoning
          <> "\\n"
        })
        |> string.join("\\n")

      header <> suggestion_lines
    }
  }
}
