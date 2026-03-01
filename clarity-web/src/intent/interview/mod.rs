//! Interview Submodule
//!
//! Core interview session management including:
//! - Session creation and lifecycle
//! - Gap detection (profile-specific required fields)
//! - Conflict detection (CAP theorem, anonymous+audit, perspective conflicts)
//! - JSONL storage and session diffing
//! - Question loading and management
//! - Contract validation
//! - Progress formatting for terminal display
//! - Answer extraction from free-text responses
//! - Answer file parsing (TOML/JSON formats)
//!
//! # Phase and Stage Management
//!
//! The interview system uses two complementary concepts to track progress:
//!
//! ## `InterviewStage`
//!
//! [`InterviewStage`] represents the high-level lifecycle phase of an interview session.
//! It follows a state machine pattern with explicit transition rules:
//!
//! ```text
//! Discovery -> Refinement -> Validation -> Complete
//!      |            |             |
//!      v            v             v
//!   Paused <-------+-------------+
//!      |
//!      +-> (resume to any active stage)
//! ```
//!
//! ### Stage Transitions
//!
//! | From | Valid To |
//! |------|----------|
//! | Discovery | Discovery, Refinement, Validation, Paused |
//! | Refinement | Refinement, Validation, Complete, Paused |
//! | Validation | Validation, Complete, Paused |
//! | Paused | Discovery, Refinement, Validation, Paused |
//! | Complete | Complete (terminal) |
//!
//! Use [`InterviewStage::can_transition_to`] to check validity and
//! [`InterviewStage::transition_to`] to attempt a transition.
//!
//! ## Phase Tracking
//!
//! Within each stage, the session tracks finer-grained progress via phases:
//!
//! - [`InterviewSession::current_phase`]: The next phase to be completed (starts at 1)
//! - [`InterviewSession::completed_phases`]: List of phases that have been marked complete
//!
//! ### Phase Properties
//!
//! - Phases are 1-indexed (phase 0 is invalid)
//! - Phases can be completed out of order (e.g., complete phase 3 before phase 1)
//! - `current_phase` only advances when the *current* phase is completed
//! - Completing the same phase multiple times is idempotent (recorded once)
//! - Phase completion works regardless of [`InterviewStage`]
//!
//! ### Example: Sequential Phase Completion
//!
//! ```ignore
//! use clarity_web::intent::interview::types::{InterviewSession, Profile};
//!
//! let mut session = InterviewSession::new(
//!     "session-1".to_string(),
//!     Profile::Api,
//!     "2024-01-01T00:00:00Z".to_string(),
//! );
//!
//! // Initially at phase 1
//! assert_eq!(session.current_phase, 1);
//! assert!(session.completed_phases.is_empty());
//!
//! // Complete phase 1
//! session.complete_phase(1, "2024-01-01T00:01:00Z").unwrap();
//! assert_eq!(session.current_phase, 2);
//! assert_eq!(session.completed_phases, vec![1]);
//! ```
//!
//! ### Example: Out-of-Order Phase Completion
//!
//! ```ignore
//! // Complete phase 3 before phase 1
//! session.complete_phase(3, "t1").unwrap();
//! assert!(session.completed_phases.contains(&3));
//! assert_eq!(session.current_phase, 1); // unchanged - not the current phase
//!
//! // Now complete phase 1
//! session.complete_phase(1, "t2").unwrap();
//! assert_eq!(session.current_phase, 2); // advances now
//! ```
//!
//! ## Gap and Conflict State Machines
//!
//! The session also manages two additional state machines:
//!
//! ### `GapState`
//!
//! [`GapState`] tracks missing required information:
//!
//! ```text
//! Open -> Resolved(resolution: String)
//! ```
//!
//! - One-way transition (cannot reopen a resolved gap)
//! - Resolution text must be non-empty
//! - Use [`GapState::resolve`] or [`GapState::transition_to`]
//!
//! ### `ConflictState`
//!
//! [`ConflictState`] tracks conflicting answers:
//!
//! ```text
//! Pending -> Resolved(chosen_index: i32)
//! ```
//!
//! - One-way transition (cannot unresolve a conflict)
//! - Chosen index must be non-negative and within bounds
//! - Use [`ConflictState::resolve`] or [`ConflictState::transition_to`]
//!
//! ## Error Handling
//!
//! All state transitions return `Result` with specific error types:
//!
//! - [`InterviewSessionError`]: Session-level errors (invalid phase, round mismatch, etc.)
//! - [`InterviewStageError`]: Stage transition errors
//! - [`GapStateError`]: Gap resolution errors
//! - [`ConflictStateError`]: Conflict resolution errors
//! - [`InterviewError`]: General interview errors
//!
//! [`InterviewStage`]: types::InterviewStage
//! [`InterviewSession::current_phase`]: types::InterviewSession::current_phase
//! [`InterviewSession::completed_phases`]: types::InterviewSession::completed_phases
//! [`GapState`]: types::GapState
//! [`ConflictState`]: types::ConflictState
//! [`InterviewSessionError`]: types::InterviewSessionError
//! [`InterviewStageError`]: types::InterviewStageError
//! [`GapStateError`]: types::GapStateError
//! [`ConflictStateError`]: types::ConflictStateError
//! [`InterviewError`]: types::InterviewError

pub mod answer_extraction;
pub mod answer_file;
pub mod formatting;
pub mod interview_contract;
pub mod interview_questions;
pub mod question_loader;
pub mod question_types;
pub mod storage;
pub mod types;

// Re-export types
pub use answer_extraction::*;
pub use answer_file::*;
pub use formatting::*;
pub use interview_contract::*;
pub use interview_questions::*;
pub use question_loader::*;
pub use question_types::*;
pub use storage::*;
pub use types::*;
