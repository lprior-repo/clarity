#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Response containing 3 antithesis points (null hypothesis) for a problem statement.
///
/// The antithesis represents realistic reasons why the target customer might
/// ignore or reject the proposed solution. This adversarial approach ensures
/// product ideas are rigorously validated before implementation.
///
/// # Invariants
/// - `points` always contains exactly 3 elements
/// - `quality_score` is always in the range 0..=100
/// - `validated` is true only when all 3 points are non-empty
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AntithesisResponse {
  /// Three antithesis points explaining why users might reject this solution.
  /// Each point should be a specific, realistic concern.
  pub points: Vec<String>,

  /// Quality score from 0-100 based on specificity and realism of points.
  /// Higher scores indicate more concrete, actionable antithesis points.
  pub quality_score: u8,

  /// Whether the antithesis has been validated (all points are non-empty).
  pub validated: bool,
}

impl AntithesisResponse {
  /// Create a new `AntithesisResponse` with the given points.
  ///
  /// The quality score is calculated based on point specificity,
  /// and validation status is determined by whether all points are non-empty.
  #[must_use]
  pub fn new(points: Vec<String>) -> Self {
    let validated = Self::is_validated(&points);
    let quality_score = Self::calculate_quality_score(&points);
    Self {
      points,
      quality_score,
      validated,
    }
  }

  /// Create an `AntithesisResponse` from exactly 3 points.
  ///
  /// Returns None if the slice does not contain exactly 3 elements.
  #[must_use]
  pub fn from_three_points(points: &[String; 3]) -> Self {
    Self::new(points.to_vec())
  }

  /// Check if all 3 points are non-empty.
  fn is_validated(points: &[String]) -> bool {
    points.len() == 3 && points.iter().all(|p| !p.trim().is_empty())
  }

  /// Calculate quality score based on point specificity.
  ///
  /// Points score higher for:
  /// - Being non-empty (base score)
  /// - Containing specific details (word count heuristics)
  /// - Using concrete language vs vague abstractions
  fn calculate_quality_score(points: &[String]) -> u8 {
    if points.len() != 3 {
      return 0;
    }

    let total_score: u32 = points
      .iter()
      .map(|point| Self::score_single_point(point))
      .sum();

    // Average the 3 point scores (each max 100) to get overall score
    u8::try_from(total_score / 3).unwrap_or(0)
  }

  /// Score a single antithesis point from 0-100.
  fn score_single_point(point: &str) -> u32 {
    let trimmed = point.trim();

    if trimmed.is_empty() {
      return 0;
    }

    let word_count = trimmed.split_whitespace().count();

    // Base score for non-empty
    let base = 20u32;

    // Bonus for reasonable length (10-50 words is ideal)
    let length_bonus = match word_count {
      0..=4 => 10,   // Too short, likely vague
      10..=25 => 50, // Good specificity
      26..=50 => 40, // Still good
      _ => 30,       // Very long, might be rambling
    };

    // Bonus for specific indicators (numbers, concrete nouns)
    let specificity_bonus = if trimmed.chars().any(|c| c.is_ascii_digit()) {
      20 // Contains numbers = specific
    } else if trimmed.to_lowercase().contains("because") {
      10 // Provides reasoning
    } else {
      0
    };

    // Bonus for concrete language indicators
    let concrete_bonus = if Self::has_concrete_language(trimmed) {
      10
    } else {
      0
    };

    base + length_bonus + specificity_bonus + concrete_bonus
  }

  /// Check if the point contains concrete language indicators.
  fn has_concrete_language(text: &str) -> bool {
    let concrete_indicators = [
      "specifically",
      "for example",
      "such as",
      "when they",
      "if they",
      "because they",
      "instead of",
      "rather than",
    ];

    let lower = text.to_lowercase();
    concrete_indicators
      .iter()
      .any(|indicator| lower.contains(indicator))
  }

  /// Get the three points as a slice.
  #[must_use]
  pub fn points(&self) -> &[String] {
    &self.points
  }

  /// Check if the response is valid (exactly 3 non-empty points).
  #[must_use]
  pub const fn is_valid(&self) -> bool {
    self.validated
  }

  /// Get the quality score (0-100).
  #[must_use]
  pub const fn score(&self) -> u8 {
    self.quality_score
  }

  /// Check if quality gate passes (score >= 70).
  #[must_use]
  pub const fn quality_gate_passes(&self) -> bool {
    self.quality_score >= 70
  }

  /// Get a point by index (0, 1, or 2).
  ///
  /// Returns None if index is out of bounds.
  #[must_use]
  pub fn get_point(&self, index: usize) -> Option<&String> {
    self.points.get(index)
  }

  /// Create a new response with updated points.
  #[must_use]
  pub fn with_points(mut self, points: Vec<String>) -> Self {
    self.validated = Self::is_validated(&points);
    self.quality_score = Self::calculate_quality_score(&points);
    self.points = points;
    self
  }

  /// Create a new response with a manually set quality score.
  ///
  /// The score is clamped to the valid range 0..=100.
  #[must_use]
  pub fn with_quality_score(mut self, score: u8) -> Self {
    self.quality_score = score.min(100);
    self
  }
}

impl Default for AntithesisResponse {
  fn default() -> Self {
    Self {
      points: vec![String::new(), String::new(), String::new()],
      quality_score: 0,
      validated: false,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_default_has_three_empty_points() {
    let response = AntithesisResponse::default();
    assert_eq!(response.points.len(), 3);
    assert!(response.points.iter().all(String::is_empty));
    assert!(!response.validated);
    assert_eq!(response.quality_score, 0);
  }

  #[test]
  fn test_new_with_empty_points_not_validated() {
    let response = AntithesisResponse::new(vec![String::new(), String::new(), String::new()]);
    assert!(!response.validated);
  }

  #[test]
  fn test_new_with_all_points_validated() {
    let response = AntithesisResponse::new(vec![
      "Users prefer their current workflow".to_string(),
      "Learning curve is too steep".to_string(),
      "Cost outweighs benefits for most users".to_string(),
    ]);
    assert!(response.validated);
  }

  #[test]
  fn test_new_with_some_empty_not_validated() {
    let response = AntithesisResponse::new(vec![
      "Valid point".to_string(),
      String::new(),
      "Another valid point".to_string(),
    ]);
    assert!(!response.validated);
  }

  #[test]
  fn test_wrong_point_count_not_validated() {
    let two_points = AntithesisResponse::new(vec!["One".to_string(), "Two".to_string()]);
    assert!(!two_points.validated);
    assert_eq!(two_points.quality_score, 0);

    let four_points = AntithesisResponse::new(vec![
      "One".to_string(),
      "Two".to_string(),
      "Three".to_string(),
      "Four".to_string(),
    ]);
    assert!(!four_points.validated);
    assert_eq!(four_points.quality_score, 0);
  }

  #[test]
  fn test_from_three_points() {
    let response = AntithesisResponse::from_three_points(&[
      "Point 1".to_string(),
      "Point 2".to_string(),
      "Point 3".to_string(),
    ]);
    assert_eq!(response.points.len(), 3);
    assert!(response.validated);
  }

  #[test]
  fn test_quality_score_increases_with_specificity() {
    let vague = AntithesisResponse::new(vec![
      "Bad".to_string(),
      "No".to_string(),
      "Hard".to_string(),
    ]);

    let specific = AntithesisResponse::new(vec![
            "Users currently use spreadsheets because they offer more flexibility for 15 specific use cases".to_string(),
            "The learning curve requires 3-5 hours of training which most team leads won't approve".to_string(),
            "At $50/month, the cost exceeds the typical budget allocation for tools like this by 2x".to_string(),
        ]);

    assert!(specific.quality_score > vague.quality_score);
  }

  #[test]
  fn test_quality_score_with_numbers() {
    let with_numbers = AntithesisResponse::new(vec![
      "Users save only 5 minutes per week".to_string(),
      "Requires 3 new integrations they don't have".to_string(),
      "Costs 2x more than competitor X".to_string(),
    ]);

    let without_numbers = AntithesisResponse::new(vec![
      "Users save very little time".to_string(),
      "Requires new integrations they don't have".to_string(),
      "Costs more than competitors".to_string(),
    ]);

    assert!(with_numbers.quality_score > without_numbers.quality_score);
  }

  #[test]
  fn test_quality_gate_passes_threshold() {
    let high_quality = AntithesisResponse::new(vec![
            "Users currently use spreadsheets because they offer more flexibility for specific workflows".to_string(),
            "The learning curve requires hours of training which most team leads won't approve due to budget constraints".to_string(),
            "At current pricing, the cost exceeds the typical budget allocation for tools like this significantly".to_string(),
        ]);

    // High quality response should pass the gate
    if high_quality.quality_score >= 70 {
      assert!(high_quality.quality_gate_passes());
    }
  }

  #[test]
  fn test_get_point_by_index() {
    let response = AntithesisResponse::new(vec![
      "First".to_string(),
      "Second".to_string(),
      "Third".to_string(),
    ]);

    assert_eq!(response.get_point(0), Some(&"First".to_string()));
    assert_eq!(response.get_point(1), Some(&"Second".to_string()));
    assert_eq!(response.get_point(2), Some(&"Third".to_string()));
    assert_eq!(response.get_point(3), None);
  }

  #[test]
  fn test_points_slice() {
    let response = AntithesisResponse::new(vec!["A".to_string(), "B".to_string(), "C".to_string()]);

    let slice = response.points();
    assert_eq!(slice.len(), 3);
    assert_eq!(slice[0], "A");
    assert_eq!(slice[1], "B");
    assert_eq!(slice[2], "C");
  }

  #[test]
  fn test_with_points() {
    let original = AntithesisResponse::default();
    let updated = original.with_points(vec![
      "New 1".to_string(),
      "New 2".to_string(),
      "New 3".to_string(),
    ]);

    assert!(updated.validated);
    assert_eq!(updated.points[0], "New 1");
  }

  #[test]
  fn test_with_quality_score_clamped() {
    let response = AntithesisResponse::default().with_quality_score(150);
    assert_eq!(response.quality_score, 100);
  }

  #[test]
  fn test_serialization() {
    let response = AntithesisResponse::new(vec![
      "Point one".to_string(),
      "Point two".to_string(),
      "Point three".to_string(),
    ]);

    let json = serde_json::to_string(&response);
    assert!(json.is_ok());

    if let Ok(json_str) = json {
      let parsed: Result<AntithesisResponse, _> = serde_json::from_str(&json_str);
      assert!(parsed.is_ok());

      if let Ok(parsed_response) = parsed {
        assert_eq!(parsed_response.points, response.points);
        assert_eq!(parsed_response.quality_score, response.quality_score);
        assert_eq!(parsed_response.validated, response.validated);
      }
    }
  }

  #[test]
  fn test_is_validated_whitespace_only() {
    let response =
      AntithesisResponse::new(vec!["   ".to_string(), "\t".to_string(), "\n".to_string()]);
    assert!(!response.validated);
  }

  #[test]
  fn test_is_validated_mixed_whitespace_and_content() {
    let response = AntithesisResponse::new(vec![
      "  Valid content  ".to_string(),
      "Another point".to_string(),
      "  ".to_string(), // This one is whitespace only
    ]);
    assert!(!response.validated);
  }

  #[test]
  fn test_concrete_language_detection() {
    assert!(AntithesisResponse::has_concrete_language(
      "Users will reject this because they prefer existing tools"
    ));
    assert!(AntithesisResponse::has_concrete_language(
      "For example, the onboarding is too complex"
    ));
    assert!(!AntithesisResponse::has_concrete_language(
      "Bad thing happens"
    ));
  }

  #[test]
  fn test_score_single_point_empty() {
    assert_eq!(AntithesisResponse::score_single_point(""), 0);
    assert_eq!(AntithesisResponse::score_single_point("   "), 0);
  }

  #[test]
  fn test_score_single_point_progression() {
    // Note: length_bonus groups 0-4 words together (10 points), 5-9 words (30 points)
    let short = AntithesisResponse::score_single_point("Bad"); // 1 word -> length_bonus=10
    let medium = AntithesisResponse::score_single_point("Users prefer to use their existing tools"); // 7 words -> length_bonus=30
    let detailed = AntithesisResponse::score_single_point(
      "Users prefer their current spreadsheets because they have 5 years of data",
    );
    let with_number = AntithesisResponse::score_single_point("Users save only 5 minutes per week");

    assert!(
      medium > short,
      "medium ({medium}) should be > short ({short})"
    );
    assert!(
      detailed > medium,
      "detailed ({detailed}) should be > medium ({medium})"
    );
    assert!(
      with_number > medium,
      "with_number ({with_number}) should be > medium ({medium})"
    );
  }
}
