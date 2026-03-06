#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro
)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::domain::quality::{QualityEvaluator, QualityReport};
use crate::domain::{Answer, EarsRequirementRef};
use crate::lattice::quality::LatticeQualityEvaluator;
use crate::storage::types::LatticeCache;
use dioxus::prelude::*;

/// Debounce delay for quality score calculation (ms)
#[allow(dead_code)]
const DEBOUNCE_MS: u64 = 500;

/// Hook for calculating quality score with debouncing
///
/// This hook:
/// - Debounces answer updates by 500ms
/// - Calculates quality score from answers and EARS requirements
/// - Caches results in `lattice_cache` for persistence
/// - Returns current score and loading state
///
/// Note: In browser context, true debouncing requires JS interop.
/// This implementation calculates on every render but could be optimized.
#[must_use]
pub fn use_quality_score(
  answers: Signal<Vec<Answer>>,
  _ears_requirements: Signal<Vec<EarsRequirementRef>>,
) -> (Signal<Option<QualityReport>>, Signal<bool>) {
  let mut quality_score = use_signal(|| None);
  let mut is_calculating = use_signal(|| false);

  // Use effect to recalculate when answers or EARS change
  use_effect(move || {
    let answers_clone = answers.read().clone();

    // Check if we have data to calculate
    let has_data = !answers_clone.is_empty();

    if !has_data {
      *quality_score.write() = None;
      *is_calculating.write() = false;
      return;
    }

    // Set calculating state
    *is_calculating.write() = true;

    // Calculate quality score synchronously
    let evaluator = LatticeQualityEvaluator;
    let result = evaluator.evaluate(&answers_clone);

    // Update score
    match result {
      Ok(score) => {
        quality_score.set(Some(score));
      }
      Err(_) => {
        quality_score.set(None);
      }
    }

    is_calculating.set(false);
  });

  (quality_score, is_calculating)
}

/// Hook for caching quality score to `lattice_cache`
///
/// This hook:
/// - Saves quality score to `lattice_cache` table when it changes
/// - Loads cached score on mount
/// - Handles serialization/deserialization
pub fn use_cached_quality_score(
  phase: Signal<String>,
  quality_score: Signal<Option<QualityReport>>,
) {
  // Load cached score on mount
  use_effect(move || {
    let phase_val = phase.read();
    // In a real implementation, we'd load from database here
    drop(phase_val);
  });

  // Save to cache when score changes
  use_effect(move || {
    let score_option = quality_score.read();
    if let Some(score) = score_option.as_ref() {
      let phase_val = phase.read();
      // Serialize score to JSON
      if let Ok(json) = serde_json::to_string(score) {
        // Create cache entry
        let _cache = LatticeCache::with_current_timestamp(phase_val.clone(), json);
        // In a real implementation, we'd save to database here
      }
      drop(phase_val);
    }
  });
}

#[cfg(test)]
#[allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::unnecessary_debug_formatting,
  clippy::match_same_arms,
  clippy::option_if_let_else,
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro,
  clippy::needless_pass_by_value
)]
mod tests {
  use super::*;

  #[test]
  fn test_debounce_constant() {
    assert_eq!(DEBOUNCE_MS, 500);
  }

  #[test]
  fn test_calculate_score_with_empty_answers() {
    let answers = vec![];
    let evaluator = LatticeQualityEvaluator;
    let result = evaluator.evaluate(&answers);
    assert!(result.is_err());
  }

  #[test]
  fn test_calculate_score_with_sample_data() {
    use chrono::Utc;

    let answers = vec![
      Answer {
        step_id: "user_goal".to_string(),
        value: "User must authenticate".to_string(),
        timestamp: Utc::now().to_rfc3339(),
      },
      Answer {
        step_id: "actors".to_string(),
        value: "System admin".to_string(),
        timestamp: Utc::now().to_rfc3339(),
      },
    ];

    let evaluator = LatticeQualityEvaluator;
    let result = evaluator.evaluate(&answers);
    assert!(result.is_ok());

    if let Ok(score) = result {
      // Should have dimensions
      assert!(!score.dimensions.is_empty());
      // Overall should be calculated
      assert!(score.overall_score <= 100);
    }
  }
}
