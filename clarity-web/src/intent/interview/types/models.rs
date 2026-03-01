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

/// Gap lifecycle state - explicit state machine for tracking missing information.
///
/// A gap represents a missing required field in the interview. This enum
/// provides a type-safe way to track the gap lifecycle with explicit states.
///
/// # State Machine
///
/// ```text
/// Open ──────> Resolved(resolution: String)
///   │                 │
///   └─────────────────┘  (one-way transition)
/// ```
///
/// The transition is one-way: once a gap is resolved, it cannot be reopened.
/// This makes illegal states unrepresentable (e.g., "resolved but no resolution text").
///
/// # Examples
///
/// ## Creating and Resolving a Gap
///
/// ```
/// # use clarity_web::intent::interview::types::GapState;
/// let gap = GapState::Open;
/// assert!(gap.is_open());
/// assert!(!gap.is_resolved());
///
/// // Resolve the gap
/// let resolved = gap.resolve("User provided the missing value".to_string()).unwrap();
/// assert!(resolved.is_resolved());
/// assert_eq!(resolved.resolution(), Some("User provided the missing value"));
/// ```
///
/// ## Using Transition Method
///
/// ```
/// # use clarity_web::intent::interview::types::GapState;
/// let gap = GapState::Open;
///
/// // Transition to resolved state
/// let resolved = gap.transition_to(GapState::Resolved {
///     resolution: "Value provided".to_string(),
/// }).unwrap();
///
/// // Cannot transition back to Open
/// let result = resolved.transition_to(GapState::Open);
/// assert!(result.is_err());
/// ```
///
/// ## Validation
///
/// ```
/// # use clarity_web::intent::interview::types::GapState;
/// // Empty resolution is invalid
/// let result = GapState::Open.resolve("".to_string());
/// assert!(result.is_err());
///
/// let result = GapState::Open.resolve("   ".to_string());
/// assert!(result.is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
#[derive(Default)]
pub enum GapState {
  /// Gap is open and needs resolution.
  ///
  /// The required field has not been provided and the gap is blocking
  /// progress until resolved.
  #[default]
  Open,

  /// Gap has been resolved with the given resolution text.
  ///
  /// The `resolution` field contains the text explaining how the gap
  /// was addressed (e.g., a provided value or explanation).
  Resolved { resolution: String },
}

impl GapState {
  /// Check if the gap is resolved.
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::GapState;
  /// assert!(!GapState::Open.is_resolved());
  /// assert!(GapState::Resolved { resolution: "done".into() }.is_resolved());
  /// ```
  #[must_use]
  pub const fn is_resolved(&self) -> bool {
    matches!(self, Self::Resolved { .. })
  }

  /// Check if the gap is open.
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::GapState;
  /// assert!(GapState::Open.is_open());
  /// assert!(!GapState::Resolved { resolution: "done".into() }.is_open());
  /// ```
  #[must_use]
  pub const fn is_open(&self) -> bool {
    matches!(self, Self::Open)
  }

  /// Get the resolution text if resolved.
  ///
  /// Returns `None` if the gap is still open.
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::GapState;
  /// assert!(GapState::Open.resolution().is_none());
  ///
  /// let resolved = GapState::Resolved { resolution: "answer".into() };
  /// assert_eq!(resolved.resolution(), Some("answer"));
  /// ```
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
  /// Once resolved, no further transitions to Open are allowed.
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::GapState;
  /// let open = GapState::Open;
  /// let resolved = GapState::Resolved { resolution: "done".into() };
  ///
  /// // Open can transition to Resolved
  /// assert!(open.can_transition_to(&resolved));
  ///
  /// // Resolved cannot transition back to Open
  /// assert!(!resolved.can_transition_to(&GapState::Open));
  ///
  /// // No-op transitions are always valid
  /// assert!(open.can_transition_to(&GapState::Open));
  /// assert!(resolved.can_transition_to(&resolved));
  /// ```
  #[must_use]
  pub const fn can_transition_to(&self, next: &Self) -> bool {
    !matches!((self, next), (Self::Resolved { .. }, Self::Open))
  }

  /// Transition to a new state with exhaustive pattern matching.
  ///
  /// This method validates the transition and returns the new state,
  /// or an error if the transition is invalid.
  ///
  /// # Errors
  ///
  /// - Returns [`GapStateError::AlreadyResolved`] if attempting to
  ///   transition from Resolved back to Open.
  /// - Returns [`GapStateError::EmptyResolution`] if the target state
  ///   is Resolved but has empty/whitespace resolution text.
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::{GapState, GapStateError};
  /// let open = GapState::Open;
  ///
  /// // Valid transition
  /// let resolved = open.transition_to(GapState::Resolved {
  ///     resolution: "provided".into()
  /// }).unwrap();
  /// assert!(resolved.is_resolved());
  ///
  /// // Invalid: empty resolution
  /// let result = GapState::Open.transition_to(GapState::Resolved {
  ///     resolution: "".into()
  /// });
  /// assert_eq!(result, Err(GapStateError::EmptyResolution));
  ///
  /// // Invalid: going back to Open
  /// let result = resolved.transition_to(GapState::Open);
  /// assert_eq!(result, Err(GapStateError::AlreadyResolved));
  /// ```
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
  /// This is a convenience method that creates a Resolved state.
  /// Equivalent to `transition_to(GapState::Resolved { resolution })`.
  ///
  /// # Errors
  ///
  /// - Returns [`GapStateError::AlreadyResolved`] if the gap is already resolved.
  /// - Returns [`GapStateError::EmptyResolution`] if the resolution text
  ///   is empty or contains only whitespace.
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::{GapState, GapStateError};
  /// let open = GapState::Open;
  ///
  /// // Valid resolution
  /// let resolved = open.resolve("User provided value".into()).unwrap();
  /// assert!(resolved.is_resolved());
  ///
  /// // Cannot resolve again
  /// let result = resolved.resolve("Another value".into());
  /// assert_eq!(result, Err(GapStateError::AlreadyResolved));
  ///
  /// // Cannot resolve with empty text
  /// let result = GapState::Open.resolve("".into());
  /// assert_eq!(result, Err(GapStateError::EmptyResolution));
  /// ```
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
  /// Ensures that a Resolved state has non-empty resolution text.
  ///
  /// # Errors
  ///
  /// Returns [`GapStateError::EmptyResolution`] if the state is Resolved
  /// but has empty resolution text.
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::{GapState, GapStateError};
  /// // Open is always valid
  /// assert!(GapState::Open.validate().is_ok());
  ///
  /// // Resolved with content is valid
  /// let valid = GapState::Resolved { resolution: "done".into() };
  /// assert!(valid.validate().is_ok());
  ///
  /// // Resolved without content is invalid
  /// let invalid = GapState::Resolved { resolution: "".into() };
  /// assert_eq!(invalid.validate(), Err(GapStateError::EmptyResolution));
  /// ```
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
///
/// These errors are returned by [`GapState::resolve`], [`GapState::transition_to`],
/// and [`GapState::validate`] when operations violate the state machine rules.
///
/// # Examples
///
/// ```
/// # use clarity_web::intent::interview::types::{GapState, GapStateError};
/// // Attempting to resolve with empty text
/// let result = GapState::Open.resolve("".into());
/// match result {
///     Err(GapStateError::EmptyResolution) => println!("Resolution cannot be empty"),
///     _ => {}
/// }
///
/// // Attempting to re-resolve a resolved gap
/// let resolved = GapState::Resolved { resolution: "done".into() };
/// let result = resolved.resolve("another".into());
/// match result {
///     Err(GapStateError::AlreadyResolved) => println!("Cannot re-resolve"),
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GapStateError {
  /// The resolution text is empty or contains only whitespace.
  #[error("resolution text cannot be empty")]
  EmptyResolution,

  /// Attempted to modify a gap that is already resolved.
  ///
  /// GapState is a one-way state machine; once resolved, it cannot
  /// be reopened or re-resolved.
  #[error("gap is already resolved")]
  AlreadyResolved,
}

/// A gap in required information discovered during an interview.
///
/// A gap represents a missing required field that needs to be addressed
/// before the interview can proceed or be considered complete.
///
/// Uses explicit [`GapState`] to track lifecycle, making the state machine
/// visible in the type system.
///
/// # Examples
///
/// ```
/// # use clarity_web::intent::interview::types::{Gap, GapState};
/// let gap = Gap {
///     id: "gap-base_url".into(),
///     field: "base_url".into(),
///     description: "Missing required field: base_url".into(),
///     blocking: true,
///     state: GapState::Open,
///     ..Gap::default()
/// };
///
/// assert!(gap.is_resolved() == false);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gap {
  /// Unique identifier for this gap (e.g., "gap-base_url").
  pub id: String,
  /// The name of the missing field.
  pub field: String,
  /// Human-readable description of what's missing.
  pub description: String,
  /// Whether this gap blocks progress until resolved.
  pub blocking: bool,
  /// Suggested default value, if any.
  pub suggested_default: String,
  /// Explanation of why this field is needed.
  pub why_needed: String,
  /// The interview round when this gap was detected.
  pub round: u32,
  /// Lifecycle state - use `state.is_resolved()` to check status.
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
  ///
  /// Delegates to the underlying [`GapState`].
  #[must_use]
  pub const fn is_resolved(&self) -> bool {
    self.state.is_resolved()
  }

  /// Get the resolution text if resolved.
  ///
  /// Returns `None` if the gap is still open.
  #[must_use]
  pub fn resolution(&self) -> Option<&str> {
    self.state.resolution()
  }
}

/// Conflict lifecycle state - explicit state machine for tracking answer conflicts.
///
/// A conflict represents contradictory or inconsistent answers discovered
/// during the interview. This enum provides type-safe state management.
///
/// # State Machine
///
/// ```text
/// Pending ──────> Resolved(chosen_index: i32)
///    │                    │
///    └────────────────────┘  (one-way transition)
/// ```
///
/// The transition is one-way: once a conflict is resolved, it cannot be reopened.
///
/// # Index Validation
///
/// When resolving a conflict, the chosen index must:
/// - Be non-negative (>= 0)
/// - Be within bounds of the available options
///
/// # Examples
///
/// ## Creating and Resolving a Conflict
///
/// ```
/// # use clarity_web::intent::interview::types::ConflictState;
/// let conflict = ConflictState::Pending;
/// assert!(conflict.is_pending());
///
/// // Resolve by choosing option 1 of 3
/// let resolved = conflict.resolve(1, 3).unwrap();
/// assert!(resolved.is_resolved());
/// assert_eq!(resolved.chosen_index(), Some(1));
/// ```
///
/// ## Using Transition Method
///
/// ```
/// # use clarity_web::intent::interview::types::ConflictState;
/// let pending = ConflictState::Pending;
///
/// // Transition to resolved state (with 3 options available)
/// let resolved = pending.transition_to(ConflictState::Resolved { chosen_index: 0 }, 3).unwrap();
///
/// // Cannot transition back to Pending
/// let result = resolved.transition_to(ConflictState::Pending, 3);
/// assert!(result.is_err());
/// ```
///
/// ## Index Validation
///
/// ```
/// # use clarity_web::intent::interview::types::ConflictState;
/// // Negative index is invalid
/// let result = ConflictState::Pending.resolve(-1, 3);
/// assert!(result.is_err());
///
/// // Out of bounds index is invalid
/// let result = ConflictState::Pending.resolve(5, 3);
/// assert!(result.is_err());
///
/// // Empty options cannot be resolved
/// let result = ConflictState::Pending.resolve(0, 0);
/// assert!(result.is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
#[derive(Default)]
pub enum ConflictState {
  /// Conflict is pending resolution.
  ///
  /// The conflicting answers have been identified but no choice
  /// has been made yet.
  #[default]
  Pending,

  /// Conflict has been resolved by choosing an option.
  ///
  /// The `chosen_index` indicates which of the available options
  /// was selected to resolve the conflict.
  Resolved { chosen_index: i32 },
}

impl ConflictState {
  /// Check if the conflict is resolved.
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::ConflictState;
  /// assert!(!ConflictState::Pending.is_resolved());
  /// assert!(ConflictState::Resolved { chosen_index: 0 }.is_resolved());
  /// ```
  #[must_use]
  pub const fn is_resolved(&self) -> bool {
    matches!(self, Self::Resolved { .. })
  }

  /// Check if the conflict is pending.
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::ConflictState;
  /// assert!(ConflictState::Pending.is_pending());
  /// assert!(!ConflictState::Resolved { chosen_index: 0 }.is_pending());
  /// ```
  #[must_use]
  pub const fn is_pending(&self) -> bool {
    matches!(self, Self::Pending)
  }

  /// Get the chosen option index if resolved.
  ///
  /// Returns `None` if the conflict is still pending.
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::ConflictState;
  /// assert!(ConflictState::Pending.chosen_index().is_none());
  ///
  /// let resolved = ConflictState::Resolved { chosen_index: 2 };
  /// assert_eq!(resolved.chosen_index(), Some(2));
  /// ```
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
  /// Once resolved, no further transitions to Pending are allowed.
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::ConflictState;
  /// let pending = ConflictState::Pending;
  /// let resolved = ConflictState::Resolved { chosen_index: 0 };
  ///
  /// // Pending can transition to Resolved
  /// assert!(pending.can_transition_to(&resolved));
  ///
  /// // Resolved cannot transition back to Pending
  /// assert!(!resolved.can_transition_to(&ConflictState::Pending));
  ///
  /// // No-op transitions are always valid
  /// assert!(pending.can_transition_to(&ConflictState::Pending));
  /// assert!(resolved.can_transition_to(&resolved));
  /// ```
  #[must_use]
  pub const fn can_transition_to(&self, next: &Self) -> bool {
    !matches!((self, next), (Self::Resolved { .. }, Self::Pending))
  }

  /// Transition to a new state with exhaustive pattern matching.
  ///
  /// This method validates the transition and the target state, returning
  /// the new state or an error if invalid.
  ///
  /// # Errors
  ///
  /// - Returns [`ConflictStateError::AlreadyResolved`] if attempting to
  ///   transition from Resolved back to Pending.
  /// - Returns [`ConflictStateError::NegativeIndex`] if the target state
  ///   is Resolved with a negative chosen_index.
  /// - Returns [`ConflictStateError::InvalidIndex`] if the chosen_index
  ///   is out of bounds for the given option_count.
  /// - Returns [`ConflictStateError::EmptyOptions`] if option_count is 0
  ///   and attempting to resolve.
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::{ConflictState, ConflictStateError};
  /// let pending = ConflictState::Pending;
  ///
  /// // Valid transition (choosing option 1 of 3)
  /// let resolved = pending.transition_to(ConflictState::Resolved { chosen_index: 1 }, 3).unwrap();
  /// assert!(resolved.is_resolved());
  ///
  /// // Invalid: negative index
  /// let result = ConflictState::Pending.transition_to(ConflictState::Resolved { chosen_index: -1 }, 3);
  /// assert_eq!(result, Err(ConflictStateError::NegativeIndex(-1)));
  ///
  /// // Invalid: out of bounds
  /// let result = ConflictState::Pending.transition_to(ConflictState::Resolved { chosen_index: 5 }, 3);
  /// assert!(matches!(result, Err(ConflictStateError::InvalidIndex { .. })));
  ///
  /// // Invalid: going back to Pending
  /// let result = resolved.transition_to(ConflictState::Pending, 3);
  /// assert_eq!(result, Err(ConflictStateError::AlreadyResolved));
  /// ```
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
  /// This is a convenience method that creates a Resolved state.
  /// Equivalent to `transition_to(ConflictState::Resolved { chosen_index }, option_count)`.
  ///
  /// # Errors
  ///
  /// - Returns [`ConflictStateError::AlreadyResolved`] if the conflict is already resolved.
  /// - Returns [`ConflictStateError::NegativeIndex`] if chosen_index is negative.
  /// - Returns [`ConflictStateError::InvalidIndex`] if chosen_index is out of bounds.
  /// - Returns [`ConflictStateError::EmptyOptions`] if option_count is 0.
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::{ConflictState, ConflictStateError};
  /// let pending = ConflictState::Pending;
  ///
  /// // Valid resolution (choosing option 1 of 3)
  /// let resolved = pending.resolve(1, 3).unwrap();
  /// assert!(resolved.is_resolved());
  ///
  /// // Cannot resolve again
  /// let result = resolved.resolve(2, 3);
  /// assert_eq!(result, Err(ConflictStateError::AlreadyResolved));
  ///
  /// // Invalid: negative index
  /// let result = ConflictState::Pending.resolve(-1, 3);
  /// assert_eq!(result, Err(ConflictStateError::NegativeIndex(-1)));
  ///
  /// // Invalid: out of bounds
  /// let result = ConflictState::Pending.resolve(5, 3);
  /// assert!(matches!(result, Err(ConflictStateError::InvalidIndex { .. })));
  /// ```
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
  /// This method performs basic validation without knowing the option count.
  /// For full bounds validation, use [`validate_bounds`](Self::validate_bounds).
  ///
  /// # Errors
  ///
  /// Returns [`ConflictStateError::NegativeIndex`] if the state is Resolved
  /// with a negative chosen_index.
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::{ConflictState, ConflictStateError};
  /// // Pending is always valid
  /// assert!(ConflictState::Pending.validate().is_ok());
  ///
  /// // Resolved with non-negative index is valid
  /// let valid = ConflictState::Resolved { chosen_index: 0 };
  /// assert!(valid.validate().is_ok());
  ///
  /// // Resolved with negative index is invalid
  /// let invalid = ConflictState::Resolved { chosen_index: -1 };
  /// assert_eq!(invalid.validate(), Err(ConflictStateError::NegativeIndex(-1)));
  /// ```
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
  /// This method performs full validation including bounds checking.
  ///
  /// # Errors
  ///
  /// - Returns [`ConflictStateError::NegativeIndex`] if the chosen_index is negative.
  /// - Returns [`ConflictStateError::EmptyOptions`] if option_count is 0.
  /// - Returns [`ConflictStateError::InvalidIndex`] if the chosen_index is out of bounds.
  ///
  /// # Examples
  ///
  /// ```
  /// # use clarity_web::intent::interview::types::{ConflictState, ConflictStateError};
  /// let resolved = ConflictState::Resolved { chosen_index: 1 };
  ///
  /// // Valid: index 1 is within bounds for 3 options
  /// assert!(resolved.validate_bounds(3).is_ok());
  ///
  /// // Invalid: index 1 is out of bounds for 1 option
  /// let result = resolved.validate_bounds(1);
  /// assert!(matches!(result, Err(ConflictStateError::InvalidIndex { .. })));
  /// ```
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
///
/// These errors are returned by [`ConflictState::resolve`], [`ConflictState::transition_to`],
/// and [`ConflictState::validate`] when operations violate the state machine rules.
///
/// # Examples
///
/// ```
/// # use clarity_web::intent::interview::types::{ConflictState, ConflictStateError};
/// // Attempting to resolve with negative index
/// let result = ConflictState::Pending.resolve(-1, 3);
/// match result {
///     Err(ConflictStateError::NegativeIndex(idx)) => {
///         println!("Index {} cannot be negative", idx);
///     }
///     _ => {}
/// }
///
/// // Attempting to resolve with out-of-bounds index
/// let result = ConflictState::Pending.resolve(10, 3);
/// match result {
///     Err(ConflictStateError::InvalidIndex { index, option_count }) => {
///         println!("Index {} out of bounds ({} options)", index, option_count);
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ConflictStateError {
  /// The chosen option index is negative.
  ///
  /// Option indices must be >= 0.
  #[error("option index cannot be negative: {0}")]
  NegativeIndex(i32),

  /// Attempted to modify a conflict that is already resolved.
  ///
  /// ConflictState is a one-way state machine; once resolved, it cannot
  /// be reopened or re-resolved.
  #[error("conflict is already resolved")]
  AlreadyResolved,

  /// The chosen option index is out of bounds.
  ///
  /// The index must be less than the number of available options.
  #[error("invalid option index {index} (has {option_count} options)")]
  InvalidIndex { index: i32, option_count: usize },

  /// Cannot resolve a conflict that has no options.
  ///
  /// A conflict must have at least one option to choose from.
  #[error("cannot resolve conflict with no options")]
  EmptyOptions,
}

/// A conflict between different answers discovered during an interview.
///
/// A conflict represents contradictory or inconsistent information that
/// needs to be resolved by choosing one of the provided options.
///
/// Uses explicit [`ConflictState`] to track lifecycle.
///
/// # Examples
///
/// ```
/// # use clarity_web::intent::interview::types::{Conflict, ConflictState, ConflictResolution};
/// let conflict = Conflict {
///     id: "conflict-1".into(),
///     between: ("q1".into(), "q2".into()),
///     description: "Conflicting answers about the API version".into(),
///     options: vec![
///         ConflictResolution { option: "v1".into(), ..Default::default() },
///         ConflictResolution { option: "v2".into(), ..Default::default() },
///     ],
///     state: ConflictState::Pending,
///     ..Conflict::default()
/// };
///
/// assert!(conflict.is_resolved() == false);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conflict {
  /// Unique identifier for this conflict.
  pub id: String,
  /// The question IDs that have conflicting answers.
  pub between: (String, String),
  /// Human-readable description of the conflict.
  pub description: String,
  /// Impact of not resolving this conflict.
  pub impact: String,
  /// Available options for resolving the conflict.
  pub options: Vec<ConflictResolution>,
  /// Lifecycle state - use `state.chosen_index()` to get the chosen option.
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
  ///
  /// Delegates to the underlying [`ConflictState`].
  #[must_use]
  pub const fn is_resolved(&self) -> bool {
    self.state.is_resolved()
  }

  /// Get the chosen option index if resolved.
  ///
  /// Returns `None` if the conflict is still pending.
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

/// An interview session containing all collected information.
///
/// This is the main aggregate for the interview system, containing:
///
/// - Profile-specific configuration
/// - Current stage and phase tracking
/// - Collected answers, gaps, and conflicts
/// - Timestamps for lifecycle events
///
/// # Phase Tracking
///
/// The session tracks progress through phases:
///
/// - `current_phase`: The next phase to be completed (starts at 1)
/// - `completed_phases`: List of phases that have been marked complete
///
/// Use [`complete_phase`](InterviewSession::complete_phase) to mark phases as done.
///
/// # Stage Management
///
/// The session progresses through stages based on rounds completed:
///
/// | Rounds | Stage |
/// |--------|-------|
/// | 0-1 | Discovery |
/// | 2 | Discovery |
/// | 3 | Refinement |
/// | 4 | Validation |
/// | 5+ | Complete |
///
/// Use [`complete_round`](InterviewSession::complete_round) to advance rounds.
///
/// # Examples
///
/// ```
/// # use clarity_web::intent::interview::types::{InterviewSession, Profile, InterviewStage};
/// let session = InterviewSession::new(
///     "session-1".into(),
///     Profile::Api,
///     "2024-01-01T00:00:00Z".into(),
/// );
///
/// assert_eq!(session.current_phase, 1);
/// assert_eq!(session.stage, InterviewStage::Discovery);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterviewSession {
  /// Unique identifier for this session.
  pub id: String,
  /// The profile type determining required fields and questions.
  pub profile: Profile,
  /// ISO 8601 timestamp when the session was created.
  pub created_at: String,
  /// ISO 8601 timestamp when the session was last updated.
  pub updated_at: String,
  /// ISO 8601 timestamp when the session was completed, if applicable.
  pub completed_at: Option<String>,
  /// Current lifecycle stage of the interview.
  pub stage: InterviewStage,
  /// Number of rounds that have been completed.
  pub rounds_completed: u32,
  /// All answers collected during the interview.
  pub answers: Vec<Answer>,
  /// Gaps (missing required fields) detected during the interview.
  pub gaps: Vec<Gap>,
  /// Conflicts between answers detected during the interview.
  pub conflicts: Vec<Conflict>,
  /// Raw notes entered by the interviewer.
  pub raw_notes: String,
  /// The current phase number (1-indexed, next phase to complete).
  pub current_phase: u32,
  /// List of phase numbers that have been completed.
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
