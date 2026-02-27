#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Straw Man trap types that indicate unrealistic user persona assumptions.
///
/// These traps help validate that persona descriptions represent realistic users
/// rather than idealized or impossible user behaviors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum StrawManTrap {
  /// User acts against their own self-interest or motivations.
  /// Example: "Users will pay for features they don't need"
  IrrationalActor,

  /// User magically loves every feature without discernment.
  /// Example: "Users will be delighted by all notifications"
  ManicPixieDreamUser,

  /// User tolerates excessive friction without abandonment.
  /// Example: "Users will complete a 20-step onboarding flow"
  StoicMonk,

  /// User possesses the developer's system knowledge.
  /// Example: "Users will know to check the config file"
  YourClone,
}

impl StrawManTrap {
  /// All straw man trap variants.
  pub fn all() -> &'static [StrawManTrap] {
    &[
      StrawManTrap::IrrationalActor,
      StrawManTrap::ManicPixieDreamUser,
      StrawManTrap::StoicMonk,
      StrawManTrap::YourClone,
    ]
  }

  /// Short label for display in UI.
  pub fn label(&self) -> &'static str {
    match self {
      StrawManTrap::IrrationalActor => "Irrational Actor",
      StrawManTrap::ManicPixieDreamUser => "Manic Pixie Dream User",
      StrawManTrap::StoicMonk => "Stoic Monk",
      StrawManTrap::YourClone => "Your Clone",
    }
  }

  /// Detailed description of what this trap means.
  pub fn description(&self) -> &'static str {
    match self {
      StrawManTrap::IrrationalActor => {
        "User acts against their own motivations or self-interest. \
                 Real users optimize for their own goals, not yours."
      }
      StrawManTrap::ManicPixieDreamUser => {
        "User magically loves everything without discernment. \
                 Real users have preferences, constraints, and competing priorities."
      }
      StrawManTrap::StoicMonk => {
        "User tolerates immense friction without complaint. \
                 Real users abandon products at the first sign of difficulty."
      }
      StrawManTrap::YourClone => {
        "User has your system knowledge and mental models. \
                 Real users don't know what you know about how the system works."
      }
    }
  }

  /// Checkbox label for the trap check UI.
  pub fn checkbox_label(&self) -> &'static str {
    match self {
      StrawManTrap::IrrationalActor => "acting against own motivations?",
      StrawManTrap::ManicPixieDreamUser => "magically loves everything?",
      StrawManTrap::StoicMonk => "tolerating immense friction?",
      StrawManTrap::YourClone => "has your system knowledge?",
    }
  }
}

/// Result of validating a persona description against straw man traps.
///
/// Tracks which traps were detected and whether the validation passed.
/// A validation passes only when no traps are detected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrawManValidation {
  /// Traps detected in the persona description.
  pub traps_detected: Vec<StrawManTrap>,

  /// Whether the validation passed (no traps detected).
  /// This is true only when `traps_detected` is empty.
  pub passed: bool,
}

impl StrawManValidation {
  /// Create a new validation result.
  pub fn new(traps_detected: Vec<StrawManTrap>) -> Self {
    let passed = traps_detected.is_empty();
    Self {
      traps_detected,
      passed,
    }
  }

  /// Create a passing validation (no traps detected).
  pub fn passing() -> Self {
    Self {
      traps_detected: vec![],
      passed: true,
    }
  }

  /// Check if a specific trap was detected.
  pub fn has_trap(&self, trap: StrawManTrap) -> bool {
    self.traps_detected.contains(&trap)
  }

  /// Get the count of detected traps.
  pub fn trap_count(&self) -> usize {
    self.traps_detected.len()
  }

  /// Check if validation is valid (passed field matches traps_detected.is_empty()).
  /// This enforces the invariant that passed is true only when traps_detected is empty.
  pub fn is_valid(&self) -> bool {
    self.passed == self.traps_detected.is_empty()
  }
}

impl Default for StrawManValidation {
  fn default() -> Self {
    Self::passing()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_straw_man_trap_all_returns_four_variants() {
    let all = StrawManTrap::all();
    assert_eq!(all.len(), 4);
    assert!(all.contains(&StrawManTrap::IrrationalActor));
    assert!(all.contains(&StrawManTrap::ManicPixieDreamUser));
    assert!(all.contains(&StrawManTrap::StoicMonk));
    assert!(all.contains(&StrawManTrap::YourClone));
  }

  #[test]
  fn test_straw_man_trap_labels() {
    assert_eq!(StrawManTrap::IrrationalActor.label(), "Irrational Actor");
    assert_eq!(
      StrawManTrap::ManicPixieDreamUser.label(),
      "Manic Pixie Dream User"
    );
    assert_eq!(StrawManTrap::StoicMonk.label(), "Stoic Monk");
    assert_eq!(StrawManTrap::YourClone.label(), "Your Clone");
  }

  #[test]
  fn test_straw_man_trap_descriptions_are_helpful() {
    for trap in StrawManTrap::all() {
      let desc = trap.description();
      assert!(
        !desc.is_empty(),
        "Description should not be empty for {trap:?}"
      );
      assert!(
        desc.len() > 20,
        "Description should be detailed for {trap:?}: {desc}"
      );
    }
  }

  #[test]
  fn test_straw_man_trap_checkbox_labels() {
    for trap in StrawManTrap::all() {
      let label = trap.checkbox_label();
      assert!(
        !label.is_empty(),
        "Checkbox label should not be empty for {trap:?}"
      );
      assert!(
        label.ends_with('?'),
        "Checkbox label should be a question for {trap:?}: {label}"
      );
    }
  }

  #[test]
  fn test_default_validation_passes() {
    let validation = StrawManValidation::default();
    assert!(validation.passed);
    assert!(validation.traps_detected.is_empty());
    assert!(validation.is_valid());
  }

  #[test]
  fn test_passing_validation() {
    let validation = StrawManValidation::passing();
    assert!(validation.passed);
    assert!(validation.traps_detected.is_empty());
    assert_eq!(validation.trap_count(), 0);
  }

  #[test]
  fn test_detected_traps_fails_validation() {
    let traps = vec![StrawManTrap::IrrationalActor, StrawManTrap::YourClone];
    let validation = StrawManValidation::new(traps);

    assert!(!validation.passed);
    assert_eq!(validation.traps_detected.len(), 2);
    assert_eq!(validation.trap_count(), 2);
    assert!(validation.is_valid()); // invariant holds
  }

  #[test]
  fn test_has_trap() {
    let traps = vec![StrawManTrap::StoicMonk];
    let validation = StrawManValidation::new(traps);

    assert!(validation.has_trap(StrawManTrap::StoicMonk));
    assert!(!validation.has_trap(StrawManTrap::IrrationalActor));
    assert!(!validation.has_trap(StrawManTrap::ManicPixieDreamUser));
    assert!(!validation.has_trap(StrawManTrap::YourClone));
  }

  #[test]
  fn test_validation_invariant_passed_matches_empty() {
    // Test that passed is always equal to traps_detected.is_empty()
    let passing = StrawManValidation::new(vec![]);
    assert_eq!(passing.passed, passing.traps_detected.is_empty());
    assert!(passing.is_valid());

    let failing = StrawManValidation::new(vec![StrawManTrap::IrrationalActor]);
    assert_eq!(failing.passed, failing.traps_detected.is_empty());
    assert!(failing.is_valid());

    // Multiple traps
    let multi = StrawManValidation::new(vec![
      StrawManTrap::IrrationalActor,
      StrawManTrap::ManicPixieDreamUser,
      StrawManTrap::StoicMonk,
      StrawManTrap::YourClone,
    ]);
    assert_eq!(multi.passed, multi.traps_detected.is_empty());
    assert!(multi.is_valid());
  }

  #[test]
  fn test_trap_serialization() {
    let trap = StrawManTrap::ManicPixieDreamUser;
    let json = serde_json::to_string(&trap);
    assert!(json.is_ok());

    let parsed: Result<StrawManTrap, _> = serde_json::from_str(&json.unwrap());
    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap(), StrawManTrap::ManicPixieDreamUser);
  }

  #[test]
  fn test_validation_serialization() {
    let validation =
      StrawManValidation::new(vec![StrawManTrap::IrrationalActor, StrawManTrap::StoicMonk]);

    let json = serde_json::to_string(&validation);
    assert!(json.is_ok());

    let parsed: Result<StrawManValidation, _> = serde_json::from_str(&json.unwrap());
    assert!(parsed.is_ok());

    let parsed = parsed.unwrap();
    assert_eq!(parsed.traps_detected.len(), 2);
    assert!(!parsed.passed);
  }
}
