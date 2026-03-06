#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PlanError {
  #[error("bead ID cannot be empty")]
  EmptyBeadId,
  #[error("bead title cannot be empty")]
  EmptyBeadTitle,
  #[error("dependency '{dependency}' refers to non-existent bead '{bead_id}'")]
  InvalidDependency { bead_id: String, dependency: String },
  #[error("circular dependency detected in bead graph")]
  CircularDependency,
  #[error("invalid phase number: {phase_number}")]
  InvalidPhaseNumber { phase_number: u32 },
  #[error("session ID cannot be empty")]
  EmptySessionId,
  #[error("no beads available to plan")]
  NoBeads,
  #[error("duplicate bead ID: {0}")]
  DuplicateBeadId(String),
  #[error("invalid state transition for bead '{bead_id}': {from} -> {to}")]
  InvalidStateTransition {
    bead_id: String,
    from: String,
    to: String,
  },
}
