#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unreachable_patterns)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{InterviewStage, Perspective, Profile, QuestionCategory, QuestionPriority};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Answer {
  pub question_id: String,
  pub question_text: String,
  pub perspective: Perspective,
  pub round: u32,
  pub response: String,
  pub extracted: HashMap<String, String>,
  pub confidence: f64,
  pub notes: String,
  pub timestamp: String,
}

impl Default for Answer {
  fn default() -> Self {
    Self {
      question_id: String::new(),
      question_text: String::new(),
      perspective: Perspective::default(),
      round: 1,
      response: String::new(),
      extracted: HashMap::new(),
      confidence: 0.0,
      notes: String::new(),
      timestamp: String::new(),
    }
  }
}

/// Gap lifecycle state - explicit state machine replacing Option-as-state.
///
/// A gap transitions from Open -> Resolved when a resolution is provided.
/// This makes illegal states unrepresentable (e.g., "resolved but no resolution text").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
#[derive(Default)]
pub enum GapState {
  /// Gap is open and needs resolution
  #[default]
  Open,
  /// Gap has been resolved with the given resolution text
  Resolved { resolution: String },
}

impl GapState {
  /// Check if the gap is resolved.
  #[must_use]
  pub const fn is_resolved(&self) -> bool {
    matches!(self, Self::Resolved { .. })
  }

  /// Check if the gap is open.
  #[must_use]
  pub const fn is_open(&self) -> bool {
    matches!(self, Self::Open)
  }

  /// Get the resolution text if resolved.
  #[must_use]
  pub fn resolution(&self) -> Option<&str> {
    match self {
      Self::Open => None,
      Self::Resolved { resolution } => Some(resolution),
    }
  }

  /// Check if transition to another state is valid.
  ///
  /// `GapState` is a one-way state machine: Open -> Resolved.
  /// Once resolved, no further transitions are allowed.
  #[must_use]
  pub const fn can_transition_to(&self, next: &Self) -> bool {
    !matches!((self, next), (Self::Resolved { .. }, Self::Open))
  }

  /// Transition to a new state with exhaustive pattern matching.
  ///
  /// # Errors
  /// Returns `GapStateError::AlreadyResolved` if the gap is already resolved.
  /// Returns `GapStateError::EmptyResolution` if the resolution text is empty.
  pub fn transition_to(self, next: Self) -> Result<Self, GapStateError> {
    if matches!((&self, &next), (Self::Resolved { .. }, Self::Open)) {
      return Err(GapStateError::AlreadyResolved);
    }

    if let Self::Resolved { resolution } = &next {
      if resolution.trim().is_empty() {
        return Err(GapStateError::EmptyResolution);
      }
    }

    Ok(next)
  }

  /// Resolve the gap with the given resolution text.
  ///
  /// # Errors
  /// Returns an error if:
  /// - The resolution text is empty or whitespace only
  /// - The gap is already resolved (one-way transition enforced)
  pub fn resolve(&self, resolution: String) -> Result<Self, GapStateError> {
    // P0: Enforce one-way transition
    if self.is_resolved() {
      return Err(GapStateError::AlreadyResolved);
    }
    // P1: Resolution must be non-empty when resolved
    if resolution.trim().is_empty() {
      return Err(GapStateError::EmptyResolution);
    }
    Ok(Self::Resolved { resolution })
  }

  /// Validate the current state for invariants.
  ///
  /// # Errors
  /// Returns an error if the state violates invariants (e.g., empty resolution).
  pub fn validate(&self) -> Result<(), GapStateError> {
    match self {
      Self::Open => Ok(()),
      Self::Resolved { resolution } => {
        if resolution.trim().is_empty() {
          Err(GapStateError::EmptyResolution)
        } else {
          Ok(())
        }
      }
    }
  }
}

/// Errors for gap state transitions.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GapStateError {
  #[error("resolution text cannot be empty")]
  EmptyResolution,
  #[error("gap is already resolved")]
  AlreadyResolved,
}

/// A gap in required information discovered during an interview.
///
/// Uses explicit `GapState` to track lifecycle, making the state machine
/// visible in the type system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gap {
  pub id: String,
  pub field: String,
  pub description: String,
  pub blocking: bool,
  pub suggested_default: String,
  pub why_needed: String,
  pub round: u32,
  /// Lifecycle state - use `state.is_resolved()` instead of `resolved` field
  pub state: GapState,
}

impl Default for Gap {
  fn default() -> Self {
    Self {
      id: String::new(),
      field: String::new(),
      description: String::new(),
      blocking: true,
      suggested_default: String::new(),
      why_needed: String::new(),
      round: 1,
      state: GapState::Open,
    }
  }
}

impl Gap {
  /// Check if this gap is resolved.
  #[must_use]
  pub const fn is_resolved(&self) -> bool {
    self.state.is_resolved()
  }

  /// Get the resolution text if resolved.
  #[must_use]
  pub fn resolution(&self) -> Option<&str> {
    self.state.resolution()
  }
}

/// Conflict lifecycle state - explicit state machine replacing Option<i32>.
///
/// A conflict transitions from Pending -> Resolved when an option is chosen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
#[derive(Default)]
pub enum ConflictState {
  /// Conflict is pending resolution
  #[default]
  Pending,
  /// Conflict has been resolved by choosing an option
  Resolved { chosen_index: i32 },
}

impl ConflictState {
  /// Check if the conflict is resolved.
  #[must_use]
  pub const fn is_resolved(&self) -> bool {
    matches!(self, Self::Resolved { .. })
  }

  /// Check if the conflict is pending.
  #[must_use]
  pub const fn is_pending(&self) -> bool {
    matches!(self, Self::Pending)
  }

  /// Get the chosen option index if resolved.
  #[must_use]
  pub const fn chosen_index(&self) -> Option<i32> {
    match self {
      Self::Pending => None,
      Self::Resolved { chosen_index } => Some(*chosen_index),
    }
  }

  /// Check if transition to another state is valid.
  ///
  /// `ConflictState` is a one-way state machine: Pending -> Resolved.
  /// Once resolved, no further transitions are allowed.
  #[must_use]
  pub const fn can_transition_to(&self, next: &Self) -> bool {
    !matches!((self, next), (Self::Resolved { .. }, Self::Pending))
  }

  /// Transition to a new state with exhaustive pattern matching.
  ///
  /// # Errors
  /// Returns `ConflictStateError::AlreadyResolved` if the conflict is already resolved.
  /// Returns `ConflictStateError::NegativeIndex` if the index is negative.
  /// Returns `ConflictStateError::InvalidIndex` if the index is out of bounds.
  pub fn transition_to(self, next: Self, option_count: usize) -> Result<Self, ConflictStateError> {
    if matches!(
      (&self, &next),
      (Self::Pending, Self::Pending) | (Self::Resolved { .. }, Self::Resolved { .. })
    ) {
      return Ok(self);
    }

    if matches!((&self, &next), (Self::Resolved { .. }, Self::Pending)) {
      return Err(ConflictStateError::AlreadyResolved);
    }

    if let Self::Resolved { chosen_index } = &next {
      if option_count == 0 {
        return Err(ConflictStateError::EmptyOptions);
      }
      if *chosen_index < 0 {
        return Err(ConflictStateError::NegativeIndex(*chosen_index));
      }
      let index = usize::try_from(*chosen_index)
        .map_err(|_| ConflictStateError::NegativeIndex(*chosen_index))?;
      if index >= option_count {
        return Err(ConflictStateError::InvalidIndex {
          index: *chosen_index,
          option_count,
        });
      }
    }

    Ok(next)
  }

  /// Resolve the conflict by choosing an option.
  ///
  /// # Errors
  /// Returns an error if:
  /// - The index is negative (P0: value range validation)
  /// - The index is out of bounds (P0: bounds checking)
  /// - There are no options to choose from (P1: empty options check)
  /// - The conflict is already resolved (P0: one-way transition)
  pub fn resolve(
    &self,
    chosen_index: i32,
    option_count: usize,
  ) -> Result<Self, ConflictStateError> {
    // P0: Enforce one-way transition
    if self.is_resolved() {
      return Err(ConflictStateError::AlreadyResolved);
    }
    // P1: Options must exist when choosing
    if option_count == 0 {
      return Err(ConflictStateError::EmptyOptions);
    }
    // P0: Index must be non-negative
    if chosen_index < 0 {
      return Err(ConflictStateError::NegativeIndex(chosen_index));
    }
    // P0: Index must be within bounds
    let index =
      usize::try_from(chosen_index).map_err(|_| ConflictStateError::NegativeIndex(chosen_index))?;
    if index >= option_count {
      return Err(ConflictStateError::InvalidIndex {
        index: chosen_index,
        option_count,
      });
    }
    Ok(Self::Resolved { chosen_index })
  }

  /// Validate the current state for invariants.
  ///
  /// # Errors
  /// Returns an error if the state violates invariants.
  /// Note: Cannot fully validate `chosen_index` without knowing `option_count`.
  pub const fn validate(&self) -> Result<(), ConflictStateError> {
    match self {
      Self::Pending => Ok(()),
      Self::Resolved { chosen_index } => {
        // Can only validate that index is non-negative
        if *chosen_index < 0 {
          Err(ConflictStateError::NegativeIndex(*chosen_index))
        } else {
          Ok(())
        }
      }
    }
  }

  /// Validate that the chosen index is within bounds for the given options.
  ///
  /// # Errors
  /// Returns an error if the index is out of bounds or options are empty.
  pub fn validate_bounds(&self, option_count: usize) -> Result<(), ConflictStateError> {
    match self {
      Self::Pending => Ok(()),
      Self::Resolved { chosen_index } => {
        if *chosen_index < 0 {
          return Err(ConflictStateError::NegativeIndex(*chosen_index));
        }
        let index = usize::try_from(*chosen_index)
          .map_err(|_| ConflictStateError::NegativeIndex(*chosen_index))?;
        if option_count == 0 {
          return Err(ConflictStateError::EmptyOptions);
        }
        if index >= option_count {
          return Err(ConflictStateError::InvalidIndex {
            index: *chosen_index,
            option_count,
          });
        }
        Ok(())
      }
    }
  }
}

/// Errors for conflict state transitions.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ConflictStateError {
  #[error("option index cannot be negative: {0}")]
  NegativeIndex(i32),
  #[error("conflict is already resolved")]
  AlreadyResolved,
  #[error("invalid option index {index} (has {option_count} options)")]
  InvalidIndex { index: i32, option_count: usize },
  #[error("cannot resolve conflict with no options")]
  EmptyOptions,
}

/// A conflict between different answers discovered during an interview.
///
/// Uses explicit `ConflictState` to track lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conflict {
  pub id: String,
  pub between: (String, String),
  pub description: String,
  pub impact: String,
  pub options: Vec<ConflictResolution>,
  /// Lifecycle state - use `state.chosen_index()` instead of `chosen` field
  pub state: ConflictState,
}

impl Default for Conflict {
  fn default() -> Self {
    Self {
      id: String::new(),
      between: (String::new(), String::new()),
      description: String::new(),
      impact: String::new(),
      options: Vec::new(),
      state: ConflictState::Pending,
    }
  }
}

impl Conflict {
  /// Check if this conflict is resolved.
  #[must_use]
  pub const fn is_resolved(&self) -> bool {
    self.state.is_resolved()
  }

  /// Get the chosen option index if resolved.
  #[must_use]
  pub const fn chosen_index(&self) -> Option<i32> {
    self.state.chosen_index()
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConflictResolution {
  pub option: String,
  pub description: String,
  pub tradeoffs: String,
  pub recommendation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Question {
  pub id: String,
  pub round: u32,
  pub perspective: Perspective,
  pub category: QuestionCategory,
  pub priority: QuestionPriority,
  pub question: String,
  pub context: String,
  pub example: String,
  pub expected_type: String,
  pub extract_into: Vec<String>,
  pub depends_on: Vec<String>,
  pub blocks: Vec<String>,
}

impl Default for Question {
  fn default() -> Self {
    Self {
      id: String::new(),
      round: 1,
      perspective: Perspective::default(),
      category: QuestionCategory::default(),
      priority: QuestionPriority::default(),
      question: String::new(),
      context: String::new(),
      example: String::new(),
      expected_type: String::new(),
      extract_into: Vec::new(),
      depends_on: Vec::new(),
      blocks: Vec::new(),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterviewSession {
  pub id: String,
  pub profile: Profile,
  pub created_at: String,
  pub updated_at: String,
  pub completed_at: Option<String>,
  pub stage: InterviewStage,
  pub rounds_completed: u32,
  pub answers: Vec<Answer>,
  pub gaps: Vec<Gap>,
  pub conflicts: Vec<Conflict>,
  pub raw_notes: String,
  pub current_phase: u32,
  pub completed_phases: Vec<u32>,
}

impl Default for InterviewSession {
  fn default() -> Self {
    Self {
      id: String::new(),
      profile: Profile::default(),
      created_at: String::new(),
      updated_at: String::new(),
      completed_at: None,
      stage: InterviewStage::default(),
      rounds_completed: 0,
      answers: Vec::new(),
      gaps: Vec::new(),
      conflicts: Vec::new(),
      raw_notes: String::new(),
      current_phase: 1,
      completed_phases: Vec::new(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // ============================================
  // GapState Exhaustive Transition Tests
  // ============================================

  #[test]
  fn gap_state_default_is_open() {
    let state = GapState::default();
    assert!(state.is_open());
    assert!(!state.is_resolved());
  }

  #[test]
  fn gap_state_can_transition_to_exhaustive() {
    let open = GapState::Open;
    let resolved = GapState::Resolved {
      resolution: "test".to_string(),
    };

    // Open -> Open (no-op)
    assert!(open.can_transition_to(&GapState::Open));
    // Open -> Resolved (valid)
    assert!(open.can_transition_to(&resolved));

    // Resolved -> Resolved (no-op)
    assert!(resolved.can_transition_to(&resolved));
    // Resolved -> Open (invalid - one-way)
    assert!(!resolved.can_transition_to(&GapState::Open));
  }

  #[test]
  fn gap_state_transition_to_open_to_resolved() {
    let open = GapState::Open;
    let resolved = GapState::Resolved {
      resolution: "fixed".to_string(),
    };

    let result = open.transition_to(resolved);
    assert!(result.is_ok());
    assert_eq!(
      result,
      Ok(GapState::Resolved {
        resolution: "fixed".to_string()
      })
    );
  }

  #[test]
  fn gap_state_transition_to_resolved_to_open_fails() {
    let resolved = GapState::Resolved {
      resolution: "done".to_string(),
    };

    let result = resolved.transition_to(GapState::Open);
    assert!(result.is_err());
    assert_eq!(result, Err(GapStateError::AlreadyResolved));
  }

  #[test]
  fn gap_state_transition_to_empty_resolution_fails() {
    let open = GapState::Open;
    let resolved_empty = GapState::Resolved {
      resolution: String::new(),
    };
    let resolved_whitespace = GapState::Resolved {
      resolution: "   ".to_string(),
    };

    assert_eq!(
      open.clone().transition_to(resolved_empty),
      Err(GapStateError::EmptyResolution)
    );
    assert_eq!(
      open.transition_to(resolved_whitespace),
      Err(GapStateError::EmptyResolution)
    );
  }

  #[test]
  fn gap_state_no_op_transitions() {
    // Open -> Open
    let result = GapState::Open.transition_to(GapState::Open);
    assert_eq!(result, Ok(GapState::Open));

    // Resolved -> Resolved
    let resolved = GapState::Resolved {
      resolution: "test".to_string(),
    };
    let result = resolved.clone().transition_to(resolved.clone());
    assert_eq!(result, Ok(resolved));
  }

  #[test]
  fn gap_state_resolve_success() {
    let open = GapState::Open;
    let result = open.resolve("fixed the issue".to_string());
    assert!(result.is_ok());
    assert_eq!(
      result,
      Ok(GapState::Resolved {
        resolution: "fixed the issue".to_string()
      })
    );
  }

  #[test]
  fn gap_state_resolve_already_resolved_fails() {
    let resolved = GapState::Resolved {
      resolution: "done".to_string(),
    };
    let result = resolved.resolve("try again".to_string());
    assert_eq!(result, Err(GapStateError::AlreadyResolved));
  }

  #[test]
  fn gap_state_resolve_empty_fails() {
    let open = GapState::Open;
    assert_eq!(
      open.resolve(String::new()),
      Err(GapStateError::EmptyResolution)
    );
    assert_eq!(
      open.resolve("   ".to_string()),
      Err(GapStateError::EmptyResolution)
    );
  }

  #[test]
  fn gap_state_validate() {
    // Open is always valid
    assert!(GapState::Open.validate().is_ok());

    // Resolved with content is valid
    let resolved = GapState::Resolved {
      resolution: "test".to_string(),
    };
    assert!(resolved.validate().is_ok());

    // Resolved with empty is invalid
    let resolved_empty = GapState::Resolved {
      resolution: String::new(),
    };
    assert_eq!(
      resolved_empty.validate(),
      Err(GapStateError::EmptyResolution)
    );
  }

  #[test]
  fn gap_state_resolution_method() {
    assert!(GapState::Open.resolution().is_none());
    let resolved = GapState::Resolved {
      resolution: "answer".to_string(),
    };
    assert_eq!(resolved.resolution(), Some("answer"));
  }

  // ============================================
  // ConflictState Exhaustive Transition Tests
  // ============================================

  #[test]
  fn conflict_state_default_is_pending() {
    let state = ConflictState::default();
    assert!(state.is_pending());
    assert!(!state.is_resolved());
  }

  #[test]
  fn conflict_state_can_transition_to_exhaustive() {
    let pending = ConflictState::Pending;
    let resolved = ConflictState::Resolved { chosen_index: 0 };

    // Pending -> Pending (no-op)
    assert!(pending.can_transition_to(&ConflictState::Pending));
    // Pending -> Resolved (valid)
    assert!(pending.can_transition_to(&resolved));

    // Resolved -> Resolved (no-op)
    assert!(resolved.can_transition_to(&resolved));
    // Resolved -> Pending (invalid - one-way)
    assert!(!resolved.can_transition_to(&ConflictState::Pending));
  }

  #[test]
  fn conflict_state_transition_to_pending_to_resolved() {
    let pending = ConflictState::Pending;
    let resolved = ConflictState::Resolved { chosen_index: 1 };

    let result = pending.transition_to(resolved, 3);
    assert!(result.is_ok());
    assert_eq!(result, Ok(ConflictState::Resolved { chosen_index: 1 }));
  }

  #[test]
  fn conflict_state_transition_to_resolved_to_pending_fails() {
    let resolved = ConflictState::Resolved { chosen_index: 0 };

    let result = resolved.transition_to(ConflictState::Pending, 3);
    assert!(result.is_err());
    assert_eq!(result, Err(ConflictStateError::AlreadyResolved));
  }

  #[test]
  fn conflict_state_transition_to_negative_index_fails() {
    let pending = ConflictState::Pending;
    let resolved = ConflictState::Resolved { chosen_index: -1 };

    let result = pending.transition_to(resolved, 3);
    assert!(result.is_err());
    assert_eq!(result, Err(ConflictStateError::NegativeIndex(-1)));
  }

  #[test]
  fn conflict_state_transition_to_out_of_bounds_fails() {
    let pending = ConflictState::Pending;
    let resolved = ConflictState::Resolved { chosen_index: 5 };

    let result = pending.transition_to(resolved, 3);
    assert!(result.is_err());
    assert!(matches!(
      result,
      Err(ConflictStateError::InvalidIndex {
        index: 5,
        option_count: 3
      })
    ));
  }

  #[test]
  fn conflict_state_transition_to_empty_options_fails() {
    let pending = ConflictState::Pending;
    let resolved = ConflictState::Resolved { chosen_index: 0 };

    let result = pending.transition_to(resolved, 0);
    assert!(result.is_err());
    assert_eq!(result, Err(ConflictStateError::EmptyOptions));
  }

  #[test]
  fn conflict_state_no_op_transitions() {
    // Pending -> Pending
    let result = ConflictState::Pending.transition_to(ConflictState::Pending, 0);
    assert_eq!(result, Ok(ConflictState::Pending));

    // Resolved -> Resolved
    let resolved = ConflictState::Resolved { chosen_index: 0 };
    let result = resolved.clone().transition_to(resolved.clone(), 3);
    assert_eq!(result, Ok(resolved));
  }

  #[test]
  fn conflict_state_resolve_success() {
    let pending = ConflictState::Pending;
    let result = pending.resolve(1, 3);
    assert!(result.is_ok());
    assert_eq!(result, Ok(ConflictState::Resolved { chosen_index: 1 }));
  }

  #[test]
  fn conflict_state_resolve_already_resolved_fails() {
    let resolved = ConflictState::Resolved { chosen_index: 0 };
    let result = resolved.resolve(1, 3);
    assert_eq!(result, Err(ConflictStateError::AlreadyResolved));
  }

  #[test]
  fn conflict_state_resolve_negative_index_fails() {
    let pending = ConflictState::Pending;
    let result = pending.resolve(-1, 3);
    assert_eq!(result, Err(ConflictStateError::NegativeIndex(-1)));
  }

  #[test]
  fn conflict_state_resolve_out_of_bounds_fails() {
    let pending = ConflictState::Pending;
    let result = pending.resolve(5, 3);
    assert!(matches!(
      result,
      Err(ConflictStateError::InvalidIndex {
        index: 5,
        option_count: 3
      })
    ));
  }

  #[test]
  fn conflict_state_resolve_empty_options_fails() {
    let pending = ConflictState::Pending;
    let result = pending.resolve(0, 0);
    assert_eq!(result, Err(ConflictStateError::EmptyOptions));
  }

  #[test]
  fn conflict_state_validate() {
    // Pending is always valid
    assert!(ConflictState::Pending.validate().is_ok());

    // Resolved with non-negative index is valid
    let resolved = ConflictState::Resolved { chosen_index: 0 };
    assert!(resolved.validate().is_ok());

    // Resolved with negative index is invalid
    let resolved_negative = ConflictState::Resolved { chosen_index: -1 };
    assert_eq!(
      resolved_negative.validate(),
      Err(ConflictStateError::NegativeIndex(-1))
    );
  }

  #[test]
  fn conflict_state_chosen_index_method() {
    assert!(ConflictState::Pending.chosen_index().is_none());
    let resolved = ConflictState::Resolved { chosen_index: 2 };
    assert_eq!(resolved.chosen_index(), Some(2));
  }
}
