#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(
  clippy::struct_field_names,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro
)]

use serde::{Deserialize, Serialize};
use std::fmt;

/// Types of holes that can be identified in a scenario
///
/// These represent gaps in the user journey that need to be addressed
/// to ensure the scenario is complete and realistic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HoleType {
  /// How did the user discover this feature/solution?
  /// Addresses the gap between user need and awareness of the solution.
  DiscoveryHole,
  /// What happens in edge cases (internet drops, typos, errors)?
  /// Addresses technical and usability edge cases.
  EdgeCaseHole,
  /// Why would users continue through high-friction steps?
  /// Addresses motivation and engagement at critical points.
  MotivationDropOff,
}

impl HoleType {
  /// Get all hole types as a slice
  #[must_use]
  pub const fn all() -> &'static [Self] {
    &[
      Self::DiscoveryHole,
      Self::EdgeCaseHole,
      Self::MotivationDropOff,
    ]
  }

  /// Get a human-readable label for this hole type
  #[must_use]
  pub const fn label(self) -> &'static str {
    match self {
      Self::DiscoveryHole => "Discovery Hole",
      Self::EdgeCaseHole => "Edge Case Hole",
      Self::MotivationDropOff => "Motivation Drop-off",
    }
  }

  /// Get a description of what this hole type checks for
  #[must_use]
  pub const fn description(self) -> &'static str {
    match self {
      Self::DiscoveryHole => "How did they find the feature?",
      Self::EdgeCaseHole => "What if internet drops, mistype, etc?",
      Self::MotivationDropOff => "Why continue at high-friction steps?",
    }
  }
}

impl fmt::Display for HoleType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let label = self.label();
    write!(f, "{label}")
  }
}

/// A single identified hole in the scenario
///
/// Represents a gap in the scenario that needs to be addressed,
/// along with metadata about its severity and description.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hole {
  /// The type of hole identified
  pub hole_type: HoleType,
  /// Human-readable description of the hole
  pub description: String,
  /// Severity level (1-5, where 5 is most severe)
  pub severity: u8,
}

impl Hole {
  /// Create a new hole with the given type and description
  #[must_use]
  pub const fn new(hole_type: HoleType, description: String) -> Self {
    Self {
      hole_type,
      description,
      severity: 3, // Default medium severity
    }
  }

  /// Create a new hole with a specific severity level
  ///
  /// # Panics
  /// This function does not panic. Severity is clamped to 1-5 range.
  #[must_use]
  pub fn with_severity(hole_type: HoleType, description: String, severity: u8) -> Self {
    Self {
      hole_type,
      description,
      severity: severity.clamp(1, 5),
    }
  }

  /// Check if this hole is high severity (4 or 5)
  #[must_use]
  pub const fn is_high_severity(&self) -> bool {
    self.severity >= 4
  }
}

/// Results from hole punching validation for scenario
///
/// Hole punching checks identify gaps in the scenario:
/// - Discovery Hole: How did they find the feature?
/// - Edge Case Hole: What if internet drops, mistype, etc?
/// - Motivation Drop-off: Why continue at high-friction steps?
///
/// Each hole stores an optional explanation of how it was addressed.
/// `None` means the hole has not been addressed yet.
/// Empty strings are treated as `None` (via normalization).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HolePunchingResults {
  /// Explanation of how the discovery hole was addressed
  /// None means not yet addressed
  pub discovery_hole: Option<String>,
  /// Explanation of how the edge case hole was addressed
  /// None means not yet addressed
  pub edge_case_hole: Option<String>,
  /// Explanation of how the motivation drop-off was addressed
  /// None means not yet addressed
  pub motivation_dropoff: Option<String>,
}

impl HolePunchingResults {
  /// Create a new `HolePunchingResults` with all holes unaddressed
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Create an empty `HolePunchingResults` (alias for default)
  #[must_use]
  pub fn empty() -> Self {
    Self::default()
  }

  /// Check if all hole punching checks are complete
  ///
  /// Returns true only when all three holes have been addressed
  /// (i.e., all fields are `Some` with non-empty content).
  #[must_use]
  pub fn is_complete(&self) -> bool {
    self
      .discovery_hole
      .as_ref()
      .is_some_and(|s| !s.trim().is_empty())
      && self
        .edge_case_hole
        .as_ref()
        .is_some_and(|s| !s.trim().is_empty())
      && self
        .motivation_dropoff
        .as_ref()
        .is_some_and(|s| !s.trim().is_empty())
  }

  /// Check if a specific hole type has been addressed
  #[must_use]
  pub fn is_addressed(&self, hole_type: HoleType) -> bool {
    match hole_type {
      HoleType::DiscoveryHole => self
        .discovery_hole
        .as_ref()
        .is_some_and(|s| !s.trim().is_empty()),
      HoleType::EdgeCaseHole => self
        .edge_case_hole
        .as_ref()
        .is_some_and(|s| !s.trim().is_empty()),
      HoleType::MotivationDropOff => self
        .motivation_dropoff
        .as_ref()
        .is_some_and(|s| !s.trim().is_empty()),
    }
  }

  /// Address a hole by setting its explanation
  ///
  /// Empty strings are normalized to `None`.
  #[must_use]
  pub fn address(mut self, hole_type: HoleType, explanation: String) -> Self {
    let normalized = Self::normalize_explanation(explanation);
    match hole_type {
      HoleType::DiscoveryHole => self.discovery_hole = normalized,
      HoleType::EdgeCaseHole => self.edge_case_hole = normalized,
      HoleType::MotivationDropOff => self.motivation_dropoff = normalized,
    }
    self
  }

  /// Get the explanation for a specific hole type
  #[must_use]
  pub fn explanation(&self, hole_type: HoleType) -> Option<&str> {
    match hole_type {
      HoleType::DiscoveryHole => self.discovery_hole.as_deref(),
      HoleType::EdgeCaseHole => self.edge_case_hole.as_deref(),
      HoleType::MotivationDropOff => self.motivation_dropoff.as_deref(),
    }
  }

  /// Get list of unaddressed hole types
  #[must_use]
  pub fn unaddressed_holes(&self) -> Vec<HoleType> {
    HoleType::all()
      .iter()
      .filter(|&&hole_type| !self.is_addressed(hole_type))
      .copied()
      .collect()
  }

  /// Get count of addressed holes (0-3)
  #[must_use]
  pub fn addressed_count(&self) -> usize {
    HoleType::all()
      .iter()
      .filter(|&&hole_type| self.is_addressed(hole_type))
      .count()
  }

  /// Normalize an explanation string, converting empty/whitespace to None
  fn normalize_explanation(s: String) -> Option<String> {
    if s.trim().is_empty() {
      None
    } else {
      Some(s)
    }
  }

  /// Create from raw string inputs, normalizing empty strings to None
  #[must_use]
  pub fn from_strings(discovery: String, edge_case: String, motivation: String) -> Self {
    Self {
      discovery_hole: Self::normalize_explanation(discovery),
      edge_case_hole: Self::normalize_explanation(edge_case),
      motivation_dropoff: Self::normalize_explanation(motivation),
    }
  }
}

/// Scenario field containing the 3 bullet prompts for North Star Scenario
///
/// These prompts paint a complete picture of the user journey:
/// 1. Trigger: What triggers them to look for a solution?
/// 2. Value moment: What's the key moment of value?
/// 3. Feeling: How do they feel after?
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioField {
  /// What triggers them to look for a solution?
  pub trigger: String,
  /// What's the key moment of value?
  pub value_moment: String,
  /// How do they feel after?
  pub feeling: String,
  /// Hole punching validation results
  pub hole_punching: HolePunchingResults,
}

impl ScenarioField {
  /// Create a new scenario field with the given values and empty hole punching results.
  #[must_use]
  pub fn new(trigger: String, value_moment: String, feeling: String) -> Self {
    Self {
      trigger,
      value_moment,
      feeling,
      hole_punching: HolePunchingResults::default(),
    }
  }

  /// Create an empty scenario field with default values.
  #[must_use]
  pub fn empty() -> Self {
    Self::default()
  }

  /// Check if all scenario fields are complete (non-empty) and holes addressed
  #[must_use]
  pub fn is_complete(&self) -> bool {
    self.is_bullets_complete() && self.hole_punching.is_complete()
  }

  /// Check if all 3 bullet fields are non-empty (ignoring whitespace)
  #[must_use]
  pub fn is_bullets_complete(&self) -> bool {
    !self.trigger.trim().is_empty()
      && !self.value_moment.trim().is_empty()
      && !self.feeling.trim().is_empty()
  }

  /// Check if the trigger field is empty (for validation)
  #[must_use]
  pub fn is_trigger_empty(&self) -> bool {
    self.trigger.trim().is_empty()
  }

  /// Check if the `value_moment` field is empty (for validation)
  #[must_use]
  pub fn is_value_moment_empty(&self) -> bool {
    self.value_moment.trim().is_empty()
  }

  /// Check if the feeling field is empty (for validation)
  #[must_use]
  pub fn is_feeling_empty(&self) -> bool {
    self.feeling.trim().is_empty()
  }
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

  // ========== HoleType Tests ==========

  #[test]
  fn test_hole_type_all_returns_three_types() {
    let all = HoleType::all();
    assert_eq!(all.len(), 3);
    assert!(all.contains(&HoleType::DiscoveryHole));
    assert!(all.contains(&HoleType::EdgeCaseHole));
    assert!(all.contains(&HoleType::MotivationDropOff));
  }

  #[test]
  fn test_hole_type_labels() {
    assert_eq!(HoleType::DiscoveryHole.label(), "Discovery Hole");
    assert_eq!(HoleType::EdgeCaseHole.label(), "Edge Case Hole");
    assert_eq!(HoleType::MotivationDropOff.label(), "Motivation Drop-off");
  }

  #[test]
  fn test_hole_type_descriptions() {
    assert!(!HoleType::DiscoveryHole.description().is_empty());
    assert!(!HoleType::EdgeCaseHole.description().is_empty());
    assert!(!HoleType::MotivationDropOff.description().is_empty());
  }

  #[test]
  fn test_hole_type_display() {
    assert_eq!(format!("{}", HoleType::DiscoveryHole), "Discovery Hole");
  }

  #[test]
  fn test_hole_type_serialization() {
    let json = serde_json::to_string(&HoleType::DiscoveryHole).ok();
    assert_eq!(json, Some(r#""DiscoveryHole""#.to_string()));
  }

  // ========== Hole Tests ==========

  #[test]
  fn test_hole_new_has_default_severity() {
    let hole = Hole::new(HoleType::DiscoveryHole, "Test description".to_string());
    assert_eq!(hole.severity, 3);
    assert_eq!(hole.hole_type, HoleType::DiscoveryHole);
    assert_eq!(hole.description, "Test description");
  }

  #[test]
  fn test_hole_with_severity_clamps_high() {
    let hole = Hole::with_severity(HoleType::EdgeCaseHole, "Test".to_string(), 10);
    assert_eq!(hole.severity, 5);
  }

  #[test]
  fn test_hole_with_severity_clamps_low() {
    let hole = Hole::with_severity(HoleType::MotivationDropOff, "Test".to_string(), 0);
    assert_eq!(hole.severity, 1);
  }

  #[test]
  fn test_hole_is_high_severity() {
    let high = Hole::with_severity(HoleType::DiscoveryHole, "Test".to_string(), 4);
    let low = Hole::with_severity(HoleType::DiscoveryHole, "Test".to_string(), 3);
    assert!(high.is_high_severity());
    assert!(!low.is_high_severity());
  }

  // ========== HolePunchingResults Tests ==========

  #[test]
  fn test_default_holes_are_none() {
    let holes = HolePunchingResults::default();
    assert!(holes.discovery_hole.is_none());
    assert!(holes.edge_case_hole.is_none());
    assert!(holes.motivation_dropoff.is_none());
    assert!(!holes.is_complete());
  }

  #[test]
  fn test_is_complete_requires_all_holes() {
    let holes = HolePunchingResults {
      discovery_hole: Some("Found via search".to_string()),
      edge_case_hole: Some("Handles offline".to_string()),
      motivation_dropoff: Some("Clear progress indicator".to_string()),
    };
    assert!(holes.is_complete());
  }

  #[test]
  fn test_partial_holes_incomplete() {
    // Only one hole addressed
    let holes = HolePunchingResults {
      discovery_hole: Some("Found via search".to_string()),
      edge_case_hole: None,
      motivation_dropoff: None,
    };
    assert!(!holes.is_complete());
    assert_eq!(holes.addressed_count(), 1);

    // Two holes addressed
    let holes = HolePunchingResults {
      discovery_hole: Some("Found via search".to_string()),
      edge_case_hole: Some("Handles offline".to_string()),
      motivation_dropoff: None,
    };
    assert!(!holes.is_complete());
    assert_eq!(holes.addressed_count(), 2);
  }

  #[test]
  fn test_empty_string_treated_as_none() {
    let holes =
      HolePunchingResults::from_strings(String::new(), "   ".to_string(), "\t\n".to_string());
    assert!(holes.discovery_hole.is_none());
    assert!(holes.edge_case_hole.is_none());
    assert!(holes.motivation_dropoff.is_none());
    assert!(!holes.is_complete());
  }

  #[test]
  fn test_address_method() {
    let holes = HolePunchingResults::new()
      .address(HoleType::DiscoveryHole, "Found via search".to_string())
      .address(
        HoleType::EdgeCaseHole,
        "Handles offline gracefully".to_string(),
      )
      .address(
        HoleType::MotivationDropOff,
        "Progress indicator".to_string(),
      );
    assert!(holes.is_complete());
    assert_eq!(holes.addressed_count(), 3);
  }

  #[test]
  fn test_address_method_empty_normalizes() {
    let holes = HolePunchingResults::new().address(HoleType::DiscoveryHole, String::new());
    assert!(holes.discovery_hole.is_none());
  }

  #[test]
  fn test_is_addressed() {
    let holes = HolePunchingResults {
      discovery_hole: Some("test".to_string()),
      edge_case_hole: None,
      motivation_dropoff: None,
    };
    assert!(holes.is_addressed(HoleType::DiscoveryHole));
    assert!(!holes.is_addressed(HoleType::EdgeCaseHole));
    assert!(!holes.is_addressed(HoleType::MotivationDropOff));
  }

  #[test]
  fn test_explanation() {
    let holes = HolePunchingResults {
      discovery_hole: Some("Found via search".to_string()),
      edge_case_hole: None,
      motivation_dropoff: None,
    };
    assert_eq!(
      holes.explanation(HoleType::DiscoveryHole),
      Some("Found via search")
    );
    assert_eq!(holes.explanation(HoleType::EdgeCaseHole), None);
  }

  #[test]
  fn test_unaddressed_holes() {
    let holes = HolePunchingResults {
      discovery_hole: Some("test".to_string()),
      edge_case_hole: None,
      motivation_dropoff: None,
    };
    let unaddressed = holes.unaddressed_holes();
    assert_eq!(unaddressed.len(), 2);
    assert!(unaddressed.contains(&HoleType::EdgeCaseHole));
    assert!(unaddressed.contains(&HoleType::MotivationDropOff));
  }

  #[test]
  fn test_hole_punching_serialization() {
    let holes = HolePunchingResults {
      discovery_hole: Some("Found via search".to_string()),
      edge_case_hole: Some("Handles offline".to_string()),
      motivation_dropoff: None,
    };

    let json = serde_json::to_string(&holes);
    assert!(json.is_ok());

    let deserialized: HolePunchingResults =
      serde_json::from_str(&json.unwrap_or_default()).unwrap_or_default();
    assert_eq!(
      deserialized.discovery_hole.as_deref(),
      Some("Found via search")
    );
    assert_eq!(
      deserialized.edge_case_hole.as_deref(),
      Some("Handles offline")
    );
    assert!(deserialized.motivation_dropoff.is_none());
  }

  // ========== ScenarioField Tests ==========

  #[test]
  fn test_scenario_field_default_is_incomplete() {
    let scenario = ScenarioField::default();
    assert!(!scenario.is_complete());
    assert!(!scenario.is_bullets_complete());
  }

  #[test]
  fn test_scenario_field_bullets_complete_but_holes_not() {
    let scenario = ScenarioField {
      trigger: "User gets error message".to_string(),
      value_moment: "Problem resolved instantly".to_string(),
      feeling: "Relieved and confident".to_string(),
      hole_punching: HolePunchingResults::default(),
    };
    assert!(scenario.is_bullets_complete());
    assert!(!scenario.is_complete());
  }

  #[test]
  fn test_scenario_field_complete_when_all_filled_and_holes_addressed() {
    let scenario = ScenarioField {
      trigger: "User gets error message".to_string(),
      value_moment: "Problem resolved instantly".to_string(),
      feeling: "Relieved and confident".to_string(),
      hole_punching: HolePunchingResults {
        discovery_hole: Some("Search engine".to_string()),
        edge_case_hole: Some("Offline mode".to_string()),
        motivation_dropoff: Some("Progress bar".to_string()),
      },
    };
    assert!(scenario.is_bullets_complete());
    assert!(scenario.is_complete());
  }

  #[test]
  fn test_whitespace_treated_as_empty() {
    let scenario = ScenarioField {
      trigger: "   ".to_string(),
      value_moment: "\t\n".to_string(),
      feeling: String::new(),
      hole_punching: HolePunchingResults::default(),
    };
    assert!(!scenario.is_bullets_complete());
    assert!(scenario.is_trigger_empty());
    assert!(scenario.is_value_moment_empty());
    assert!(scenario.is_feeling_empty());
  }

  #[test]
  fn test_individual_field_empty_checks() {
    let scenario = ScenarioField {
      trigger: "valid".to_string(),
      value_moment: String::new(),
      feeling: "valid".to_string(),
      hole_punching: HolePunchingResults::default(),
    };
    assert!(!scenario.is_trigger_empty());
    assert!(scenario.is_value_moment_empty());
    assert!(!scenario.is_feeling_empty());
  }

  #[test]
  fn test_scenario_field_serialization() {
    let scenario = ScenarioField {
      trigger: "Error occurs".to_string(),
      value_moment: "Fixed quickly".to_string(),
      feeling: "Happy".to_string(),
      hole_punching: HolePunchingResults {
        discovery_hole: Some("Found via search".to_string()),
        edge_case_hole: None,
        motivation_dropoff: Some("Progress indicator".to_string()),
      },
    };

    let json = serde_json::to_string(&scenario);
    assert!(json.is_ok());

    let deserialized: ScenarioField =
      serde_json::from_str(&json.unwrap_or_default()).unwrap_or_default();

    assert_eq!(deserialized.trigger.as_str(), "Error occurs");
    assert_eq!(deserialized.value_moment.as_str(), "Fixed quickly");
    assert_eq!(deserialized.feeling.as_str(), "Happy");
  }
}
