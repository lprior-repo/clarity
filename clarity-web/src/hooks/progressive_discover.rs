#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Hook for managing Progressive Discover state machine.
//!
//! Provides state management for the progressive discovery flow,
//! handling phase transitions, transcript management, and persistence.

use dioxus::prelude::*;

use crate::components::discover::state::{ConfirmSubPhase, ProgressiveDiscoverPhase};
use crate::storage::transcript_store::InterrogationTranscript;

/// State for the Progressive Discover flow.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProgressiveDiscoverState {
    /// Current phase in the discovery flow
    pub phase: ProgressiveDiscoverPhase,
    /// Current sub-phase when in ConfirmingFields phase
    pub sub_phase: ConfirmSubPhase,
    /// The interrogation transcript containing all extracted data
    pub transcript: InterrogationTranscript,
    /// Whether an async operation is in progress
    pub is_loading: bool,
    /// Error message if something went wrong
    pub error: Option<String>,
}

impl ProgressiveDiscoverState {
    /// Create a new state with the given prompt.
    #[must_use]
    pub fn from_prompt(prompt: String) -> Self {
        Self {
            phase: ProgressiveDiscoverPhase::Prompt,
            sub_phase: ConfirmSubPhase::default(),
            transcript: InterrogationTranscript::from_prompt(prompt),
            is_loading: false,
            error: None,
        }
    }

    /// Transition to the next phase.
    ///
    /// Returns the new state with the phase advanced.
    /// Returns `None` if already at the final phase.
    #[must_use]
    pub fn advance_phase(self) -> Option<Self> {
        let next_phase = self.phase.next()?;
        Some(Self {
            phase: next_phase,
            sub_phase: ConfirmSubPhase::default(),
            transcript: self.transcript,
            is_loading: self.is_loading,
            error: self.error,
        })
    }

    /// Transition to the previous phase.
    ///
    /// Returns the new state with the phase regressed.
    /// Returns `None` if already at the initial phase.
    #[must_use]
    pub fn regress_phase(self) -> Option<Self> {
        let prev_phase = self.phase.previous()?;
        Some(Self {
            phase: prev_phase,
            sub_phase: ConfirmSubPhase::default(),
            transcript: self.transcript,
            is_loading: self.is_loading,
            error: self.error,
        })
    }

    /// Transition to the next sub-phase within ConfirmingFields.
    ///
    /// Returns the new state with the sub-phase advanced.
    /// Returns `None` if at the last sub-phase.
    #[must_use]
    pub fn advance_sub_phase(self) -> Option<Self> {
        let next_sub = self.sub_phase.next()?;
        Some(Self {
            phase: self.phase,
            sub_phase: next_sub,
            transcript: self.transcript,
            is_loading: self.is_loading,
            error: self.error,
        })
    }

    /// Transition to the previous sub-phase within ConfirmingFields.
    ///
    /// Returns the new state with the sub-phase regressed.
    /// Returns `None` if at the first sub-phase.
    #[must_use]
    pub fn regress_sub_phase(self) -> Option<Self> {
        let prev_sub = self.sub_phase.previous()?;
        Some(Self {
            phase: self.phase,
            sub_phase: prev_sub,
            transcript: self.transcript,
            is_loading: self.is_loading,
            error: self.error,
        })
    }

    /// Update the transcript.
    #[must_use]
    pub fn with_transcript(self, transcript: InterrogationTranscript) -> Self {
        Self {
            phase: self.phase,
            sub_phase: self.sub_phase,
            transcript,
            is_loading: self.is_loading,
            error: self.error,
        }
    }

    /// Set the loading state.
    #[must_use]
    pub fn with_loading(self, is_loading: bool) -> Self {
        Self {
            phase: self.phase,
            sub_phase: self.sub_phase,
            transcript: self.transcript,
            is_loading,
            error: self.error,
        }
    }

    /// Set an error message.
    #[must_use]
    pub fn with_error(self, error: Option<String>) -> Self {
        Self {
            phase: self.phase,
            sub_phase: self.sub_phase,
            transcript: self.transcript,
            is_loading: self.is_loading,
            error,
        }
    }

    /// Check if the current phase is ConfirmingFields.
    #[must_use]
    pub const fn is_confirming(&self) -> bool {
        matches!(self.phase, ProgressiveDiscoverPhase::ConfirmingFields)
    }

    /// Check if the state is in a terminal (locked) phase.
    #[must_use]
    pub const fn is_locked(&self) -> bool {
        matches!(self.phase, ProgressiveDiscoverPhase::Locked)
    }
}

/// Hook for managing Progressive Discover state.
///
/// This hook provides:
/// - Phase state management with transitions
/// - Transcript data management
/// - Loading and error states
/// - Persistence integration (future)
///
/// # Example
///
/// ```rust,ignore
/// let state = use_progressive_discover();
///
/// // Read current phase
/// let phase = state.read().phase;
///
/// // Advance to next phase
/// state.write().phase = phase.next().unwrap_or(phase);
/// ```
pub fn use_progressive_discover() -> Signal<ProgressiveDiscoverState> {
    use_signal(ProgressiveDiscoverState::default)
}

/// Hook for managing Progressive Discover state with an initial prompt.
///
/// This is a convenience wrapper that initializes the state with a prompt.
pub fn use_progressive_discover_with_prompt(prompt: String) -> Signal<ProgressiveDiscoverState> {
    use_signal(|| ProgressiveDiscoverState::from_prompt(prompt))
}

/// Actions for manipulating Progressive Discover state.
///
/// This struct provides convenience methods for state transitions.
/// Since Dioxus Signal requires interior mutability via closures,
/// this struct stores the signal and provides methods that can be called
/// from within component closures.
///
/// Note: Methods take `self` by value (Copy) to work around Signal's
/// interior mutability requirements in Dioxus 0.7.
#[derive(Clone, Copy)]
pub struct ProgressiveDiscoverActions {
    /// The state signal
    pub state: Signal<ProgressiveDiscoverState>,
}

impl PartialEq for ProgressiveDiscoverActions {
    fn eq(&self, _other: &Self) -> bool {
        // Signals cannot be meaningfully compared for equality.
        // Two Signal instances wrapping the same underlying signal are
        // indistinguishable via their API, so we return false to avoid
        // incorrect equality assumptions.
        false
    }
}

impl ProgressiveDiscoverActions {
    /// Create new actions for the given state signal.
    #[must_use]
    pub const fn new(state: Signal<ProgressiveDiscoverState>) -> Self {
        Self { state }
    }

    /// Get a reference to the current state.
    #[must_use]
    pub fn current(&self) -> ProgressiveDiscoverState {
        self.state.read().clone()
    }

    /// Get the current phase.
    #[must_use]
    pub fn phase(&self) -> ProgressiveDiscoverPhase {
        self.state.read().phase
    }

    /// Get the current sub-phase.
    #[must_use]
    pub fn sub_phase(&self) -> ConfirmSubPhase {
        self.state.read().sub_phase
    }

    /// Check if currently in the confirming fields phase.
    #[must_use]
    pub fn is_confirming(&self) -> bool {
        self.state.read().is_confirming()
    }

    /// Check if currently in the locked (final) phase.
    #[must_use]
    pub fn is_locked(&self) -> bool {
        self.state.read().is_locked()
    }

    /// Check if loading.
    #[must_use]
    pub fn is_loading(&self) -> bool {
        self.state.read().is_loading
    }

    /// Get the current error, if any.
    #[must_use]
    pub fn error(&self) -> Option<String> {
        self.state.read().error.clone()
    }

    /// Advance to the next phase.
    pub fn advance_phase(&mut self) {
        let current = self.state.read().clone();
        if let Some(next) = current.advance_phase() {
            self.state.set(next);
        }
    }

    /// Regress to the previous phase.
    pub fn regress_phase(&mut self) {
        let current = self.state.read().clone();
        if let Some(prev) = current.regress_phase() {
            self.state.set(prev);
        }
    }

    /// Update the transcript.
    pub fn update_transcript(&mut self, transcript: InterrogationTranscript) {
        let current = self.state.read().clone();
        self.state.set(current.with_transcript(transcript));
    }

    /// Advance to the next sub-phase within ConfirmingFields.
    pub fn advance_sub_phase(&mut self) {
        let current = self.state.read().clone();
        if let Some(next) = current.advance_sub_phase() {
            self.state.set(next);
        }
    }

    /// Regress to the previous sub-phase within ConfirmingFields.
    pub fn regress_sub_phase(&mut self) {
        let current = self.state.read().clone();
        if let Some(prev) = current.regress_sub_phase() {
            self.state.set(prev);
        }
    }
}

/// Hook that returns actions for reading Progressive Discover state.
///
/// For state mutations, use the signal's `.set()` method directly in closures:
///
/// ```rust,ignore
/// let state = use_progressive_discover();
/// let actions = use_progressive_discover_actions(state);
///
/// rsx! {
///     // Reading state
///     p { "Current phase: {actions.phase()}" }
///
///     // Writing state (use closure + .set())
///     button {
///         onclick: move |_| {
///             let current = state.read().clone();
///             if let Some(next) = current.advance_phase() {
///                 state.set(next);
///             }
///         },
///         "Next Phase"
///     }
/// }
/// ```
pub fn use_progressive_discover_actions(
    state: Signal<ProgressiveDiscoverState>,
) -> ProgressiveDiscoverActions {
    ProgressiveDiscoverActions::new(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state_starts_at_prompt() {
        let state = ProgressiveDiscoverState::default();
        assert_eq!(state.phase, ProgressiveDiscoverPhase::Prompt);
        assert_eq!(state.sub_phase, ConfirmSubPhase::ConfirmProblem);
        assert!(!state.is_loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_from_prompt_sets_original_prompt() {
        let state = ProgressiveDiscoverState::from_prompt("Build a todo app".to_string());
        assert_eq!(state.transcript.original_prompt, "Build a todo app");
        assert_eq!(state.phase, ProgressiveDiscoverPhase::Prompt);
    }

    #[test]
    fn test_advance_phase_transitions_correctly() {
        let state = ProgressiveDiscoverState::default();

        let state = state.advance_phase().expect("Should advance to Extracting");
        assert_eq!(state.phase, ProgressiveDiscoverPhase::Extracting);

        let state = state.advance_phase().expect("Should advance to ConfirmingFields");
        assert_eq!(state.phase, ProgressiveDiscoverPhase::ConfirmingFields);

        let state = state.advance_phase().expect("Should advance to Preview");
        assert_eq!(state.phase, ProgressiveDiscoverPhase::Preview);

        let state = state.advance_phase().expect("Should advance to KirkCompilation");
        assert_eq!(state.phase, ProgressiveDiscoverPhase::KirkCompilation);

        let state = state.advance_phase().expect("Should advance to Locked");
        assert_eq!(state.phase, ProgressiveDiscoverPhase::Locked);

        // Cannot advance past Locked
        assert!(state.advance_phase().is_none());
    }

    #[test]
    fn test_regress_phase_transitions_correctly() {
        let state = ProgressiveDiscoverState {
            phase: ProgressiveDiscoverPhase::Locked,
            ..Default::default()
        };

        let state = state.regress_phase().expect("Should regress to KirkCompilation");
        assert_eq!(state.phase, ProgressiveDiscoverPhase::KirkCompilation);

        let state = state.regress_phase().expect("Should regress to Preview");
        assert_eq!(state.phase, ProgressiveDiscoverPhase::Preview);

        // Cannot regress past Prompt
        let prompt_state = ProgressiveDiscoverState::default();
        assert!(prompt_state.regress_phase().is_none());
    }

    #[test]
    fn test_advance_sub_phase() {
        let state = ProgressiveDiscoverState {
            phase: ProgressiveDiscoverPhase::ConfirmingFields,
            sub_phase: ConfirmSubPhase::ConfirmProblem,
            ..Default::default()
        };

        let state = state.advance_sub_phase().expect("Should advance to Persona");
        assert_eq!(state.sub_phase, ConfirmSubPhase::ConfirmPersona);

        let state = state.advance_sub_phase().expect("Should advance to Solution");
        assert_eq!(state.sub_phase, ConfirmSubPhase::ConfirmSolution);

        let state = state.advance_sub_phase().expect("Should advance to Nonpersona");
        assert_eq!(state.sub_phase, ConfirmSubPhase::ConfirmNonpersona);

        let state = state.advance_sub_phase().expect("Should advance to Scenario");
        assert_eq!(state.sub_phase, ConfirmSubPhase::ConfirmScenario);

        // Cannot advance past Scenario
        assert!(state.advance_sub_phase().is_none());
    }

    #[test]
    fn test_regress_sub_phase() {
        let state = ProgressiveDiscoverState {
            phase: ProgressiveDiscoverPhase::ConfirmingFields,
            sub_phase: ConfirmSubPhase::ConfirmScenario,
            ..Default::default()
        };

        let state = state.regress_sub_phase().expect("Should regress to Nonpersona");
        assert_eq!(state.sub_phase, ConfirmSubPhase::ConfirmNonpersona);

        // Cannot regress past Problem
        let problem_state = ProgressiveDiscoverState {
            phase: ProgressiveDiscoverPhase::ConfirmingFields,
            sub_phase: ConfirmSubPhase::ConfirmProblem,
            ..Default::default()
        };
        assert!(problem_state.regress_sub_phase().is_none());
    }

    #[test]
    fn test_with_loading() {
        let state = ProgressiveDiscoverState::default().with_loading(true);
        assert!(state.is_loading);

        let state = state.with_loading(false);
        assert!(!state.is_loading);
    }

    #[test]
    fn test_with_error() {
        let state = ProgressiveDiscoverState::default()
            .with_error(Some("Something went wrong".to_string()));
        assert_eq!(state.error, Some("Something went wrong".to_string()));

        let state = state.with_error(None);
        assert!(state.error.is_none());
    }

    #[test]
    fn test_is_confirming() {
        let state = ProgressiveDiscoverState {
            phase: ProgressiveDiscoverPhase::ConfirmingFields,
            ..Default::default()
        };
        assert!(state.is_confirming());

        let state = ProgressiveDiscoverState {
            phase: ProgressiveDiscoverPhase::Prompt,
            ..Default::default()
        };
        assert!(!state.is_confirming());
    }

    #[test]
    fn test_is_locked() {
        let state = ProgressiveDiscoverState {
            phase: ProgressiveDiscoverPhase::Locked,
            ..Default::default()
        };
        assert!(state.is_locked());

        let state = ProgressiveDiscoverState {
            phase: ProgressiveDiscoverPhase::Preview,
            ..Default::default()
        };
        assert!(!state.is_locked());
    }

    #[test]
    fn test_phase_advancement_resets_sub_phase() {
        let state = ProgressiveDiscoverState {
            phase: ProgressiveDiscoverPhase::ConfirmingFields,
            sub_phase: ConfirmSubPhase::ConfirmScenario,
            ..Default::default()
        };

        let state = state.advance_phase().expect("Should advance to Preview");
        // Sub-phase should reset to default when phase changes
        assert_eq!(state.sub_phase, ConfirmSubPhase::ConfirmProblem);
    }

    #[test]
    fn test_with_transcript_preserves_other_fields() {
        let original = ProgressiveDiscoverState {
            phase: ProgressiveDiscoverPhase::Extracting,
            sub_phase: ConfirmSubPhase::ConfirmSolution,
            is_loading: true,
            error: Some("Error".to_string()),
            ..Default::default()
        };

        let new_transcript =
            InterrogationTranscript::from_prompt("New prompt".to_string());
        let updated = original.with_transcript(new_transcript.clone());

        assert_eq!(updated.phase, ProgressiveDiscoverPhase::Extracting);
        assert_eq!(updated.sub_phase, ConfirmSubPhase::ConfirmSolution);
        assert!(updated.is_loading);
        assert_eq!(updated.error, Some("Error".to_string()));
        assert_eq!(updated.transcript.original_prompt, "New prompt");
    }
}
