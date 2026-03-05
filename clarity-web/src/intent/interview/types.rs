//! Interview Types - Core data structures for interview sessions.
//!
//! This module provides the core types for managing interview sessions, including:
//!
//! # State Machines
//!
//! ## `InterviewStage`
//!
//! The [`InterviewStage`] enum represents the high-level lifecycle of an interview:
//!
//! - **Discovery**: Initial information gathering (rounds 1-2)
//! - **Refinement**: Deep-dive into details (round 3)
//! - **Validation**: Confirm understanding (round 4)
//! - **Complete**: Session finished (round 5+)
//! - **Paused**: Temporarily suspended
//!
//! See [module documentation](../index.html#phase-and-stage-management) for transition rules.
//!
//! ## `GapState`
//!
//! [`GapState`] tracks missing required fields:
//!
//! ```
//! # use clarity_web::intent::interview::types::GapState;
//! let open = GapState::Open;
//! assert!(open.is_open());
//!
//! let resolved = open.resolve("Provided default value".to_string()).unwrap();
//! assert!(resolved.is_resolved());
//! assert_eq!(resolved.resolution(), Some("Provided default value"));
//! ```
//!
//! ## `ConflictState`
//!
//! [`ConflictState`] tracks conflicting answers between questions:
//!
//! ```
//! # use clarity_web::intent::interview::types::ConflictState;
//! let pending = ConflictState::Pending;
//! assert!(pending.is_pending());
//!
//! let resolved = pending.resolve(1, 3).unwrap(); // Choose option 1 of 3
//! assert!(resolved.is_resolved());
//! assert_eq!(resolved.chosen_index(), Some(1));
//! ```
//!
//! # Session Management
//!
//! [`InterviewSession`] is the main aggregate containing:
//!
//! - Profile-specific configuration
//! - Current stage and phase tracking
//! - Collected answers, gaps, and conflicts
//! - Timestamps for lifecycle events
//!
//! # Error Types
//!
//! Each state machine has a corresponding error type:
//!
//! - [`InterviewStageError`]: Invalid stage transitions
//! - [`GapStateError`]: Gap resolution failures
//! - [`ConflictStateError`]: Conflict resolution failures
//! - [`InterviewSessionError`]: Session operation failures
//! - [`InterviewError`]: General interview errors

mod conflict_detection;
mod enums;
mod errors;
mod models;
mod session;

pub use enums::*;
pub use errors::*;
pub use models::*;

#[cfg(test)]
mod tests;
