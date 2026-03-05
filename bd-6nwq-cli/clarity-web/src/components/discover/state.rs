#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::enum_variant_names)]

use serde::{Deserialize, Serialize};
use std::fmt;

/// Phases in the Progressive Discover flow
///
/// This enum represents the state machine for the progressive discovery process.
/// Each phase represents a distinct step in guiding users from initial idea to locked plan.
///
/// # Phase Transitions
///
/// ```text
/// Prompt -> Extracting -> ConfirmingFields -> Preview -> KirkCompilation -> Locked
/// ```
///
/// Phase transitions are one-way (no skipping phases allowed).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProgressiveDiscoverPhase {
  /// Initial phase: User enters freeform description of their idea
  #[default]
  Prompt,

  /// Loading phase: AI extracts structured fields from user input
  Extracting,

  /// Interactive phase: Field-by-field confirmation with adversarial grilling
  /// Includes sub-phases for each field: Problem, Persona, Solution, Nonpersonas, Scenario
  ConfirmingFields,

  /// Review phase: Plan summary with Four Brutal Truths check
  Preview,

  /// Processing phase: Translate interrogation transcript into KIRK contracts
  KirkCompilation,

  /// Final phase: Collapsed state, KIRK JSON ready for Bead Factory handoff
  Locked,
}

impl ProgressiveDiscoverPhase {
  /// Get a human-readable display name for the phase
  #[must_use]
  pub const fn display_name(self) -> &'static str {
    match self {
      Self::Prompt => "Prompt",
      Self::Extracting => "Extracting",
      Self::ConfirmingFields => "Confirming Fields",
      Self::Preview => "Preview",
      Self::KirkCompilation => "KIRK Compilation",
      Self::Locked => "Locked",
    }
  }

  /// Get a description of the phase for tooltips or help text
  #[must_use]
  pub const fn description(self) -> &'static str {
    match self {
      Self::Prompt => "Describe your idea in freeform text",
      Self::Extracting => "AI is extracting structured fields from your input",
      Self::ConfirmingFields => "Review and confirm each extracted field",
      Self::Preview => "Preview your plan before locking",
      Self::KirkCompilation => "Compiling your plan into KIRK contracts",
      Self::Locked => "Plan is locked and ready for implementation",
    }
  }

  /// Get the 1-based ordinal position of this phase in the flow
  #[must_use]
  pub const fn ordinal(self) -> usize {
    match self {
      Self::Prompt => 1,
      Self::Extracting => 2,
      Self::ConfirmingFields => 3,
      Self::Preview => 4,
      Self::KirkCompilation => 5,
      Self::Locked => 6,
    }
  }

  /// Check if this is the final (locked) phase
  #[must_use]
  pub const fn is_final(self) -> bool {
    matches!(self, Self::Locked)
  }

  /// Check if this is the initial (prompt) phase
  #[must_use]
  pub const fn is_initial(self) -> bool {
    matches!(self, Self::Prompt)
  }

  /// Attempt to transition to the next phase
  ///
  /// Returns `None` if already at the final phase (cannot advance past Locked).
  /// This ensures phase transitions are one-way and sequential.
  #[must_use]
  pub const fn next(self) -> Option<Self> {
    match self {
      Self::Prompt => Some(Self::Extracting),
      Self::Extracting => Some(Self::ConfirmingFields),
      Self::ConfirmingFields => Some(Self::Preview),
      Self::Preview => Some(Self::KirkCompilation),
      Self::KirkCompilation => Some(Self::Locked),
      Self::Locked => None,
    }
  }

  /// Attempt to transition to the previous phase
  ///
  /// Returns `None` if already at the initial phase.
  /// Note: This is primarily for navigation; business logic may restrict backwards transitions.
  #[must_use]
  pub const fn previous(self) -> Option<Self> {
    match self {
      Self::Prompt => None,
      Self::Extracting => Some(Self::Prompt),
      Self::ConfirmingFields => Some(Self::Extracting),
      Self::Preview => Some(Self::ConfirmingFields),
      Self::KirkCompilation => Some(Self::Preview),
      Self::Locked => Some(Self::KirkCompilation),
    }
  }

  /// Get all phases in order
  #[must_use]
  pub const fn all() -> &'static [Self] {
    &[
      Self::Prompt,
      Self::Extracting,
      Self::ConfirmingFields,
      Self::Preview,
      Self::KirkCompilation,
      Self::Locked,
    ]
  }

  /// Get total number of phases
  #[must_use]
  pub const fn count() -> usize {
    6
  }
}

impl fmt::Display for ProgressiveDiscoverPhase {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.display_name())
  }
}

/// Sub-phases within the `ConfirmingFields` phase
///
/// Each sub-phase corresponds to a field that requires confirmation
/// with adversarial coaching:
///
/// 1. **Problem** - Includes antithesis (null hypothesis)
/// 2. **Persona** - Includes straw man trap validation
/// 3. **Solution** - Includes VORP justification
/// 4. **Nonpersona** - Explicit exclusion definition
/// 5. **Scenario** - Three bullet prompts + hole punching
///
/// # Navigation
///
/// - Use [`next`](Self::next) to advance to the next sub-phase
/// - Use [`previous`](Self::previous) to go back to the previous sub-phase
/// - First sub-phase (Problem) has no previous
/// - Last sub-phase (Scenario) has no next
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default, Hash)]
pub enum ConfirmSubPhase {
  /// Confirm the problem statement with antithesis (null hypothesis)
  /// User provides 3 realistic reasons why customers will ignore/reject
  #[default]
  ConfirmProblem,

  /// Confirm target user persona with straw man trap validation
  /// AI validates against: Irrational Actor, Manic Pixie Dream User, Stoic Monk, Your Clone
  ConfirmPersona,

  /// Confirm solution with VORP (Value Over Replacement) justification
  /// Why will they switch? What makes this significantly better?
  ConfirmSolution,

  /// Confirm nonpersonas - who is explicitly NOT being built for
  /// Who will be alienated to keep this focused?
  ConfirmNonpersona,

  /// Confirm north star scenario with 3 bullet prompts + hole punching
  /// Trigger, value moment, feeling + discovery/edge case/motivation checks
  ConfirmScenario,
}

impl ConfirmSubPhase {
  /// Get a human-readable display name for the sub-phase
  #[must_use]
  pub const fn display_name(self) -> &'static str {
    match self {
      Self::ConfirmProblem => "Problem",
      Self::ConfirmPersona => "Persona",
      Self::ConfirmSolution => "Solution",
      Self::ConfirmNonpersona => "Nonpersona",
      Self::ConfirmScenario => "Scenario",
    }
  }

  /// Get a description of what happens in this sub-phase
  #[must_use]
  pub const fn description(self) -> &'static str {
    match self {
      Self::ConfirmProblem => "Confirm the problem and provide antithesis points",
      Self::ConfirmPersona => "Confirm target user and validate against straw man traps",
      Self::ConfirmSolution => "Confirm solution and justify VORP (Value Over Replacement)",
      Self::ConfirmNonpersona => "Define who you are explicitly NOT building for",
      Self::ConfirmScenario => "Define trigger, value moment, and outcome with hole punching",
    }
  }

  /// Get the 1-based ordinal position (1-5) for progress display
  #[must_use]
  pub const fn ordinal(self) -> usize {
    match self {
      Self::ConfirmProblem => 1,
      Self::ConfirmPersona => 2,
      Self::ConfirmSolution => 3,
      Self::ConfirmNonpersona => 4,
      Self::ConfirmScenario => 5,
    }
  }

  /// Check if this is the first sub-phase
  #[must_use]
  pub const fn is_first(self) -> bool {
    matches!(self, Self::ConfirmProblem)
  }

  /// Check if this is the last sub-phase
  #[must_use]
  pub const fn is_last(self) -> bool {
    matches!(self, Self::ConfirmScenario)
  }

  /// Attempt to transition to the next sub-phase
  ///
  /// Returns `None` if already at the last sub-phase (Scenario).
  /// This ensures sequential progression through confirmation steps.
  #[must_use]
  pub const fn next(self) -> Option<Self> {
    match self {
      Self::ConfirmProblem => Some(Self::ConfirmPersona),
      Self::ConfirmPersona => Some(Self::ConfirmSolution),
      Self::ConfirmSolution => Some(Self::ConfirmNonpersona),
      Self::ConfirmNonpersona => Some(Self::ConfirmScenario),
      Self::ConfirmScenario => None,
    }
  }

  /// Attempt to transition to the previous sub-phase
  ///
  /// Returns `None` if already at the first sub-phase (Problem).
  #[must_use]
  pub const fn previous(self) -> Option<Self> {
    match self {
      Self::ConfirmProblem => None,
      Self::ConfirmPersona => Some(Self::ConfirmProblem),
      Self::ConfirmSolution => Some(Self::ConfirmPersona),
      Self::ConfirmNonpersona => Some(Self::ConfirmSolution),
      Self::ConfirmScenario => Some(Self::ConfirmNonpersona),
    }
  }

  /// Get all sub-phases in order
  #[must_use]
  pub const fn all() -> &'static [Self] {
    &[
      Self::ConfirmProblem,
      Self::ConfirmPersona,
      Self::ConfirmSolution,
      Self::ConfirmNonpersona,
      Self::ConfirmScenario,
    ]
  }

  /// Get total number of sub-phases
  #[must_use]
  pub const fn count() -> usize {
    5
  }

  /// Get the field name for this sub-phase (used in form data)
  #[must_use]
  pub const fn field_name(self) -> &'static str {
    match self {
      Self::ConfirmProblem => "problem",
      Self::ConfirmPersona => "persona",
      Self::ConfirmSolution => "solution",
      Self::ConfirmNonpersona => "nonpersona",
      Self::ConfirmScenario => "scenario",
    }
  }
}

impl fmt::Display for ConfirmSubPhase {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.display_name())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_default_phase_is_prompt() {
    assert_eq!(
      ProgressiveDiscoverPhase::default(),
      ProgressiveDiscoverPhase::Prompt
    );
  }

  #[test]
  fn test_display_formats_correctly() {
    assert_eq!(format!("{}", ProgressiveDiscoverPhase::Prompt), "Prompt");
    assert_eq!(
      format!("{}", ProgressiveDiscoverPhase::Extracting),
      "Extracting"
    );
    assert_eq!(
      format!("{}", ProgressiveDiscoverPhase::ConfirmingFields),
      "Confirming Fields"
    );
    assert_eq!(format!("{}", ProgressiveDiscoverPhase::Preview), "Preview");
    assert_eq!(
      format!("{}", ProgressiveDiscoverPhase::KirkCompilation),
      "KIRK Compilation"
    );
    assert_eq!(format!("{}", ProgressiveDiscoverPhase::Locked), "Locked");
  }

  #[test]
  fn test_display_name_matches_variant() {
    for phase in ProgressiveDiscoverPhase::all() {
      let display = phase.display_name();
      assert!(!display.is_empty());
      assert!(display.contains(match phase {
        ProgressiveDiscoverPhase::Prompt => "Prompt",
        ProgressiveDiscoverPhase::Extracting => "Extract",
        ProgressiveDiscoverPhase::ConfirmingFields => "Confirm",
        ProgressiveDiscoverPhase::Preview => "Preview",
        ProgressiveDiscoverPhase::KirkCompilation => "KIRK",
        ProgressiveDiscoverPhase::Locked => "Locked",
      }));
    }
  }

  #[test]
  fn test_description_not_empty() {
    for phase in ProgressiveDiscoverPhase::all() {
      assert!(!phase.description().is_empty());
    }
  }

  #[test]
  fn test_ordinal_values() {
    assert_eq!(ProgressiveDiscoverPhase::Prompt.ordinal(), 1);
    assert_eq!(ProgressiveDiscoverPhase::Extracting.ordinal(), 2);
    assert_eq!(ProgressiveDiscoverPhase::ConfirmingFields.ordinal(), 3);
    assert_eq!(ProgressiveDiscoverPhase::Preview.ordinal(), 4);
    assert_eq!(ProgressiveDiscoverPhase::KirkCompilation.ordinal(), 5);
    assert_eq!(ProgressiveDiscoverPhase::Locked.ordinal(), 6);
  }

  #[test]
  fn test_is_final() {
    assert!(!ProgressiveDiscoverPhase::Prompt.is_final());
    assert!(!ProgressiveDiscoverPhase::Extracting.is_final());
    assert!(!ProgressiveDiscoverPhase::ConfirmingFields.is_final());
    assert!(!ProgressiveDiscoverPhase::Preview.is_final());
    assert!(!ProgressiveDiscoverPhase::KirkCompilation.is_final());
    assert!(ProgressiveDiscoverPhase::Locked.is_final());
  }

  #[test]
  fn test_is_initial() {
    assert!(ProgressiveDiscoverPhase::Prompt.is_initial());
    assert!(!ProgressiveDiscoverPhase::Extracting.is_initial());
    assert!(!ProgressiveDiscoverPhase::ConfirmingFields.is_initial());
    assert!(!ProgressiveDiscoverPhase::Preview.is_initial());
    assert!(!ProgressiveDiscoverPhase::KirkCompilation.is_initial());
    assert!(!ProgressiveDiscoverPhase::Locked.is_initial());
  }

  #[test]
  fn test_next_transitions() {
    assert_eq!(
      ProgressiveDiscoverPhase::Prompt.next(),
      Some(ProgressiveDiscoverPhase::Extracting)
    );
    assert_eq!(
      ProgressiveDiscoverPhase::Extracting.next(),
      Some(ProgressiveDiscoverPhase::ConfirmingFields)
    );
    assert_eq!(
      ProgressiveDiscoverPhase::ConfirmingFields.next(),
      Some(ProgressiveDiscoverPhase::Preview)
    );
    assert_eq!(
      ProgressiveDiscoverPhase::Preview.next(),
      Some(ProgressiveDiscoverPhase::KirkCompilation)
    );
    assert_eq!(
      ProgressiveDiscoverPhase::KirkCompilation.next(),
      Some(ProgressiveDiscoverPhase::Locked)
    );
    assert_eq!(ProgressiveDiscoverPhase::Locked.next(), None);
  }

  #[test]
  fn test_previous_transitions() {
    assert_eq!(ProgressiveDiscoverPhase::Prompt.previous(), None);
    assert_eq!(
      ProgressiveDiscoverPhase::Extracting.previous(),
      Some(ProgressiveDiscoverPhase::Prompt)
    );
    assert_eq!(
      ProgressiveDiscoverPhase::ConfirmingFields.previous(),
      Some(ProgressiveDiscoverPhase::Extracting)
    );
    assert_eq!(
      ProgressiveDiscoverPhase::Preview.previous(),
      Some(ProgressiveDiscoverPhase::ConfirmingFields)
    );
    assert_eq!(
      ProgressiveDiscoverPhase::KirkCompilation.previous(),
      Some(ProgressiveDiscoverPhase::Preview)
    );
    assert_eq!(
      ProgressiveDiscoverPhase::Locked.previous(),
      Some(ProgressiveDiscoverPhase::KirkCompilation)
    );
  }

  #[test]
  fn test_all_phases_count() {
    assert_eq!(
      ProgressiveDiscoverPhase::all().len(),
      ProgressiveDiscoverPhase::count()
    );
    assert_eq!(ProgressiveDiscoverPhase::count(), 6);
  }

  #[test]
  fn test_all_phases_are_unique() {
    let phases = ProgressiveDiscoverPhase::all();
    for (i, phase) in phases.iter().enumerate() {
      for (j, other) in phases.iter().enumerate() {
        if i != j {
          assert_ne!(phase, other, "Phases should be unique");
        }
      }
    }
  }

  #[test]
  fn test_clone() {
    let phase = ProgressiveDiscoverPhase::Preview;
    let cloned = phase;
    assert_eq!(phase, cloned);
  }

  #[test]
  fn test_copy() {
    let phase = ProgressiveDiscoverPhase::KirkCompilation;
    let copied = phase;
    assert_eq!(phase, copied);
  }

  #[test]
  fn test_equality() {
    assert_eq!(
      ProgressiveDiscoverPhase::Prompt,
      ProgressiveDiscoverPhase::Prompt
    );
    assert_ne!(
      ProgressiveDiscoverPhase::Prompt,
      ProgressiveDiscoverPhase::Locked
    );
  }

  #[test]
  fn test_serialize_deserialize_roundtrip() -> Result<(), serde_json::Error> {
    for phase in ProgressiveDiscoverPhase::all() {
      let json = serde_json::to_string(phase)?;
      let deserialized: ProgressiveDiscoverPhase = serde_json::from_str(&json)?;
      assert_eq!(*phase, deserialized);
    }
    Ok(())
  }

  #[test]
  fn test_serialize_values() {
    assert_eq!(
      serde_json::to_string(&ProgressiveDiscoverPhase::Prompt).ok(),
      Some(r#""Prompt""#.to_string())
    );
    assert_eq!(
      serde_json::to_string(&ProgressiveDiscoverPhase::Extracting).ok(),
      Some(r#""Extracting""#.to_string())
    );
    assert_eq!(
      serde_json::to_string(&ProgressiveDiscoverPhase::ConfirmingFields).ok(),
      Some(r#""ConfirmingFields""#.to_string())
    );
    assert_eq!(
      serde_json::to_string(&ProgressiveDiscoverPhase::Preview).ok(),
      Some(r#""Preview""#.to_string())
    );
    assert_eq!(
      serde_json::to_string(&ProgressiveDiscoverPhase::KirkCompilation).ok(),
      Some(r#""KirkCompilation""#.to_string())
    );
    assert_eq!(
      serde_json::to_string(&ProgressiveDiscoverPhase::Locked).ok(),
      Some(r#""Locked""#.to_string())
    );
  }

  #[test]
  fn test_phase_sequence_via_next() {
    // Start at Prompt, walk through all phases
    let mut current = ProgressiveDiscoverPhase::Prompt;
    let mut visited = vec![current];

    while let Some(next) = current.next() {
      visited.push(next);
      current = next;
    }

    assert_eq!(visited.len(), 6, "Should visit all 6 phases");
    assert_eq!(visited.last(), Some(&ProgressiveDiscoverPhase::Locked));
    assert_eq!(
      visited,
      vec![
        ProgressiveDiscoverPhase::Prompt,
        ProgressiveDiscoverPhase::Extracting,
        ProgressiveDiscoverPhase::ConfirmingFields,
        ProgressiveDiscoverPhase::Preview,
        ProgressiveDiscoverPhase::KirkCompilation,
        ProgressiveDiscoverPhase::Locked,
      ]
    );
  }

  #[test]
  fn test_cannot_skip_phases() -> Result<(), &'static str> {
    let mut phase = ProgressiveDiscoverPhase::Prompt;

    phase = phase.next().ok_or("Should transition to Extracting")?;
    assert_eq!(phase, ProgressiveDiscoverPhase::Extracting);

    phase = phase
      .next()
      .ok_or("Should transition to ConfirmingFields")?;
    assert_eq!(phase, ProgressiveDiscoverPhase::ConfirmingFields);

    phase = phase.next().ok_or("Should transition to Preview")?;
    assert_eq!(phase, ProgressiveDiscoverPhase::Preview);

    phase = phase.next().ok_or("Should transition to KirkCompilation")?;
    assert_eq!(phase, ProgressiveDiscoverPhase::KirkCompilation);

    phase = phase.next().ok_or("Should transition to Locked")?;
    assert_eq!(phase, ProgressiveDiscoverPhase::Locked);

    assert_eq!(phase.next(), None);
    Ok(())
  }

  // ===== ConfirmSubPhase Tests =====

  #[test]
  fn test_default_subphase_is_problem() {
    assert_eq!(ConfirmSubPhase::default(), ConfirmSubPhase::ConfirmProblem);
  }

  #[test]
  fn test_subphase_display_formats_correctly() {
    assert_eq!(format!("{}", ConfirmSubPhase::ConfirmProblem), "Problem");
    assert_eq!(format!("{}", ConfirmSubPhase::ConfirmPersona), "Persona");
    assert_eq!(format!("{}", ConfirmSubPhase::ConfirmSolution), "Solution");
    assert_eq!(
      format!("{}", ConfirmSubPhase::ConfirmNonpersona),
      "Nonpersona"
    );
    assert_eq!(format!("{}", ConfirmSubPhase::ConfirmScenario), "Scenario");
  }

  #[test]
  fn test_subphase_display_name_matches_variant() {
    for subphase in ConfirmSubPhase::all() {
      let display = subphase.display_name();
      assert!(!display.is_empty());
    }
  }

  #[test]
  fn test_subphase_description_not_empty() {
    for subphase in ConfirmSubPhase::all() {
      assert!(!subphase.description().is_empty());
    }
  }

  #[test]
  fn test_subphase_ordinal_values() {
    assert_eq!(ConfirmSubPhase::ConfirmProblem.ordinal(), 1);
    assert_eq!(ConfirmSubPhase::ConfirmPersona.ordinal(), 2);
    assert_eq!(ConfirmSubPhase::ConfirmSolution.ordinal(), 3);
    assert_eq!(ConfirmSubPhase::ConfirmNonpersona.ordinal(), 4);
    assert_eq!(ConfirmSubPhase::ConfirmScenario.ordinal(), 5);
  }

  #[test]
  fn test_subphase_is_first() {
    assert!(ConfirmSubPhase::ConfirmProblem.is_first());
    assert!(!ConfirmSubPhase::ConfirmPersona.is_first());
    assert!(!ConfirmSubPhase::ConfirmSolution.is_first());
    assert!(!ConfirmSubPhase::ConfirmNonpersona.is_first());
    assert!(!ConfirmSubPhase::ConfirmScenario.is_first());
  }

  #[test]
  fn test_subphase_is_last() {
    assert!(!ConfirmSubPhase::ConfirmProblem.is_last());
    assert!(!ConfirmSubPhase::ConfirmPersona.is_last());
    assert!(!ConfirmSubPhase::ConfirmSolution.is_last());
    assert!(!ConfirmSubPhase::ConfirmNonpersona.is_last());
    assert!(ConfirmSubPhase::ConfirmScenario.is_last());
  }

  #[test]
  fn test_subphase_next_transitions_correctly() {
    assert_eq!(
      ConfirmSubPhase::ConfirmProblem.next(),
      Some(ConfirmSubPhase::ConfirmPersona)
    );
    assert_eq!(
      ConfirmSubPhase::ConfirmPersona.next(),
      Some(ConfirmSubPhase::ConfirmSolution)
    );
    assert_eq!(
      ConfirmSubPhase::ConfirmSolution.next(),
      Some(ConfirmSubPhase::ConfirmNonpersona)
    );
    assert_eq!(
      ConfirmSubPhase::ConfirmNonpersona.next(),
      Some(ConfirmSubPhase::ConfirmScenario)
    );
    assert_eq!(ConfirmSubPhase::ConfirmScenario.next(), None);
  }

  #[test]
  fn test_subphase_prev_returns_none_at_start() {
    assert_eq!(ConfirmSubPhase::ConfirmProblem.previous(), None);
  }

  #[test]
  fn test_subphase_previous_transitions() {
    assert_eq!(ConfirmSubPhase::ConfirmProblem.previous(), None);
    assert_eq!(
      ConfirmSubPhase::ConfirmPersona.previous(),
      Some(ConfirmSubPhase::ConfirmProblem)
    );
    assert_eq!(
      ConfirmSubPhase::ConfirmSolution.previous(),
      Some(ConfirmSubPhase::ConfirmPersona)
    );
    assert_eq!(
      ConfirmSubPhase::ConfirmNonpersona.previous(),
      Some(ConfirmSubPhase::ConfirmSolution)
    );
    assert_eq!(
      ConfirmSubPhase::ConfirmScenario.previous(),
      Some(ConfirmSubPhase::ConfirmNonpersona)
    );
  }

  #[test]
  fn test_subphase_all_count() {
    assert_eq!(ConfirmSubPhase::all().len(), ConfirmSubPhase::count());
    assert_eq!(ConfirmSubPhase::count(), 5);
  }

  #[test]
  fn test_subphase_all_are_unique() {
    let subphases = ConfirmSubPhase::all();
    for (i, subphase) in subphases.iter().enumerate() {
      for (j, other) in subphases.iter().enumerate() {
        if i != j {
          assert_ne!(subphase, other, "Sub-phases should be unique");
        }
      }
    }
  }

  #[test]
  fn test_subphase_clone() {
    let subphase = ConfirmSubPhase::ConfirmSolution;
    let cloned = subphase;
    assert_eq!(subphase, cloned);
  }

  #[test]
  fn test_subphase_copy() {
    let subphase = ConfirmSubPhase::ConfirmNonpersona;
    let copied = subphase;
    assert_eq!(subphase, copied);
  }

  #[test]
  fn test_subphase_equality() {
    assert_eq!(
      ConfirmSubPhase::ConfirmProblem,
      ConfirmSubPhase::ConfirmProblem
    );
    assert_ne!(
      ConfirmSubPhase::ConfirmProblem,
      ConfirmSubPhase::ConfirmScenario
    );
  }

  #[test]
  fn test_subphase_serialize_deserialize_roundtrip() -> Result<(), serde_json::Error> {
    for subphase in ConfirmSubPhase::all() {
      let json = serde_json::to_string(subphase)?;
      let deserialized: ConfirmSubPhase = serde_json::from_str(&json)?;
      assert_eq!(*subphase, deserialized);
    }
    Ok(())
  }

  #[test]
  fn test_subphase_serialize_values() {
    assert_eq!(
      serde_json::to_string(&ConfirmSubPhase::ConfirmProblem).ok(),
      Some(r#""ConfirmProblem""#.to_string())
    );
    assert_eq!(
      serde_json::to_string(&ConfirmSubPhase::ConfirmPersona).ok(),
      Some(r#""ConfirmPersona""#.to_string())
    );
    assert_eq!(
      serde_json::to_string(&ConfirmSubPhase::ConfirmSolution).ok(),
      Some(r#""ConfirmSolution""#.to_string())
    );
    assert_eq!(
      serde_json::to_string(&ConfirmSubPhase::ConfirmNonpersona).ok(),
      Some(r#""ConfirmNonpersona""#.to_string())
    );
    assert_eq!(
      serde_json::to_string(&ConfirmSubPhase::ConfirmScenario).ok(),
      Some(r#""ConfirmScenario""#.to_string())
    );
  }

  #[test]
  fn test_subphase_field_names() {
    assert_eq!(ConfirmSubPhase::ConfirmProblem.field_name(), "problem");
    assert_eq!(ConfirmSubPhase::ConfirmPersona.field_name(), "persona");
    assert_eq!(ConfirmSubPhase::ConfirmSolution.field_name(), "solution");
    assert_eq!(
      ConfirmSubPhase::ConfirmNonpersona.field_name(),
      "nonpersona"
    );
    assert_eq!(ConfirmSubPhase::ConfirmScenario.field_name(), "scenario");
  }

  #[test]
  fn test_subphase_sequence_via_next() {
    let mut current = ConfirmSubPhase::ConfirmProblem;
    let mut visited = vec![current];

    while let Some(next) = current.next() {
      visited.push(next);
      current = next;
    }

    assert_eq!(visited.len(), 5, "Should visit all 5 sub-phases");
    assert_eq!(visited.last(), Some(&ConfirmSubPhase::ConfirmScenario));
  }

  #[test]
  fn test_subphase_cannot_skip() -> Result<(), &'static str> {
    let mut subphase = ConfirmSubPhase::ConfirmProblem;

    subphase = subphase.next().ok_or("Should transition to Persona")?;
    assert_eq!(subphase, ConfirmSubPhase::ConfirmPersona);

    subphase = subphase.next().ok_or("Should transition to Solution")?;
    assert_eq!(subphase, ConfirmSubPhase::ConfirmSolution);

    subphase = subphase.next().ok_or("Should transition to Nonpersona")?;
    assert_eq!(subphase, ConfirmSubPhase::ConfirmNonpersona);

    subphase = subphase.next().ok_or("Should transition to Scenario")?;
    assert_eq!(subphase, ConfirmSubPhase::ConfirmScenario);

    assert_eq!(subphase.next(), None);
    Ok(())
  }
}
