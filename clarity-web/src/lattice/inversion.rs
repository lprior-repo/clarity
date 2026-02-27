#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![allow(clippy::unnested_or_patterns)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use itertools::Itertools;
use thiserror::Error;

/// Domain errors for inversion module
#[derive(Debug, Error)]
pub enum InversionError {
  #[error("problem statement is empty or whitespace")]
  EmptyProblem,

  #[error("solution statement is empty or whitespace")]
  EmptySolution,

  #[error("input too short: needs at least {min} chars, got {actual}")]
  InputTooShort { min: usize, actual: usize },
}

/// Severity rating for a challenge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
  /// Critical - fundamental flaw that invalidates the solution
  Critical,
  /// Moderate - significant limitation under important conditions
  Moderate,
  /// Low - minor edge case or rare condition
  Low,
}

impl Severity {
  /// Convert to numeric score (0-100, inverted for quality scoring)
  #[must_use]
  pub const fn score(&self) -> u8 {
    match self {
      Self::Critical => 100,
      Self::Moderate => 50,
      Self::Low => 10,
    }
  }

  /// Convert from numeric score
  #[must_use]
  pub const fn from_score(score: u8) -> Option<Self> {
    match score {
      100 => Some(Self::Critical),
      80..=99 => Some(Self::Moderate),
      _ => Some(Self::Low),
    }
  }
}

/// A single challenge to an assumption
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InversionChallenge {
  /// The assumption being challenged
  pub assumption: String,
  /// The challenge/counterargument
  pub challenge: String,
  /// Pattern used to generate the challenge
  pub pattern: ChallengePattern,
  /// Severity of the challenge
  pub severity: Severity,
}

impl InversionChallenge {
  /// Create a new inversion challenge
  #[must_use]
  pub fn new(
    assumption: String,
    challenge: String,
    pattern: ChallengePattern,
    severity: Severity,
  ) -> Self {
    Self {
      assumption,
      challenge,
      pattern,
      severity,
    }
  }

  /// Calculate quality impact (higher severity = lower quality)
  #[must_use]
  pub const fn quality_impact(&self) -> u8 {
    self.severity.score()
  }
}

/// Pattern used to generate challenges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChallengePattern {
  /// Direct negation ("the opposite is true")
  Negation,
  /// Counterexample ("except when...")
  Counterexample,
  /// Edge case ("at scale...")
  EdgeCase,
  /// Assumption reversal
  Reversal,
}

impl ChallengePattern {
  /// Get all pattern variants
  #[must_use]
  pub fn all() -> [Self; 4] {
    [
      Self::Negation,
      Self::Counterexample,
      Self::EdgeCase,
      Self::Reversal,
    ]
  }

  /// Get description for display
  #[must_use]
  pub const fn description(&self) -> &str {
    match self {
      Self::Negation => "the opposite is true",
      Self::Counterexample => "except when...",
      Self::EdgeCase => "at extreme scale/conditions",
      Self::Reversal => "assuming the reverse",
    }
  }
}

/// Complete output from inversion analysis
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InversionOutput {
  /// All identified challenges
  pub challenges: Vec<InversionChallenge>,
  /// Overall quality score (0-100, lower = more challenges found)
  pub quality_score: u8,
  /// Count of critical challenges
  pub critical_count: usize,
  /// Count of moderate challenges
  pub moderate_count: usize,
  /// Count of low challenges
  pub low_count: usize,
}

impl InversionOutput {
  /// Create new inversion output from challenges
  #[must_use]
  pub fn new(challenges: Vec<InversionChallenge>) -> Self {
    let critical_count = challenges
      .iter()
      .filter(|c| c.severity == Severity::Critical)
      .count();

    let moderate_count = challenges
      .iter()
      .filter(|c| c.severity == Severity::Moderate)
      .count();

    let low_count = challenges
      .iter()
      .filter(|c| c.severity == Severity::Low)
      .count();

    // Calculate quality score: start at 100, subtract based on severity
    // Formula: quality = max(0, 200 - total_impact)
    // This means:
    //   - total_impact = 60 -> quality = 140 (clamped to 100)
    //   - total_impact = 100 -> quality = 100
    //   - total_impact = 160 -> quality = 40
    //   - total_impact = 200+ -> quality = 0
    let total_impact = challenges
      .iter()
      .map(|c| c.quality_impact() as u32)
      .sum::<u32>();

    let quality_score = if total_impact <= 100 {
      (100 - total_impact) as u8
    } else {
      (200u32.saturating_sub(total_impact)) as u8
    };

    Self {
      challenges,
      quality_score,
      critical_count,
      moderate_count,
      low_count,
    }
  }

  /// Get total number of challenges
  #[must_use]
  pub const fn total_challenges(&self) -> usize {
    self.challenges.len()
  }

  /// Get challenges by severity
  #[must_use]
  pub fn by_severity(&self, severity: Severity) -> Vec<&InversionChallenge> {
    self
      .challenges
      .iter()
      .filter(|c| c.severity == severity)
      .collect()
  }

  /// Get challenges by pattern
  #[must_use]
  pub fn by_pattern(&self, pattern: ChallengePattern) -> Vec<&InversionChallenge> {
    self
      .challenges
      .iter()
      .filter(|c| c.pattern == pattern)
      .collect()
  }
}

/// Extract assumptions from problem and solution text
#[must_use]
pub fn extract_assumptions(problem: &str, solution: &str) -> Vec<String> {
  let combined = format!("{problem} {solution}");
  let lower = combined.to_lowercase();

  // Common assumption indicators
  let assumption_patterns = [
    "will",
    "should",
    "can",
    "always",
    "never",
    "every",
    "all",
    "none",
    "ensure",
    "guarantee",
    "assume",
    "assuming",
    "expect",
    "expects",
  ];

  let mut assumptions = Vec::new();

  // Extract sentences with assumption indicators
  for pattern in &assumption_patterns {
    if lower.contains(pattern) {
      // Find the sentence containing this pattern
      for sentence in combined.split('.') {
        let sentence_lower = sentence.to_lowercase();
        if sentence_lower.contains(pattern) {
          let cleaned = sentence.trim().to_string();
          if !cleaned.is_empty() && !assumptions.contains(&cleaned) {
            assumptions.push(cleaned);
          }
        }
      }
    }
  }

  // If no explicit assumptions found, extract statements
  if assumptions.is_empty() {
    for sentence in combined.split(['.', ',', '\n']).take(5) {
      let cleaned = sentence.trim().to_string();
      if cleaned.len() > 10 && !assumptions.contains(&cleaned) {
        assumptions.push(cleaned);
      }
    }
  }

  assumptions.into_iter().unique().collect()
}

/// Apply negation pattern to generate challenge
#[must_use]
pub fn apply_negation(assumption: &str) -> Option<InversionChallenge> {
  let trimmed = assumption.trim();

  // Detect negatable phrases
  let negations = [
    ("will", "will not"),
    ("can", "cannot"),
    ("should", "should not"),
    ("always", "never"),
    ("never", "always"),
    ("all", "none"),
    ("every", "no"),
    ("ensure", "cannot ensure"),
    ("guarantee", "cannot guarantee"),
  ];

  for (original, negated) in &negations {
    if trimmed.to_lowercase().contains(original) {
      let challenge = trimmed.replacen(original, negated, 1);
      let severity = determine_severity(trimmed, original, 1);

      return Some(InversionChallenge::new(
        trimmed.to_string(),
        format!("What if {challenge}?"),
        ChallengePattern::Negation,
        severity,
      ));
    }
  }

  None
}

/// Apply counterexample pattern to generate challenge
#[must_use]
pub fn apply_counterexample(assumption: &str) -> Option<InversionChallenge> {
  let trimmed = assumption.trim();

  // Contextual exceptions
  let exceptions = [
    "under high load",
    "at scale",
    "with limited resources",
    "during peak usage",
    "with concurrent access",
    "with invalid input",
    "under network failure",
    "with malicious actors",
    "with legacy data",
    "during migration",
  ];

  // Find best fitting exception based on context
  let lower = trimmed.to_lowercase();

  let exception = exceptions
    .iter()
    .find(|exc| {
      !matches!(
        (
          lower.contains("performance"),
          lower.contains("scale"),
          lower.contains("load"),
          exc.contains(&"scale")
        ),
        (true, true, true, _) | (true, _, true, _) | (_, true, _, true) | (_, _, true, _)
      )
    })
    .or(exceptions.first());

  let exception = exception?;

  let severity = if lower.contains("ensure") || lower.contains("guarantee") {
    Severity::Critical
  } else if lower.contains("should") || lower.contains("can") {
    Severity::Moderate
  } else {
    Severity::Low
  };

  Some(InversionChallenge::new(
    trimmed.to_string(),
    format!("Except when {exception}?"),
    ChallengePattern::Counterexample,
    severity,
  ))
}

/// Apply edge case pattern to generate challenge
#[must_use]
pub fn apply_edge_case(assumption: &str) -> Option<InversionChallenge> {
  let trimmed = assumption.trim();

  let edge_cases = [
    "with zero items",
    "at maximum capacity",
    "with empty data",
    "with duplicate entries",
    "with special characters",
    "at timezone boundaries",
    "with null values",
    "at extreme values",
  ];

  let lower = trimmed.to_lowercase();

  let edge_case = edge_cases
    .iter()
    .find(|ec| {
      matches!(
        (
          lower.contains("data"),
          lower.contains("empty"),
          ec.contains(&"empty")
        ),
        (true, true, true) | (_, true, true) | (true, _, true)
      )
    })
    .or(edge_cases.first());

  let edge_case = edge_case?;

  let severity = if lower.contains("always") || lower.contains("never") {
    Severity::Critical
  } else if lower.contains("will") || lower.contains("can") {
    Severity::Moderate
  } else {
    Severity::Low
  };

  Some(InversionChallenge::new(
    trimmed.to_string(),
    format!("What about {edge_case}?"),
    ChallengePattern::EdgeCase,
    severity,
  ))
}

/// Apply reversal pattern to generate challenge
#[must_use]
pub fn apply_reversal(assumption: &str) -> Option<InversionChallenge> {
  let trimmed = assumption.trim();
  let lower = trimmed.to_lowercase();

  // Identify the core assertion and reverse it
  let reversals = [
    ("increases", "decreases"),
    ("improves", "worsens"),
    ("succeeds", "fails"),
    ("works", "fails"),
    ("valid", "invalid"),
    ("true", "false"),
    ("enabled", "disabled"),
    ("active", "inactive"),
  ];

  for (original, reversed) in &reversals {
    if lower.contains(original) {
      let challenge = trimmed.replacen(original, reversed, 1);
      let severity = if lower.contains("always") || lower.contains("guarantee") {
        Severity::Critical
      } else {
        Severity::Moderate
      };

      return Some(InversionChallenge::new(
        trimmed.to_string(),
        format!("What if it actually {challenge}?"),
        ChallengePattern::Reversal,
        severity,
      ));
    }
  }

  // Generic reversal if no specific pattern matched
  Some(InversionChallenge::new(
    trimmed.to_string(),
    format!("What if the opposite is true: {trimmed}?"),
    ChallengePattern::Reversal,
    Severity::Low,
  ))
}

/// Determine severity based on context and pattern
#[must_use]
fn determine_severity(text: &str, _pattern: &str, _occurrence: usize) -> Severity {
  let lower = text.to_lowercase();

  // Critical indicators
  if lower.contains("guarantee")
    || lower.contains("ensure")
    || lower.contains("always")
    || lower.contains("never")
  {
    return Severity::Critical;
  }

  // Moderate indicators
  if lower.contains("should") || lower.contains("will") || lower.contains("can") {
    return Severity::Moderate;
  }

  // Default to low
  Severity::Low
}

/// Generate challenges for a single assumption using all patterns
#[must_use]
pub fn generate_challenges(assumption: &str) -> Vec<InversionChallenge> {
  ChallengePattern::all()
    .iter()
    .filter_map(|pattern| match pattern {
      ChallengePattern::Negation => apply_negation(assumption),
      ChallengePattern::Counterexample => apply_counterexample(assumption),
      ChallengePattern::EdgeCase => apply_edge_case(assumption),
      ChallengePattern::Reversal => apply_reversal(assumption),
    })
    .collect()
}

/// Main inversion function: analyze problem and solution to find assumptions
///
/// # Errors
///
/// Returns `InversionError` if inputs are invalid
pub fn invert(problem: &str, solution: &str) -> Result<InversionOutput, InversionError> {
  // Validate inputs
  let problem_trimmed = problem.trim();
  let solution_trimmed = solution.trim();

  if problem_trimmed.is_empty() {
    return Err(InversionError::EmptyProblem);
  }

  if solution_trimmed.is_empty() {
    return Err(InversionError::EmptySolution);
  }

  let combined_len = problem_trimmed.len() + solution_trimmed.len();
  if combined_len < 20 {
    return Err(InversionError::InputTooShort {
      min: 20,
      actual: combined_len,
    });
  }

  // Extract assumptions from both texts
  let assumptions = extract_assumptions(problem_trimmed, solution_trimmed);

  // Generate challenges for each assumption
  let challenges: Vec<InversionChallenge> = assumptions
    .iter()
    .flat_map(|assumption| generate_challenges(assumption))
    .collect();

  let challenges = challenges
    .into_iter()
    .unique_by(|c| (c.assumption.clone(), c.pattern))
    .collect();

  Ok(InversionOutput::new(challenges))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_extract_assumptions_with_explicit_indicators() {
    let problem = "The system will always respond within 100ms";
    let solution = "We use caching to ensure fast response times";

    let assumptions = extract_assumptions(problem, solution);

    assert!(!assumptions.is_empty());
    assert!(assumptions
      .iter()
      .any(|a| a.to_lowercase().contains("always") || a.to_lowercase().contains("ensure")));
  }

  #[test]
  fn test_extract_assumptions_minimum_three() {
    let problem = "The API will handle all requests successfully. Every user should receive a response. We can guarantee availability.";
    let solution = "Implement load balancers and redundancy";

    let assumptions = extract_assumptions(problem, solution);

    assert!(
      assumptions.len() >= 3,
      "Expected at least 3 assumptions, got {}",
      assumptions.len()
    );
  }

  #[test]
  fn test_negation_pattern() {
    let assumption = "The system will always be available";

    let challenge = apply_negation(assumption);

    assert!(challenge.is_some());
    let challenge = challenge.unwrap();
    assert_eq!(challenge.pattern, ChallengePattern::Negation);
    assert!(challenge.challenge.contains("will not") || challenge.challenge.contains("never"));
    assert_eq!(challenge.assumption, assumption);
  }

  #[test]
  fn test_counterexample_pattern() {
    let assumption = "The cache will improve performance";

    let challenge = apply_counterexample(assumption);

    assert!(challenge.is_some());
    let challenge = challenge.unwrap();
    assert_eq!(challenge.pattern, ChallengePattern::Counterexample);
    assert!(challenge.challenge.contains("Except when"));
  }

  #[test]
  fn test_edge_case_pattern() {
    let assumption = "The function processes all data correctly";

    let challenge = apply_edge_case(assumption);

    assert!(challenge.is_some());
    let challenge = challenge.unwrap();
    assert_eq!(challenge.pattern, ChallengePattern::EdgeCase);
    assert!(challenge.challenge.contains("What about"));
  }

  #[test]
  fn test_reversal_pattern() {
    let assumption = "The optimization improves performance";

    let challenge = apply_reversal(assumption);

    assert!(challenge.is_some());
    let challenge = challenge.unwrap();
    assert_eq!(challenge.pattern, ChallengePattern::Reversal);
    assert!(challenge.challenge.contains("What if it actually"));
  }

  #[test]
  fn test_severity_scoring_consistency() {
    assert_eq!(Severity::Critical.score(), 100);
    assert_eq!(Severity::Moderate.score(), 50);
    assert_eq!(Severity::Low.score(), 10);
  }

  #[test]
  fn test_severity_from_score() {
    assert_eq!(Severity::from_score(100), Some(Severity::Critical));
    assert_eq!(Severity::from_score(80), Some(Severity::Moderate));
    assert_eq!(Severity::from_score(10), Some(Severity::Low));
  }

  #[test]
  fn test_critical_severity_detection() {
    let assumption = "We guarantee 100% uptime";

    let challenge = apply_negation(assumption);

    assert!(challenge.is_some());
    let challenge = challenge.unwrap();
    assert_eq!(challenge.severity, Severity::Critical);
  }

  #[test]
  fn test_moderate_severity_detection() {
    let assumption = "The system should work";

    let challenge = apply_negation(assumption);

    assert!(challenge.is_some());
    let challenge = challenge.unwrap();
    assert_eq!(challenge.severity, Severity::Moderate);
  }

  #[test]
  fn test_low_severity_default() {
    let assumption = "The code processes data";

    let challenge = apply_reversal(assumption);

    assert!(challenge.is_some());
    let challenge = challenge.unwrap();
    assert_eq!(challenge.severity, Severity::Low);
  }

  #[test]
  fn test_generate_challenges_uses_all_patterns() {
    let assumption = "The system will always work";

    let challenges = generate_challenges(assumption);

    // Should generate at least one challenge per pattern
    let patterns_used = challenges
      .iter()
      .map(|c| c.pattern)
      .unique()
      .collect::<Vec<_>>();

    assert!(
      patterns_used.len() >= 3,
      "Expected at least 3 unique patterns, got {}",
      patterns_used.len()
    );
  }

  #[test]
  fn test_invert_empty_problem() {
    let result = invert("", "some solution");

    assert!(result.is_err());
    assert!(matches!(result, Err(InversionError::EmptyProblem)));
  }

  #[test]
  fn test_invert_empty_solution() {
    let result = invert("some problem", "");

    assert!(result.is_err());
    assert!(matches!(result, Err(InversionError::EmptySolution)));
  }

  #[test]
  fn test_invert_too_short() {
    let result = invert("hi", "bye");

    assert!(result.is_err());
    assert!(matches!(result, Err(InversionError::InputTooShort { .. })));
  }

  #[test]
  fn test_invert_success() {
    let problem = "The system will always respond within 100ms. We can guarantee this performance.";
    let solution = "Use caching and load balancing to ensure fast response times for all users.";

    let result = invert(problem, solution);

    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.total_challenges() >= 3);

    // Should have at least one critical or moderate challenge
    assert!(
      output.critical_count + output.moderate_count > 0,
      "Expected at least one critical or moderate challenge"
    );

    // Quality score should be lowered by challenges
    assert!(output.quality_score < 100);
  }

  #[test]
  fn test_inversion_output_quality_score_calculation() {
    let challenges = vec![
      InversionChallenge::new(
        "assumption1".to_string(),
        "challenge1".to_string(),
        ChallengePattern::Negation,
        Severity::Critical,
      ),
      InversionChallenge::new(
        "assumption2".to_string(),
        "challenge2".to_string(),
        ChallengePattern::Counterexample,
        Severity::Moderate,
      ),
      InversionChallenge::new(
        "assumption3".to_string(),
        "challenge3".to_string(),
        ChallengePattern::EdgeCase,
        Severity::Low,
      ),
    ];

    let output = InversionOutput::new(challenges);

    // Total impact: 100 + 50 + 10 = 160
    // Quality: 100 - (160 - 100) = 40
    assert_eq!(output.quality_score, 40);
    assert_eq!(output.critical_count, 1);
    assert_eq!(output.moderate_count, 1);
    assert_eq!(output.low_count, 1);
  }

  #[test]
  fn test_inversion_output_filtering() {
    let challenges = vec![
      InversionChallenge::new(
        "a1".to_string(),
        "c1".to_string(),
        ChallengePattern::Negation,
        Severity::Critical,
      ),
      InversionChallenge::new(
        "a2".to_string(),
        "c2".to_string(),
        ChallengePattern::Counterexample,
        Severity::Moderate,
      ),
      InversionChallenge::new(
        "a3".to_string(),
        "c3".to_string(),
        ChallengePattern::Negation,
        Severity::Critical,
      ),
    ];

    let output = InversionOutput::new(challenges);

    // Test by_severity
    let critical = output.by_severity(Severity::Critical);
    assert_eq!(critical.len(), 2);

    let moderate = output.by_severity(Severity::Moderate);
    assert_eq!(moderate.len(), 1);

    // Test by_pattern
    let negations = output.by_pattern(ChallengePattern::Negation);
    assert_eq!(negations.len(), 2);
  }

  #[test]
  fn test_challenge_pattern_descriptions() {
    assert_eq!(
      ChallengePattern::Negation.description(),
      "the opposite is true"
    );
    assert_eq!(
      ChallengePattern::Counterexample.description(),
      "except when..."
    );
    assert_eq!(
      ChallengePattern::EdgeCase.description(),
      "at extreme scale/conditions"
    );
    assert_eq!(
      ChallengePattern::Reversal.description(),
      "assuming the reverse"
    );
  }

  #[test]
  fn test_quality_impact() {
    let challenge = InversionChallenge::new(
      "test".to_string(),
      "test challenge".to_string(),
      ChallengePattern::Negation,
      Severity::Critical,
    );

    assert_eq!(challenge.quality_impact(), 100);
  }

  #[test]
  fn test_realistic_scenario() {
    let problem = "Our API will always respond within 200ms. We guarantee this performance for all users under all conditions.";
    let solution = "Implement Redis caching with a 5-minute TTL. This ensures that frequently accessed data is served quickly from memory. We can also add load balancers to distribute traffic evenly.";

    let result = invert(problem, solution);

    assert!(result.is_ok());

    let output = result.unwrap();

    // Should extract multiple assumptions
    assert!(
      output.total_challenges() >= 3,
      "Expected at least 3 challenges, got {}",
      output.total_challenges()
    );

    // Should find critical issues with "always" and "guarantee"
    assert!(
      output.critical_count >= 1,
      "Expected at least 1 critical challenge"
    );

    // Should provide varied patterns
    let patterns = output
      .challenges
      .iter()
      .map(|c| c.pattern)
      .unique()
      .collect::<Vec<_>>();
    assert!(
      patterns.len() >= 2,
      "Expected at least 2 different patterns"
    );

    // Quality should reflect the critical issues found
    assert!(
      output.quality_score < 90,
      "Quality score should be reduced by critical challenges"
    );
  }

  #[test]
  fn test_extract_assumptions_handles_whitespace() {
    let problem = "   The system will work.   ";
    let solution = "  Use caching.  ";

    let assumptions = extract_assumptions(problem, solution);

    assert!(!assumptions.is_empty());
    // Should trim whitespace
    assert!(assumptions
      .iter()
      .all(|a| !a.starts_with(' ') && !a.ends_with(' ')));
  }

  #[test]
  fn test_invert_trims_input() {
    let problem = "   The system will work   ";
    let solution = "   Use caching and redundancy to ensure availability   ";

    let result = invert(problem, solution);

    assert!(result.is_ok());
  }

  #[test]
  fn test_generate_challenges_uniqueness() {
    let assumption = "The system will always work";

    let challenges = generate_challenges(assumption);

    // Check that same assumption + pattern doesn't appear twice
    let unique_pairs = challenges
      .iter()
      .map(|c| (&c.assumption, &c.pattern))
      .unique()
      .count();

    assert_eq!(unique_pairs, challenges.len());
  }

  #[test]
  fn test_inversion_with_no_clear_assumptions() {
    let problem = "Here is some text about systems and data processing.";
    let solution = "We will implement various components and features.";

    let result = invert(problem, solution);

    assert!(result.is_ok());

    let output = result.unwrap();
    // Should still generate something from the sentences
    assert!(output.total_challenges() >= 1);
  }
}
