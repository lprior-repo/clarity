//// Acceptance Test Synthesizer
//// Generates intelligent acceptance tests from plan-work AI answers
////
//// This module analyzes AI planning content and automatically generates
//// contextual, testable acceptance criteria for generated beads.

import gleam/list
import gleam/string
import intent/case_insensitive.{contains_any_ignore_case}

/// Synthesis context for generating acceptance tests
pub type SynthesisContext {
  SynthesisContext(
    session_id: String,
    bead_id: String,
    bead_title: String,
    ai_answer: String,
    phase: Int,
    dependencies: List(String),
  )
}

/// Test strategy for generating different types of acceptance tests
pub type TestStrategy {
  BehaviorVerification
  // "Verify behavior X works when Y"
  OutputValidation
  // "Confirm output Z matches expected format"
  IntegrationCheck
  // "Ensure integration with component W"
  ErrorHandling
  // "Test error handling for edge case E"
  PerformanceMetric
  // "Validate performance meets threshold T"
  Auto
  // Automatically infer from context
}

/// Synthesize acceptance tests from AI planning content
pub fn synthesize_acceptance_tests(
  context: SynthesisContext,
  strategy: TestStrategy,
) -> List(String) {
  let effective_strategy = case strategy {
    Auto -> infer_strategy_from_context(context)
    _ -> strategy
  }

  let testable_elements = extract_testable_elements(context.ai_answer)

  let base_tests =
    testable_elements
    |> list.map(fn(element) {
      format_acceptance_test(element, effective_strategy, context)
    })

  let dependency_tests = generate_dependency_tests(context)

  let phase_tests = generate_phase_tests(context, effective_strategy)

  // Combine and deduplicate while maintaining order
  [base_tests, dependency_tests, phase_tests]
  |> list.flatten
  |> dedupe_tests
  |> ensure_minimum_tests(context)
}

/// Extract testable elements from AI answer text
pub fn extract_testable_elements(ai_answer: String) -> List(String) {
  let lines =
    ai_answer
    |> string.split("\n")
    |> list.filter(fn(line) {
      let trimmed = string.trim(line)
      let is_empty = trimmed == ""
      let first_char =
        trimmed
        |> string.to_graphemes
        |> list.first

      let is_comment = case first_char {
        Ok(char) -> char == "#"
        Error(_) -> False
      }

      case is_empty || is_comment {
        True -> False
        False -> True
      }
    })

  let keywords = [
    "implement", "create", "build", "add", "verify", "validate", "check",
    "ensure", "test", "handle", "support", "generate", "parse", "process",
  ]

  let testable_lines =
    lines
    |> list.filter(fn(line) { contains_any_ignore_case(line, keywords) })

  case testable_lines {
    [] -> ["Complete the implementation"]
    _ -> testable_lines
  }
}

/// Format a single acceptance test based on strategy
pub fn format_acceptance_test(
  element: String,
  strategy: TestStrategy,
  context: SynthesisContext,
) -> String {
  let cleaned_element =
    element
    |> string.trim()
    |> string.replace("IMPLEMENT", "")
    |> string.replace("Implement", "")
    |> string.replace("implement", "")
    |> string.replace(".", "")
    |> string.trim()

  let prefix = case strategy {
    BehaviorVerification -> "Verify"
    OutputValidation -> "Confirm"
    IntegrationCheck -> "Ensure"
    ErrorHandling -> "Test error handling for"
    PerformanceMetric -> "Validate performance of"
    Auto -> "Verify"
  }

  let test_body = format_test_body(cleaned_element, strategy, context)

  prefix <> " " <> test_body
}

/// Infer test strategy from bead context
fn infer_strategy_from_context(context: SynthesisContext) -> TestStrategy {
  let title_lower = string.lowercase(context.bead_title)
  let answer_lower = string.lowercase(context.ai_answer)

  // Check for error handling keywords
  let error_keywords = ["error", "fail", "exception", "timeout", "edge case"]
  let has_error_keywords =
    contains_any_ignore_case(title_lower <> " " <> answer_lower, error_keywords)

  let check_performance = [
    "performance", "latency", "response time", "throughput",
  ]
  let has_performance_keywords =
    contains_any_ignore_case(
      title_lower <> " " <> answer_lower,
      check_performance,
    )

  let integration_keywords = [
    "integration", "connect", "api", "endpoint", "service",
  ]
  let has_integration_keywords =
    contains_any_ignore_case(
      title_lower <> " " <> answer_lower,
      integration_keywords,
    )

  case
    has_error_keywords,
    has_performance_keywords,
    has_integration_keywords,
    context.phase > 2
  {
    True, _, _, _ -> ErrorHandling
    _, True, _, _ -> PerformanceMetric
    _, _, True, _ -> IntegrationCheck
    _, _, _, True -> BehaviorVerification
    _, _, _, _ -> OutputValidation
  }
}

/// Generate dependency-aware tests
fn generate_dependency_tests(context: SynthesisContext) -> List(String) {
  case context.dependencies {
    [] -> []
    deps -> {
      deps
      |> list.map(fn(dep) {
        "Verify integration with " <> dep <> " is working correctly"
      })
      |> list.take(3)
      // Limit to prevent too many tests
    }
  }
}

/// Generate phase-specific tests
fn generate_phase_tests(
  context: SynthesisContext,
  _strategy: TestStrategy,
) -> List(String) {
  case context.phase {
    1 -> [
      "Verify module compiles without errors",
      "Confirm basic functionality works with minimal input",
    ]
    2 -> [
      "Verify integration with earlier phase components",
      "Test with realistic data scenarios",
    ]
    _ -> [
      "Verify end-to-end workflow completes successfully",
      "Test with production-like scenarios",
    ]
  }
}

/// Format test body based on strategy
fn format_test_body(
  element: String,
  strategy: TestStrategy,
  _context: SynthesisContext,
) -> String {
  let element_lower = string.lowercase(element)

  case strategy, element_lower {
    BehaviorVerification, _ -> {
      case string.length(element) > 50 {
        True ->
          element
          |> string.slice(0, 50)
          <> "... works as expected"
        False -> element <> " works as expected"
      }
    }
    OutputValidation, _ -> {
      element <> " produces expected output format"
    }
    IntegrationCheck, _ -> {
      "integration with " <> element <> " is properly established"
    }
    ErrorHandling, _ -> {
      case
        contains_any_ignore_case(element, ["timeout", "network", "external"])
      {
        True -> element <> " is handled gracefully"
        False -> "proper error handling for " <> element
      }
    }
    PerformanceMetric, _ -> {
      element <> " meets performance requirements"
    }
    Auto, _ -> {
      element <> " behaves correctly"
    }
  }
}

/// Deduplicate tests while preserving order
fn dedupe_tests(tests: List(String)) -> List(String) {
  dedupe_loop(tests, [])
}

fn dedupe_loop(tests: List(String), seen: List(String)) -> List(String) {
  case tests {
    [] -> []
    [current_test, ..rest] -> {
      let key = string.lowercase(string.trim(current_test))

      case list.contains(seen, key) {
        True -> dedupe_loop(rest, seen)
        False -> {
          [current_test, ..dedupe_loop(rest, [key, ..seen])]
        }
      }
    }
  }
}

/// Ensure minimum number of tests per bead
fn ensure_minimum_tests(
  tests: List(String),
  context: SynthesisContext,
) -> List(String) {
  let min_tests = 3
  let current_count = list.length(tests)

  case current_count >= min_tests {
    True -> tests
    False -> {
      let fallback_tests =
        generate_fallback_tests(context, min_tests - current_count)
      list.append(tests, fallback_tests)
    }
  }
}

/// Generate fallback tests to meet minimum requirements
fn generate_fallback_tests(
  context: SynthesisContext,
  count: Int,
) -> List(String) {
  let fallback_templates = [
    "Verify " <> context.bead_title <> " meets specification requirements",
    "Test " <> context.bead_title <> " with valid inputs",
    "Verify " <> context.bead_title <> " handles edge cases correctly",
    "Confirm " <> context.bead_title <> " produces expected outputs",
    "Test " <> context.bead_title <> " integration with dependent components",
  ]

  fallback_templates
  |> list.take(count)
}
