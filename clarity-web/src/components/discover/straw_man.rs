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
  #[must_use]
  pub const fn all() -> &'static [Self] {
    &[
      Self::IrrationalActor,
      Self::ManicPixieDreamUser,
      Self::StoicMonk,
      Self::YourClone,
    ]
  }

  /// Short label for display in UI.
  #[must_use]
  pub const fn label(&self) -> &'static str {
    match self {
      Self::IrrationalActor => "Irrational Actor",
      Self::ManicPixieDreamUser => "Manic Pixie Dream User",
      Self::StoicMonk => "Stoic Monk",
      Self::YourClone => "Your Clone",
    }
  }

  /// Detailed description of what this trap means.
  #[must_use]
  pub const fn description(&self) -> &'static str {
    match self {
      Self::IrrationalActor => {
        "User acts against their own motivations or self-interest. \
                 Real users optimize for their own goals, not yours."
      }
      Self::ManicPixieDreamUser => {
        "User magically loves everything without discernment. \
                 Real users have preferences, constraints, and competing priorities."
      }
      Self::StoicMonk => {
        "User tolerates immense friction without complaint. \
                 Real users abandon products at the first sign of difficulty."
      }
      Self::YourClone => {
        "User has your system knowledge and mental models. \
                 Real users don't know what you know about how the system works."
      }
    }
  }

  /// Checkbox label for the trap check UI.
  #[must_use]
  pub const fn checkbox_label(&self) -> &'static str {
    match self {
      Self::IrrationalActor => "acting against own motivations?",
      Self::ManicPixieDreamUser => "magically loves everything?",
      Self::StoicMonk => "tolerating immense friction?",
      Self::YourClone => "has your system knowledge?",
    }
  }
}

/// Result of validating a persona description against straw man traps.
///
/// Tracks which traps were detected and whether the validation passed.
/// A validation passes only when no traps are detected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrawManValidation {
  /// Traps detected in the persona description.
  pub traps_detected: Vec<StrawManTrap>,

  /// Whether the validation passed (no traps detected).
  /// This is true only when `traps_detected` is empty.
  pub passed: bool,
}

impl StrawManValidation {
  /// Create a new validation result.
  #[must_use]
  pub const fn new(traps_detected: Vec<StrawManTrap>) -> Self {
    let passed = traps_detected.is_empty();
    Self {
      traps_detected,
      passed,
    }
  }

  /// Create a passing validation (no traps detected).
  #[must_use]
  pub const fn passing() -> Self {
    Self {
      traps_detected: Vec::new(),
      passed: true,
    }
  }

  /// Check if a specific trap was detected.
  #[must_use]
  pub fn has_trap(&self, trap: StrawManTrap) -> bool {
    self.traps_detected.contains(&trap)
  }

  /// Get the count of detected traps.
  #[must_use]
  pub const fn trap_count(&self) -> usize {
    self.traps_detected.len()
  }

  /// Check if validation is valid (passed field matches `traps_detected.is_empty()`).
  /// This enforces the invariant that passed is true only when `traps_detected` is empty.
  #[must_use]
  pub const fn is_valid(&self) -> bool {
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
  fn test_trap_serialization() -> Result<(), serde_json::Error> {
    let trap = StrawManTrap::ManicPixieDreamUser;
    let json = serde_json::to_string(&trap)?;

    let parsed: StrawManTrap = serde_json::from_str(&json)?;
    assert_eq!(parsed, StrawManTrap::ManicPixieDreamUser);
    Ok(())
  }

  #[test]
  fn test_validation_serialization() -> Result<(), serde_json::Error> {
    let validation =
      StrawManValidation::new(vec![StrawManTrap::IrrationalActor, StrawManTrap::StoicMonk]);

    let json = serde_json::to_string(&validation)?;

    let parsed: StrawManValidation = serde_json::from_str(&json)?;

    assert_eq!(parsed.traps_detected.len(), 2);
    assert!(!parsed.passed);
    Ok(())
  }
}
