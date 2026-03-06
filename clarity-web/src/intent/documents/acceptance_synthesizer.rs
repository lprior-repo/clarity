//! Acceptance Test Synthesizer
//!
//! Generates intelligent acceptance tests from plan-work AI answers.
//! This module analyzes AI planning content and automatically generates
//! contextual, testable acceptance criteria for generated beads.
//!
//! Ported from intent-cli/src/intent/acceptance_synthesizer.gleam

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use std::collections::HashSet;

use crate::intent::util::contains_any_ignore_case;

/// Synthesis context for generating acceptance tests
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynthesisContext {
  pub session_id: String,
  pub bead_id: String,
  pub bead_title: String,
  pub ai_answer: String,
  pub phase: u32,
  pub dependencies: Vec<String>,
}

impl Default for SynthesisContext {
  fn default() -> Self {
    Self {
      session_id: String::new(),
      bead_id: String::new(),
      bead_title: String::new(),
      ai_answer: String::new(),
      phase: 1,
      dependencies: Vec::new(),
    }
  }
}

/// Test strategy for generating different types of acceptance tests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TestStrategy {
  /// Verify behavior X works when Y
  BehaviorVerification,
  /// Confirm output Z matches expected format
  OutputValidation,
  /// Ensure integration with component W
  IntegrationCheck,
  /// Test error handling for edge case E
  ErrorHandling,
  /// Validate performance meets threshold T
  PerformanceMetric,
  /// Automatically infer from context
  #[default]
  Auto,
}

/// Synthesize acceptance tests from AI planning content
#[must_use]
pub fn synthesize_acceptance_tests(
  context: &SynthesisContext,
  strategy: TestStrategy,
) -> Vec<String> {
  let effective_strategy = match strategy {
    TestStrategy::Auto => infer_strategy_from_context(context),
    s => s,
  };

  let testable_elements = extract_testable_elements(&context.ai_answer);

  let dependency_tests = generate_dependency_tests(context);
  let phase_tests = generate_phase_tests(context, effective_strategy);

  // Combine and deduplicate while maintaining order
  let combined: Vec<String> = testable_elements
    .iter()
    .map(|element| format_acceptance_test(element, effective_strategy, context))
    .chain(dependency_tests)
    .chain(phase_tests)
    .collect();

  let deduped = dedupe_tests(combined);
  ensure_minimum_tests(deduped, context)
}

/// Extract testable elements from AI answer text
#[must_use]
pub fn extract_testable_elements(ai_answer: &str) -> Vec<String> {
  let keywords = [
    "implement",
    "create",
    "build",
    "add",
    "verify",
    "validate",
    "check",
    "ensure",
    "test",
    "handle",
    "support",
    "generate",
    "parse",
    "process",
  ];

  let testable_lines: Vec<String> = ai_answer
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .filter(|line| !line.starts_with('#'))
    .filter(|line| contains_any_ignore_case(line, &keywords))
    .map(String::from)
    .collect();

  if testable_lines.is_empty() {
    vec!["Complete the implementation".to_string()]
  } else {
    testable_lines
  }
}

/// Format a single acceptance test based on strategy
#[must_use]
pub fn format_acceptance_test(
  element: &str,
  strategy: TestStrategy,
  context: &SynthesisContext,
) -> String {
  let cleaned_element = element
    .trim()
    .replace("IMPLEMENT", "")
    .replace("Implement", "")
    .replace("implement", "")
    .replace('.', "")
    .trim()
    .to_string();

  let prefix = match strategy {
    TestStrategy::BehaviorVerification | TestStrategy::Auto => "Verify",
    TestStrategy::OutputValidation => "Confirm",
    TestStrategy::IntegrationCheck => "Ensure",
    TestStrategy::ErrorHandling => "Test error handling for",
    TestStrategy::PerformanceMetric => "Validate performance of",
  };

  let test_body = format_test_body(&cleaned_element, strategy, context);

  format!("{prefix} {test_body}")
}

/// Infer test strategy from bead context
fn infer_strategy_from_context(context: &SynthesisContext) -> TestStrategy {
  let title_lower = context.bead_title.to_lowercase();
  let answer_lower = context.ai_answer.to_lowercase();
  let combined = format!("{title_lower} {answer_lower}");

  // Check for error handling keywords
  let error_keywords = ["error", "fail", "exception", "timeout", "edge case"];
  let has_error_keywords = contains_any_ignore_case(&combined, &error_keywords);

  // Check for performance keywords
  let performance_keywords = ["performance", "latency", "response time", "throughput"];
  let has_performance_keywords = contains_any_ignore_case(&combined, &performance_keywords);

  // Check for integration keywords
  let integration_keywords = ["integration", "connect", "api", "endpoint", "service"];
  let has_integration_keywords = contains_any_ignore_case(&combined, &integration_keywords);

  match (
    has_error_keywords,
    has_performance_keywords,
    has_integration_keywords,
    context.phase > 2,
  ) {
    (true, _, _, _) => TestStrategy::ErrorHandling,
    (_, true, _, _) => TestStrategy::PerformanceMetric,
    (_, _, true, _) => TestStrategy::IntegrationCheck,
    (_, _, _, true) => TestStrategy::BehaviorVerification,
    (_, _, _, false) => TestStrategy::OutputValidation,
  }
}

/// Generate dependency-aware tests
fn generate_dependency_tests(context: &SynthesisContext) -> Vec<String> {
  context
    .dependencies
    .iter()
    .take(3) // Limit to prevent too many tests
    .map(|dep| format!("Verify integration with {dep} is working correctly"))
    .collect()
}

/// Generate phase-specific tests
fn generate_phase_tests(context: &SynthesisContext, _strategy: TestStrategy) -> Vec<String> {
  match context.phase {
    1 => vec![
      "Verify module compiles without errors".to_string(),
      "Confirm basic functionality works with minimal input".to_string(),
    ],
    2 => vec![
      "Verify integration with earlier phase components".to_string(),
      "Test with realistic data scenarios".to_string(),
    ],
    _ => vec![
      "Verify end-to-end workflow completes successfully".to_string(),
      "Test with production-like scenarios".to_string(),
    ],
  }
}

/// Format test body based on strategy
fn format_test_body(element: &str, strategy: TestStrategy, _context: &SynthesisContext) -> String {
  match strategy {
    TestStrategy::BehaviorVerification => {
      if element.len() > 50 {
        format!("{}... works as expected", &element[..50])
      } else {
        format!("{element} works as expected")
      }
    }
    TestStrategy::OutputValidation => format!("{element} produces expected output format"),
    TestStrategy::IntegrationCheck => format!("integration with {element} is properly established"),
    TestStrategy::ErrorHandling => {
      if contains_any_ignore_case(element, &["timeout", "network", "external"]) {
        format!("{element} is handled gracefully")
      } else {
        format!("proper error handling for {element}")
      }
    }
    TestStrategy::PerformanceMetric => format!("{element} meets performance requirements"),
    TestStrategy::Auto => format!("{element} behaves correctly"),
  }
}

/// Deduplicate tests while preserving order
fn dedupe_tests(tests: Vec<String>) -> Vec<String> {
  let mut seen = HashSet::new();
  tests
    .into_iter()
    .filter(|test| {
      let key = test.trim().to_lowercase();
      if seen.contains(&key) {
        false
      } else {
        seen.insert(key);
        true
      }
    })
    .collect()
}

/// Ensure minimum number of tests per bead
fn ensure_minimum_tests(tests: Vec<String>, context: &SynthesisContext) -> Vec<String> {
  const MIN_TESTS: usize = 3;

  if tests.len() >= MIN_TESTS {
    tests
  } else {
    let needed = MIN_TESTS.saturating_sub(tests.len());
    let fallback_tests = generate_fallback_tests(context, needed);
    tests.into_iter().chain(fallback_tests).collect()
  }
}

/// Generate fallback tests to meet minimum requirements
fn generate_fallback_tests(context: &SynthesisContext, count: usize) -> Vec<String> {
  let fallback_templates = [
    format!(
      "Verify {} meets specification requirements",
      context.bead_title
    ),
    format!("Test {} with valid inputs", context.bead_title),
    format!("Verify {} handles edge cases correctly", context.bead_title),
    format!("Confirm {} produces expected outputs", context.bead_title),
    format!(
      "Test {} integration with dependent components",
      context.bead_title
    ),
  ];

  fallback_templates.into_iter().take(count).collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {
  use super::*;

  fn make_test_context() -> SynthesisContext {
    SynthesisContext {
      session_id: "test-session".to_string(),
      bead_id: "bead-1".to_string(),
      bead_title: "User Authentication".to_string(),
      ai_answer: "Implement user login with email validation. Create session tokens.".to_string(),
      phase: 1,
      dependencies: vec!["Database".to_string(), "Email Service".to_string()],
    }
  }

  #[test]
  fn test_synthesize_acceptance_tests_returns_minimum_tests() {
    let context = make_test_context();
    let tests = synthesize_acceptance_tests(&context, TestStrategy::Auto);
    assert!(tests.len() >= 3);
  }

  #[test]
  fn test_extract_testable_elements_finds_keywords() {
    let answer = "Implement the login feature. Create a session token.";
    let elements = extract_testable_elements(answer);
    assert!(!elements.is_empty());
    assert!(elements.iter().any(|e| e.contains("login")));
  }

  #[test]
  fn test_extract_testable_elements_returns_fallback_for_empty() {
    let elements = extract_testable_elements("");
    assert_eq!(elements, vec!["Complete the implementation"]);
  }

  #[test]
  fn test_format_acceptance_test_behavior_verification() {
    let context = make_test_context();
    let test = format_acceptance_test("user login", TestStrategy::BehaviorVerification, &context);
    assert!(test.starts_with("Verify"));
    assert!(test.contains("works as expected"));
  }

  #[test]
  fn test_format_acceptance_test_error_handling() {
    let context = make_test_context();
    let test = format_acceptance_test("network timeout", TestStrategy::ErrorHandling, &context);
    assert!(test.starts_with("Test error handling for"));
    assert!(test.contains("handled gracefully"));
  }

  #[test]
  fn test_infer_strategy_detects_error_keywords() {
    let mut context = make_test_context();
    context.bead_title = "Error Handler".to_string();
    let strategy = infer_strategy_from_context(&context);
    assert_eq!(strategy, TestStrategy::ErrorHandling);
  }

  #[test]
  fn test_infer_strategy_detects_performance_keywords() {
    let mut context = make_test_context();
    context.ai_answer = "Optimize performance for high throughput".to_string();
    let strategy = infer_strategy_from_context(&context);
    assert_eq!(strategy, TestStrategy::PerformanceMetric);
  }

  #[test]
  fn test_infer_strategy_detects_integration_keywords() {
    let mut context = make_test_context();
    context.ai_answer = "Create API endpoint for user service".to_string();
    let strategy = infer_strategy_from_context(&context);
    assert_eq!(strategy, TestStrategy::IntegrationCheck);
  }

  #[test]
  fn test_generate_dependency_tests_limits_to_three() {
    let mut context = make_test_context();
    context.dependencies = vec![
      "A".to_string(),
      "B".to_string(),
      "C".to_string(),
      "D".to_string(),
    ];
    let tests = generate_dependency_tests(&context);
    assert_eq!(tests.len(), 3);
  }

  #[test]
  fn test_generate_phase_tests_phase_1() {
    let mut context = make_test_context();
    context.phase = 1;
    let tests = generate_phase_tests(&context, TestStrategy::Auto);
    assert!(tests.iter().any(|t| t.contains("compiles")));
  }

  #[test]
  fn test_generate_phase_tests_phase_3() {
    let mut context = make_test_context();
    context.phase = 3;
    let tests = generate_phase_tests(&context, TestStrategy::Auto);
    assert!(tests.iter().any(|t| t.contains("end-to-end")));
  }

  #[test]
  fn test_dedupe_tests_removes_duplicates() {
    let tests = vec![
      "Test A".to_string(),
      "test a".to_string(),
      "Test B".to_string(),
    ];
    let deduped = dedupe_tests(tests);
    assert_eq!(deduped.len(), 2);
  }

  #[test]
  fn test_ensure_minimum_tests_adds_fallbacks() {
    let context = make_test_context();
    let tests = vec!["Single test".to_string()];
    let result = ensure_minimum_tests(tests, &context);
    assert!(result.len() >= 3);
  }
}
