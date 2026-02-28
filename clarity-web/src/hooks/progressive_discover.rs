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
//!
//! # Crash Recovery (T011)
//!
//! The module automatically stores session state to localStorage, allowing
//! users to resume their session after a browser crash or accidental refresh.
//!
//! # Auto-Save (T010)
//!
//! State changes trigger debounced saves to localStorage to prevent
//! excessive writes while ensuring data is not lost.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::components::discover::state::{ConfirmSubPhase, ProgressiveDiscoverPhase};
use crate::components::discover::straw_man::StrawManTrap;
use crate::components::discover::types::HolePunchingResults;
use crate::kirk::progressive_discover::VorpValidation;
use crate::storage::transcript_store::InterrogationTranscript;

// ============================================================================
// Storage Keys and Constants
// ============================================================================

/// `LocalStorage` key for the current session state
const SESSION_STORAGE_KEY: &str = "clarity_discover_session";

/// `LocalStorage` key for the session ID (for crash recovery detection)
const SESSION_ID_KEY: &str = "clarity_discover_session_id";

/// Debounce duration for auto-save (500ms as per T010 spec)
const AUTO_SAVE_DEBOUNCE_MS: u64 = 500;

// ============================================================================
// Persistable State (T010/T011)
// ============================================================================

/// Persistable state that can be saved to and loaded from localStorage.
///
/// This struct contains all the state needed to restore a Progressive Discover
/// session after a crash or page refresh.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PersistableState {
  /// Current phase in the discovery flow
  pub phase: ProgressiveDiscoverPhase,
  /// Current sub-phase when in `ConfirmingFields` phase
  pub sub_phase: ConfirmSubPhase,
  /// The interrogation transcript containing all extracted data
  pub transcript: InterrogationTranscript,
  /// Unique session identifier
  pub session_id: String,
  /// Timestamp when the session was last saved
  pub saved_at: String,
  /// Antithesis (null hypothesis) points
  pub antithesis_points: [String; 3],
  /// Quality score for antithesis points
  pub antithesis_score: Option<f64>,
  /// Detected straw man traps
  pub detected_traps: Vec<StrawManTrap>,
  /// Acknowledged straw man traps
  pub acknowledged_traps: Vec<StrawManTrap>,
  /// VORP validation result
  pub vorp_validation: Option<VorpValidation>,
  /// Hole punching results
  pub hole_punching: HolePunchingResults,
  /// Four Brutal Truths acknowledgment state
  pub brutal_truths: [bool; 4],
}

impl PersistableState {
  /// Create a new persistable state from the current state.
  #[must_use]
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    phase: ProgressiveDiscoverPhase,
    sub_phase: ConfirmSubPhase,
    transcript: InterrogationTranscript,
    session_id: String,
    antithesis_points: [String; 3],
    antithesis_score: Option<f64>,
    detected_traps: Vec<StrawManTrap>,
    acknowledged_traps: Vec<StrawManTrap>,
    vorp_validation: Option<VorpValidation>,
    hole_punching: HolePunchingResults,
    brutal_truths: [bool; 4],
  ) -> Self {
    Self {
      phase,
      sub_phase,
      transcript,
      session_id,
      saved_at: chrono::Utc::now().to_rfc3339(),
      antithesis_points,
      antithesis_score,
      detected_traps,
      acknowledged_traps,
      vorp_validation,
      hole_punching,
      brutal_truths,
    }
  }

  /// Convert from `ProgressiveDiscoverState`.
  #[must_use]
  pub fn from_state(state: &ProgressiveDiscoverState, session_id: &str) -> Self {
    Self::new(
      state.phase,
      state.sub_phase,
      state.transcript.clone(),
      session_id.to_string(),
      state.antithesis_points.clone(),
      state.antithesis_score,
      state.detected_traps.clone(),
      state.acknowledged_traps.clone(),
      state.vorp_validation.clone(),
      state.hole_punching.clone(),
      state.brutal_truths,
    )
  }
}

// ============================================================================
// LocalStorage Operations (T011)
// ============================================================================

/// Save state to localStorage.
///
/// This function serializes the state to JSON and stores it in localStorage.
/// It is designed to be safe to call from web environments.
///
/// # Errors
///
/// Returns a string describing the error if serialization or storage fails.
pub fn save_to_local_storage(state: &PersistableState) -> Result<(), String> {
  let json = serde_json::to_string(state).map_err(|e| format!("Serialization failed: {e}"))?;

  // Use web-sys for localStorage access
  #[cfg(target_arch = "wasm32")]
  {
    use web_sys::window;
    let window = window().ok_or_else(|| "No window object available".to_string())?;
    let storage = window
      .local_storage()
      .map_err(|e| format!("Failed to get localStorage: {e:?}"))?
      .ok_or_else(|| "localStorage not available".to_string())?;

    storage
      .set_item(SESSION_STORAGE_KEY, &json)
      .map_err(|e| format!("Failed to save to localStorage: {e:?}"))?;

    // Also save session ID separately for quick crash detection
    storage
      .set_item(SESSION_ID_KEY, &state.session_id)
      .map_err(|e| format!("Failed to save session ID: {e:?}"))?;
  }

  // For non-wasm targets (SSR, desktop), log the save
  #[cfg(not(target_arch = "wasm32"))]
  {
    tracing::debug!(
        session_id = %state.session_id,
        phase = ?state.phase,
        "Auto-save: state would be saved to localStorage (non-wasm target)"
    );
    let _ = &json; // Suppress unused variable warning
  }

  Ok(())
}

/// Load state from localStorage.
///
/// This function attempts to restore a previously saved session from localStorage.
/// Returns `None` if no session exists or if deserialization fails.
#[must_use]
#[allow(clippy::missing_const_for_fn)]
pub fn load_from_local_storage() -> Option<PersistableState> {
  #[cfg(target_arch = "wasm32")]
  {
    use web_sys::window;

    let window = window()?;
    let storage = window.local_storage().ok().flatten()?;
    let json = storage.get_item(SESSION_STORAGE_KEY).ok().flatten()?;

    serde_json::from_str(&json)
      .map_err(|e| {
        tracing::warn!("Failed to deserialize saved state: {e}");
        e
      })
      .ok()
  }

  #[cfg(not(target_arch = "wasm32"))]
  {
    None
  }
}

/// Clear saved session from localStorage.
///
/// This should be called when a session is explicitly ended or when
/// the user chooses to start fresh instead of recovering.
#[allow(clippy::missing_const_for_fn)]
pub fn clear_local_storage() {
  #[cfg(target_arch = "wasm32")]
  {
    use web_sys::window;

    if let Some(window) = window() {
      if let Ok(Some(storage)) = window.local_storage() {
        let _ = storage.remove_item(SESSION_STORAGE_KEY);
        let _ = storage.remove_item(SESSION_ID_KEY);
      }
    }
  }
}

/// Check if a recoverable session exists in localStorage.
#[must_use]
#[allow(clippy::missing_const_for_fn)]
pub fn has_recoverable_session() -> bool {
  #[cfg(target_arch = "wasm32")]
  {
    use web_sys::window;

    window()
      .and_then(|w| w.local_storage().ok().flatten())
      .and_then(|s: web_sys::Storage| s.get_item(SESSION_ID_KEY).ok().flatten())
      .is_some()
  }

  #[cfg(not(target_arch = "wasm32"))]
  {
    false
  }
}

/// Get the session ID of a recoverable session without loading the full state.
#[must_use]
#[allow(clippy::missing_const_for_fn)]
pub fn get_recoverable_session_id() -> Option<String> {
  #[cfg(target_arch = "wasm32")]
  {
    use web_sys::window;

    window()
      .and_then(|w| w.local_storage().ok().flatten())
      .and_then(|s: web_sys::Storage| s.get_item(SESSION_ID_KEY).ok().flatten())
  }

  #[cfg(not(target_arch = "wasm32"))]
  {
    None
  }
}

// ============================================================================
// Debounced Auto-Save (T010)
// ============================================================================

/// Debouncer for auto-save operations.
///
/// This struct tracks the last save time and determines whether
/// enough time has passed to warrant another save.
#[derive(Clone, Debug, Default)]
pub struct AutoSaveDebouncer {
  /// Time of the last save operation
  last_save: Rc<RefCell<Option<Instant>>>,
  /// Pending save flag
  pending: Rc<RefCell<bool>>,
}

impl AutoSaveDebouncer {
  /// Create a new debouncer.
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Check if enough time has passed since the last save.
  ///
  /// Returns `true` if a save should be performed immediately,
  /// or `false` if the save should be debounced.
  #[must_use]
  pub fn should_save_now(&self) -> bool {
    let last_save = self.last_save.borrow();
    last_save.is_none_or(|last| last.elapsed() >= Duration::from_millis(AUTO_SAVE_DEBOUNCE_MS))
  }

  /// Mark that a save is pending (will be executed after debounce period).
  pub fn mark_pending(&self) {
    *self.pending.borrow_mut() = true;
  }

  /// Clear the pending flag.
  pub fn clear_pending(&self) {
    *self.pending.borrow_mut() = false;
  }

  /// Check if there is a pending save.
  #[must_use]
  pub fn is_pending(&self) -> bool {
    *self.pending.borrow()
  }

  /// Record that a save has been performed.
  pub fn record_save(&self) {
    *self.last_save.borrow_mut() = Some(Instant::now());
    *self.pending.borrow_mut() = false;
  }

  /// Get the debounce duration.
  #[must_use]
  pub const fn debounce_duration() -> Duration {
    Duration::from_millis(AUTO_SAVE_DEBOUNCE_MS)
  }
}

// ============================================================================
// Session ID Generation
// ============================================================================

/// Generate a unique session ID.
///
/// Uses UUID v4 for generating random session identifiers.
#[must_use]
pub fn generate_session_id() -> String {
  uuid::Uuid::new_v4().to_string()
}

/// State for the Progressive Discover flow.
#[derive(Clone, Debug, PartialEq)]
pub struct ProgressiveDiscoverState {
  /// Current phase in the discovery flow
  pub phase: ProgressiveDiscoverPhase,
  /// Current sub-phase when in `ConfirmingFields` phase
  pub sub_phase: ConfirmSubPhase,
  /// The interrogation transcript containing all extracted data
  pub transcript: InterrogationTranscript,
  /// Whether an async operation is in progress
  pub is_loading: bool,
  /// Error message if something went wrong
  pub error: Option<String>,
  /// Unique session identifier for persistence (T011)
  pub session_id: String,
  /// Antithesis (null hypothesis) points - 3 realistic reasons why users might reject
  pub antithesis_points: [String; 3],
  /// Quality score for antithesis points (0.0-1.0)
  pub antithesis_score: Option<f64>,
  /// Straw man traps detected during persona validation
  pub detected_traps: Vec<StrawManTrap>,
  /// Straw man traps acknowledged by user
  pub acknowledged_traps: Vec<StrawManTrap>,
  /// VORP (Value, Obvious, Real, Possible) validation result
  pub vorp_validation: Option<VorpValidation>,
  /// Hole punching validation results for scenario
  pub hole_punching: HolePunchingResults,
  /// Four Brutal Truths acknowledgment state
  pub brutal_truths: [bool; 4],
}

impl Default for ProgressiveDiscoverState {
  fn default() -> Self {
    Self {
      phase: ProgressiveDiscoverPhase::default(),
      sub_phase: ConfirmSubPhase::default(),
      transcript: InterrogationTranscript::default(),
      is_loading: false,
      error: None,
      session_id: generate_session_id(),
      antithesis_points: [String::new(), String::new(), String::new()],
      antithesis_score: None,
      detected_traps: Vec::new(),
      acknowledged_traps: Vec::new(),
      vorp_validation: None,
      hole_punching: HolePunchingResults::default(),
      brutal_truths: [false; 4],
    }
  }
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
      session_id: generate_session_id(),
      antithesis_points: [String::new(), String::new(), String::new()],
      antithesis_score: None,
      detected_traps: Vec::new(),
      acknowledged_traps: Vec::new(),
      vorp_validation: None,
      hole_punching: HolePunchingResults::default(),
      brutal_truths: [false; 4],
    }
  }

  /// Create state from a recovered persistable state (T011).
  #[must_use]
  pub fn from_persistable(persistable: PersistableState) -> Self {
    Self {
      phase: persistable.phase,
      sub_phase: persistable.sub_phase,
      transcript: persistable.transcript,
      is_loading: false,
      error: None,
      session_id: persistable.session_id,
      antithesis_points: persistable.antithesis_points,
      antithesis_score: persistable.antithesis_score,
      detected_traps: persistable.detected_traps,
      acknowledged_traps: persistable.acknowledged_traps,
      vorp_validation: persistable.vorp_validation,
      hole_punching: persistable.hole_punching,
      brutal_truths: persistable.brutal_truths,
    }
  }

  /// Convert to persistable state for storage.
  #[must_use]
  pub fn to_persistable(&self) -> PersistableState {
    PersistableState::from_state(self, &self.session_id)
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
      session_id: self.session_id,
      antithesis_points: self.antithesis_points,
      antithesis_score: self.antithesis_score,
      detected_traps: self.detected_traps,
      acknowledged_traps: self.acknowledged_traps,
      vorp_validation: self.vorp_validation,
      hole_punching: self.hole_punching,
      brutal_truths: self.brutal_truths,
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
      session_id: self.session_id,
      antithesis_points: self.antithesis_points,
      antithesis_score: self.antithesis_score,
      detected_traps: self.detected_traps,
      acknowledged_traps: self.acknowledged_traps,
      vorp_validation: self.vorp_validation,
      hole_punching: self.hole_punching,
      brutal_truths: self.brutal_truths,
    })
  }

  /// Transition to the next sub-phase within `ConfirmingFields`.
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
      session_id: self.session_id,
      antithesis_points: self.antithesis_points,
      antithesis_score: self.antithesis_score,
      detected_traps: self.detected_traps,
      acknowledged_traps: self.acknowledged_traps,
      vorp_validation: self.vorp_validation,
      hole_punching: self.hole_punching,
      brutal_truths: self.brutal_truths,
    })
  }

  /// Transition to the previous sub-phase within `ConfirmingFields`.
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
      session_id: self.session_id,
      antithesis_points: self.antithesis_points,
      antithesis_score: self.antithesis_score,
      detected_traps: self.detected_traps,
      acknowledged_traps: self.acknowledged_traps,
      vorp_validation: self.vorp_validation,
      hole_punching: self.hole_punching,
      brutal_truths: self.brutal_truths,
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
      session_id: self.session_id,
      antithesis_points: self.antithesis_points,
      antithesis_score: self.antithesis_score,
      detected_traps: self.detected_traps,
      acknowledged_traps: self.acknowledged_traps,
      vorp_validation: self.vorp_validation,
      hole_punching: self.hole_punching,
      brutal_truths: self.brutal_truths,
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
      session_id: self.session_id,
      antithesis_points: self.antithesis_points,
      antithesis_score: self.antithesis_score,
      detected_traps: self.detected_traps,
      acknowledged_traps: self.acknowledged_traps,
      vorp_validation: self.vorp_validation,
      hole_punching: self.hole_punching,
      brutal_truths: self.brutal_truths,
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
      session_id: self.session_id,
      antithesis_points: self.antithesis_points,
      antithesis_score: self.antithesis_score,
      detected_traps: self.detected_traps,
      acknowledged_traps: self.acknowledged_traps,
      vorp_validation: self.vorp_validation,
      hole_punching: self.hole_punching,
      brutal_truths: self.brutal_truths,
    }
  }

  /// Check if the current phase is `ConfirmingFields`.
  #[must_use]
  pub const fn is_confirming(&self) -> bool {
    matches!(self.phase, ProgressiveDiscoverPhase::ConfirmingFields)
  }

  /// Check if the state is in a terminal (locked) phase.
  #[must_use]
  pub const fn is_locked(&self) -> bool {
    matches!(self.phase, ProgressiveDiscoverPhase::Locked)
  }

  /// Set antithesis points and score.
  #[must_use]
  pub fn with_antithesis(self, points: [String; 3], score: f64) -> Self {
    Self {
      phase: self.phase,
      sub_phase: self.sub_phase,
      transcript: self.transcript,
      is_loading: self.is_loading,
      error: self.error,
      session_id: self.session_id,
      antithesis_points: points,
      antithesis_score: Some(score.clamp(0.0, 1.0)),
      detected_traps: self.detected_traps,
      acknowledged_traps: self.acknowledged_traps,
      vorp_validation: self.vorp_validation,
      hole_punching: self.hole_punching,
      brutal_truths: self.brutal_truths,
    }
  }

  /// Set detected straw man traps.
  #[must_use]
  pub fn with_detected_traps(self, traps: Vec<StrawManTrap>) -> Self {
    Self {
      phase: self.phase,
      sub_phase: self.sub_phase,
      transcript: self.transcript,
      is_loading: self.is_loading,
      error: self.error,
      session_id: self.session_id,
      antithesis_points: self.antithesis_points,
      antithesis_score: self.antithesis_score,
      detected_traps: traps,
      acknowledged_traps: self.acknowledged_traps,
      vorp_validation: self.vorp_validation,
      hole_punching: self.hole_punching,
      brutal_truths: self.brutal_truths,
    }
  }

  /// Acknowledge a straw man trap.
  #[must_use]
  pub fn with_acknowledged_trap(self, trap: StrawManTrap) -> Self {
    let mut acknowledged = self.acknowledged_traps.clone();
    if !acknowledged.contains(&trap) {
      acknowledged.push(trap);
    }
    Self {
      phase: self.phase,
      sub_phase: self.sub_phase,
      transcript: self.transcript,
      is_loading: self.is_loading,
      error: self.error,
      session_id: self.session_id,
      antithesis_points: self.antithesis_points,
      antithesis_score: self.antithesis_score,
      detected_traps: self.detected_traps,
      acknowledged_traps: acknowledged,
      vorp_validation: self.vorp_validation,
      hole_punching: self.hole_punching,
      brutal_truths: self.brutal_truths,
    }
  }

  /// Set VORP validation result.
  #[must_use]
  pub fn with_vorp_validation(self, validation: VorpValidation) -> Self {
    Self {
      phase: self.phase,
      sub_phase: self.sub_phase,
      transcript: self.transcript,
      is_loading: self.is_loading,
      error: self.error,
      session_id: self.session_id,
      antithesis_points: self.antithesis_points,
      antithesis_score: self.antithesis_score,
      detected_traps: self.detected_traps,
      acknowledged_traps: self.acknowledged_traps,
      vorp_validation: Some(validation),
      hole_punching: self.hole_punching,
      brutal_truths: self.brutal_truths,
    }
  }

  /// Set hole punching results.
  #[must_use]
  pub fn with_hole_punching(self, results: HolePunchingResults) -> Self {
    Self {
      phase: self.phase,
      sub_phase: self.sub_phase,
      transcript: self.transcript,
      is_loading: self.is_loading,
      error: self.error,
      session_id: self.session_id,
      antithesis_points: self.antithesis_points,
      antithesis_score: self.antithesis_score,
      detected_traps: self.detected_traps,
      acknowledged_traps: self.acknowledged_traps,
      vorp_validation: self.vorp_validation,
      hole_punching: results,
      brutal_truths: self.brutal_truths,
    }
  }

  /// Set a brutal truth acknowledgment.
  #[must_use]
  pub fn with_brutal_truth(self, index: usize, value: bool) -> Self {
    let mut truths = self.brutal_truths;
    if index < 4 {
      truths[index] = value;
    }
    Self {
      phase: self.phase,
      sub_phase: self.sub_phase,
      transcript: self.transcript,
      is_loading: self.is_loading,
      error: self.error,
      session_id: self.session_id,
      antithesis_points: self.antithesis_points,
      antithesis_score: self.antithesis_score,
      detected_traps: self.detected_traps,
      acknowledged_traps: self.acknowledged_traps,
      vorp_validation: self.vorp_validation,
      hole_punching: self.hole_punching,
      brutal_truths: truths,
    }
  }

  /// Check if all brutal truths are acknowledged.
  #[must_use]
  pub fn are_all_brutal_truths_acknowledged(&self) -> bool {
    self.brutal_truths.iter().all(|&t| t)
  }

  /// Check if can advance from the current sub-phase.
  ///
  /// This validates that required fields are complete before allowing advancement.
  #[must_use]
  pub fn can_advance_from_subphase(&self) -> bool {
    match self.sub_phase {
      ConfirmSubPhase::ConfirmProblem => {
        // Problem must have content
        !self.transcript.problem.content.trim().is_empty()
      }
      ConfirmSubPhase::ConfirmPersona => {
        // Persona must have content and no unacknowledged traps
        !self.transcript.persona.content.trim().is_empty()
          && self
            .detected_traps
            .iter()
            .all(|t| self.acknowledged_traps.contains(t))
      }
      ConfirmSubPhase::ConfirmSolution => {
        // Solution must have content
        !self.transcript.solution.content.trim().is_empty()
      }
      ConfirmSubPhase::ConfirmNonpersona => {
        // Nonpersona must have content
        !self.transcript.nonpersona.content.trim().is_empty()
      }
      ConfirmSubPhase::ConfirmScenario => {
        // Scenario must be complete with hole punching
        self.transcript.scenario.is_complete()
      }
    }
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
#[must_use]
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

  /// Advance to the next sub-phase within `ConfirmingFields`.
  pub fn advance_sub_phase(&mut self) {
    let current = self.state.read().clone();
    if let Some(next) = current.advance_sub_phase() {
      self.state.set(next);
    }
  }

  /// Regress to the previous sub-phase within `ConfirmingFields`.
  pub fn regress_sub_phase(&mut self) {
    let current = self.state.read().clone();
    if let Some(prev) = current.regress_sub_phase() {
      self.state.set(prev);
    }
  }

  /// Set antithesis points and score.
  pub fn set_antithesis(&mut self, points: [String; 3], score: f64) {
    let current = self.state.read().clone();
    self.state.set(current.with_antithesis(points, score));
  }

  /// Acknowledge a straw man trap.
  pub fn acknowledge_trap(&mut self, trap: StrawManTrap) {
    let current = self.state.read().clone();
    self.state.set(current.with_acknowledged_trap(trap));
  }

  /// Set a brutal truth acknowledgment.
  pub fn set_brutal_truth(&mut self, index: usize, value: bool) {
    let current = self.state.read().clone();
    self.state.set(current.with_brutal_truth(index, value));
  }

  /// Check if can advance from the current sub-phase.
  #[must_use]
  pub fn can_advance_from_subphase(&self) -> bool {
    self.state.read().can_advance_from_subphase()
  }

  /// Check if all brutal truths are acknowledged.
  #[must_use]
  pub fn are_all_brutal_truths_acknowledged(&self) -> bool {
    self.state.read().are_all_brutal_truths_acknowledged()
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
#[must_use]
#[allow(clippy::missing_const_for_fn)]
pub fn use_progressive_discover_actions(
  state: Signal<ProgressiveDiscoverState>,
) -> ProgressiveDiscoverActions {
  ProgressiveDiscoverActions::new(state)
}

// ============================================================================
// Auto-Save Hook (T010)
// ============================================================================

/// Hook for auto-saving Progressive Discover state to localStorage.
///
/// This hook sets up an effect that saves the state to localStorage whenever
/// it changes, with a 500ms debounce to avoid excessive writes.
///
/// # Example
///
/// ```rust,ignore
/// let state = use_progressive_discover();
/// use_auto_save(state);
/// ```
pub fn use_auto_save(state: Signal<ProgressiveDiscoverState>) {
  // Track last save time using a signal for interior mutability
  let mut last_save_instant = use_signal(|| None::<Instant>);

  use_effect(move || {
    // Read state to establish dependency
    let current_state = state.read();
    let phase = current_state.phase;
    let session_id = current_state.session_id.clone();
    let persistable = current_state.to_persistable();
    drop(current_state);

    // Check if we should save now or debounce
    let should_save = last_save_instant
      .read()
      .is_none_or(|last| last.elapsed() >= Duration::from_millis(AUTO_SAVE_DEBOUNCE_MS));

    if should_save {
      // Save immediately
      match save_to_local_storage(&persistable) {
        Ok(()) => {
          *last_save_instant.write() = Some(Instant::now());
          tracing::debug!(
              session_id = %session_id,
              phase = ?phase,
              "Auto-save completed"
          );
        }
        Err(e) => {
          tracing::warn!("Auto-save failed: {e}");
        }
      }
    }
    // If not should_save, the change will trigger another effect after debounce period
  });
}

// ============================================================================
// Crash Recovery Hook (T011)
// ============================================================================

/// Result of checking for crash recovery.
#[derive(Clone, Debug, PartialEq)]
pub enum CrashRecoveryStatus {
  /// No recoverable session found
  NoSession,
  /// A recoverable session was found
  Recoverable {
    /// The recovered state
    state: Box<ProgressiveDiscoverState>,
    /// When the session was saved
    saved_at: String,
  },
}

/// Hook for checking and recovering from crash.
///
/// This hook should be called once when the component mounts to check
/// if there's a recoverable session from a previous crash.
///
/// # Returns
///
/// - `CrashRecoveryStatus::NoSession` if no session to recover
/// - `CrashRecoveryStatus::Recoverable { state, saved_at }` if a session was found
///
/// # Example
///
/// ```rust,ignore
/// let recovery_status = use_crash_recovery_check();
///
/// match recovery_status {
///     CrashRecoveryStatus::Recoverable { state, saved_at } => {
///         // Show recovery dialog
///     }
///     CrashRecoveryStatus::NoSession => {
///         // Start fresh
///     }
/// }
/// ```
#[must_use]
pub fn use_crash_recovery_check() -> CrashRecoveryStatus {
  // This is not a reactive hook - it runs once on mount
  load_from_local_storage().map_or(CrashRecoveryStatus::NoSession, |persistable| {
    let saved_at = persistable.saved_at.clone();
    let state = Box::new(ProgressiveDiscoverState::from_persistable(persistable));
    CrashRecoveryStatus::Recoverable { state, saved_at }
  })
}

/// Hook for managing Progressive Discover state with crash recovery.
///
/// This hook combines state initialization with crash recovery checking.
/// It returns the state signal and an optional recovery status.
///
/// # Returns
///
/// A tuple containing:
/// - The state signal
/// - An optional recovery status (Some if a recoverable session exists)
///
/// # Example
///
/// ```rust,ignore
/// let (state, recovery_status) = use_progressive_discover_with_recovery();
///
/// // In your component, show a recovery dialog if recovery_status is Some
/// if let Some(CrashRecoveryStatus::Recoverable { state: recovered, saved_at }) = recovery_status {
///     // Show dialog asking user if they want to recover
/// }
/// ```
pub fn use_progressive_discover_with_recovery() -> (
  Signal<ProgressiveDiscoverState>,
  Option<CrashRecoveryStatus>,
) {
  let recovery_status = use_crash_recovery_check();

  let initial_state = match &recovery_status {
    CrashRecoveryStatus::Recoverable { state, .. } => (**state).clone(),
    CrashRecoveryStatus::NoSession => ProgressiveDiscoverState::default(),
  };

  let state = use_signal(|| initial_state);

  (state, Some(recovery_status))
}

/// Hook for managing Progressive Discover state with crash recovery and auto-save.
///
/// This is the complete hook that combines:
/// - State management
/// - Crash recovery on mount
/// - Auto-save on state changes (with debounce)
///
/// # Returns
///
/// A tuple containing:
/// - The state signal
/// - The recovery status (None if no recovery needed)
///
/// # Example
///
/// ```rust,ignore
/// let (state, recovery_status) = use_progressive_discover_full();
///
/// // Auto-save is automatically set up
/// // Recovery status can be used to show a dialog
/// ```
pub fn use_progressive_discover_full() -> (Signal<ProgressiveDiscoverState>, CrashRecoveryStatus) {
  let recovery_status = use_crash_recovery_check();

  let initial_state = match &recovery_status {
    CrashRecoveryStatus::Recoverable { state, .. } => (**state).clone(),
    CrashRecoveryStatus::NoSession => ProgressiveDiscoverState::default(),
  };

  let state = use_signal(|| initial_state);

  // Set up auto-save
  use_auto_save(state);

  (state, recovery_status)
}

/// Clear any saved session and start fresh.
///
/// This function clears the localStorage and returns a new default state.
/// Use this when the user explicitly chooses to start fresh instead of recovering.
pub fn clear_and_start_fresh() -> ProgressiveDiscoverState {
  clear_local_storage();
  ProgressiveDiscoverState::default()
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

    let state = state.advance_phase();
    assert!(state.is_some());
    let Some(state) = state else {
      return;
    };
    assert_eq!(state.phase, ProgressiveDiscoverPhase::Extracting);

    let state = state.advance_phase();
    assert!(state.is_some());
    let Some(state) = state else {
      return;
    };
    assert_eq!(state.phase, ProgressiveDiscoverPhase::ConfirmingFields);

    let state = state.advance_phase();
    assert!(state.is_some());
    let Some(state) = state else {
      return;
    };
    assert_eq!(state.phase, ProgressiveDiscoverPhase::Preview);

    let state = state.advance_phase();
    assert!(state.is_some());
    let Some(state) = state else {
      return;
    };
    assert_eq!(state.phase, ProgressiveDiscoverPhase::KirkCompilation);

    let state = state.advance_phase();
    assert!(state.is_some());
    let Some(state) = state else {
      return;
    };
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

    let state = state.regress_phase();
    assert!(state.is_some());
    let Some(state) = state else {
      return;
    };
    assert_eq!(state.phase, ProgressiveDiscoverPhase::KirkCompilation);

    let state = state.regress_phase();
    assert!(state.is_some());
    let Some(state) = state else {
      return;
    };
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

    let state = state.advance_sub_phase();
    assert!(state.is_some());
    let Some(state) = state else {
      return;
    };
    assert_eq!(state.sub_phase, ConfirmSubPhase::ConfirmPersona);

    let state = state.advance_sub_phase();
    assert!(state.is_some());
    let Some(state) = state else {
      return;
    };
    assert_eq!(state.sub_phase, ConfirmSubPhase::ConfirmSolution);

    let state = state.advance_sub_phase();
    assert!(state.is_some());
    let Some(state) = state else {
      return;
    };
    assert_eq!(state.sub_phase, ConfirmSubPhase::ConfirmNonpersona);

    let state = state.advance_sub_phase();
    assert!(state.is_some());
    let Some(state) = state else {
      return;
    };
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

    let state = state.regress_sub_phase();
    assert!(state.is_some());
    let Some(state) = state else {
      return;
    };
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
    let state =
      ProgressiveDiscoverState::default().with_error(Some("Something went wrong".to_string()));
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

    let state = state.advance_phase();
    assert!(state.is_some());
    let Some(state) = state else {
      return;
    };
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

    let new_transcript = InterrogationTranscript::from_prompt("New prompt".to_string());
    let updated = original.with_transcript(new_transcript);

    assert_eq!(updated.phase, ProgressiveDiscoverPhase::Extracting);
    assert_eq!(updated.sub_phase, ConfirmSubPhase::ConfirmSolution);
    assert!(updated.is_loading);
    assert_eq!(updated.error, Some("Error".to_string()));
    assert_eq!(updated.transcript.original_prompt, "New prompt");
  }
}
