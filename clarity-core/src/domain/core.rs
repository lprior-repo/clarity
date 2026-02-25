#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Functional Core with Pure Business Logic
//!
//! This module implements the functional core pattern:
//! - Pure functions with no side effects
//! - Immutable data structures
//! - Zero unwrap philosophy
//! - Railway pattern for error handling
//! - Persistent state using rpds

use super::models::ModelError;
use super::{Bead, BeadId, BeadPriority, BeadStatus, BeadType, UserId};
use rpds::Vector;
use std::collections::HashMap;

/// Core domain error types (pure, thiserror-based)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
  /// Invalid bead status transition
  InvalidStatusTransition { from: BeadStatus, to: BeadStatus },

  /// Bead not found
  BeadNotFound(BeadId),

  /// Priority cannot be changed from high to low
  HighPriorityDowngrade {
    bead_id: BeadId,
    from: BeadPriority,
    to: BeadPriority,
  },

  /// Cannot close a blocked bead
  CannotCloseBlockedBead(BeadId),

  /// Invalid tag length (must be 1-50 chars)
  InvalidTagLength(String),
}

impl std::fmt::Display for DomainError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::InvalidStatusTransition { from, to } => {
        write!(
          f,
          "Cannot transition from {} to {}",
          from.as_str(),
          to.as_str()
        )
      }
      Self::BeadNotFound(id) => write!(f, "Bead {id} not found"),
      Self::HighPriorityDowngrade { bead_id, from, to } => {
        write!(
          f,
          "Cannot downgrade bead {bead_id} priority from {} to {}",
          from.as_str(),
          to.as_str()
        )
      }
      Self::CannotCloseBlockedBead(id) => {
        write!(f, "Cannot close blocked bead {id}")
      }
      Self::InvalidTagLength(tag) => {
        write!(f, "Tag '{tag}' must be 1-50 characters long")
      }
    }
  }
}

impl std::error::Error for DomainError {}

/// Immutable domain state using persistent collections
#[derive(Clone, Debug)]
pub struct DomainState {
  /// All beads in the system (persistent Vector)
  pub beads: Vector<Bead>,
}

impl Default for DomainState {
  fn default() -> Self {
    Self::new()
  }
}

impl DomainState {
  /// Create empty domain state
  #[must_use]
  pub fn new() -> Self {
    Self {
      beads: Vector::new(),
    }
  }

  /// Get bead by ID (pure function - returns Option)
  #[must_use]
  pub fn get_bead(&self, id: BeadId) -> Option<&Bead> {
    self.beads.iter().find(|bead| bead.id == id)
  }

  /// Get all beads (pure function - returns new Vector)
  #[must_use]
  pub fn get_all_beads(&self) -> Vector<Bead> {
    self.beads.clone()
  }

  /// Filter beads by status (pure function - returns new Vector)
  #[must_use]
  pub fn filter_by_status(&self, status: BeadStatus) -> Vector<Bead> {
    self
      .beads
      .iter()
      .filter(|bead| bead.status == status)
      .cloned()
      .collect()
  }

  /// Filter beads by priority (pure function - returns new Vector)
  #[must_use]
  pub fn filter_by_priority(&self, priority: BeadPriority) -> Vector<Bead> {
    self
      .beads
      .iter()
      .filter(|bead| bead.priority == priority)
      .cloned()
      .collect()
  }

  /// Filter beads by type (pure function - returns new Vector)
  #[must_use]
  pub fn filter_by_type(&self, bead_type: BeadType) -> Vector<Bead> {
    self
      .beads
      .iter()
      .filter(|bead| bead.bead_type == bead_type)
      .cloned()
      .collect()
  }

  /// Count beads by status (pure function - returns `HashMap`).
  #[must_use]
  pub fn count_by_status(&self) -> HashMap<BeadStatus, usize> {
    self.beads.iter().fold(HashMap::new(), |mut counts, bead| {
      // Using a mutable fold here for performance,
      // but this could be made fully immutable with iterators
      *counts.entry(bead.status).or_default() += 1;
      counts
    })
  }

  /// Get statistics (pure function - returns `DomainStats`).
  #[must_use]
  pub fn statistics(&self) -> DomainStats {
    let total = self.beads.len();
    let by_status = self.count_by_status();

    DomainStats {
      total_beads: total,
      open_beads: by_status.get(&BeadStatus::Open).copied().map_or(0, |v| v),
      in_progress_beads: by_status
        .get(&BeadStatus::InProgress)
        .copied()
        .map_or(0, |v| v),
      blocked_beads: by_status
        .get(&BeadStatus::Blocked)
        .copied()
        .map_or(0, |v| v),
      deferred_beads: by_status
        .get(&BeadStatus::Deferred)
        .copied()
        .map_or(0, |v| v),
      closed_beads: by_status.get(&BeadStatus::Closed).copied().map_or(0, |v| v),
    }
  }
}

/// Domain statistics (pure value object)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainStats {
  pub total_beads: usize,
  pub open_beads: usize,
  pub in_progress_beads: usize,
  pub blocked_beads: usize,
  pub deferred_beads: usize,
  pub closed_beads: usize,
}

impl DomainStats {
  /// Check if any beads are blocked
  #[must_use]
  pub const fn has_blocked_beads(&self) -> bool {
    self.blocked_beads > 0
  }

  /// Calculate completion percentage
  #[must_use]
  pub fn completion_percentage(&self) -> f64 {
    if self.total_beads == 0 {
      0.0
    } else {
      let closed_u32 = u32::try_from(self.closed_beads).unwrap_or(u32::MAX);
      let total_u32 = u32::try_from(self.total_beads).unwrap_or(u32::MAX);
      f64::from(closed_u32) / f64::from(total_u32) * 100.0
    }
  }

  /// Check if project is complete
  #[must_use]
  pub const fn is_complete(&self) -> bool {
    self.total_beads > 0 && self.open_beads == 0 && self.in_progress_beads == 0
  }
}

// Pure business logic functions (functional core)

/// Validate bead status transition (pure function)
///
/// # Errors
/// Returns a `DomainError` if the transition is not allowed.
pub const fn validate_status_transition(
  current: BeadStatus,
  target: BeadStatus,
) -> Result<(), DomainError> {
  if !current.can_transition_to(target) {
    return Err(DomainError::InvalidStatusTransition {
      from: current,
      to: target,
    });
  }
  Ok(())
}

/// Tag validation function (pure function)
///
/// # Errors
/// Returns a `DomainError` if the tag length is invalid.
pub fn validate_tag_length(tag: &str) -> Result<(), DomainError> {
  if tag.is_empty() || tag.len() > 50 {
    return Err(DomainError::InvalidTagLength(tag.to_string()));
  }
  Ok(())
}

/// Create a new bead with validation (pure function)
///
/// # Errors
/// Returns a `ModelError` if bead creation fails.
pub fn create_bead(
  title: String,
  description: Option<String>,
  priority: BeadPriority,
  bead_type: BeadType,
  created_by: Option<UserId>,
) -> Result<Bead, ModelError> {
  Bead::new(
    title,
    description,
    BeadStatus::Open,
    priority,
    bead_type,
    created_by,
  )
}

/// Update bead priority with business rules (pure function)
///
/// # Errors
/// Returns a `DomainError` if the priority downgrade is not allowed.
pub fn update_bead_priority(bead: &Bead, new_priority: BeadPriority) -> Result<Bead, DomainError> {
  // Business rule: Cannot downgrade high priority
  if bead.priority == BeadPriority::High && new_priority.sort_value() > bead.priority.sort_value() {
    return Err(DomainError::HighPriorityDowngrade {
      bead_id: bead.id,
      from: bead.priority,
      to: new_priority,
    });
  }

  // Create new bead with updated priority (immutability)
  Ok(Bead {
    id: bead.id,
    title: bead.title.clone(),
    description: bead.description.clone(),
    priority: new_priority,
    bead_type: bead.bead_type,
    created_by: bead.created_by,
    status: bead.status,
    created_at: bead.created_at,
    updated_at: chrono::Utc::now(),
  })
}

/// Close a bead with validation (pure function)
///
/// # Errors
/// Returns a `DomainError` if the bead is blocked or transition is invalid.
pub fn close_bead(bead: &Bead) -> Result<Bead, DomainError> {
  // Business rule: Cannot close blocked beads
  if bead.status == BeadStatus::Blocked {
    return Err(DomainError::CannotCloseBlockedBead(bead.id));
  }

  validate_status_transition(bead.status, BeadStatus::Closed)?;

  Ok(Bead {
    id: bead.id,
    title: bead.title.clone(),
    description: bead.description.clone(),
    priority: bead.priority,
    bead_type: bead.bead_type,
    created_by: bead.created_by,
    status: BeadStatus::Closed,
    created_at: bead.created_at,
    updated_at: chrono::Utc::now(),
  })
}

// State transition functions (pure - return new state)

/// Add a bead to the domain (pure function)
#[must_use]
#[expect(
  clippy::needless_pass_by_value,
  reason = "Domain functions intentionally consume immutable state"
)]
pub fn add_bead(state: DomainState, bead: Bead) -> DomainState {
  DomainState {
    beads: state.beads.push_back(bead),
  }
}

/// Set beads for the domain (pure function)
#[must_use]
pub fn with_beads(_state: DomainState, beads: Vector<Bead>) -> DomainState {
  DomainState { beads }
}

/// Update a bead in the domain (pure function)
///
/// # Errors
/// Returns a `DomainError` if the bead is not found.
#[expect(
  clippy::needless_pass_by_value,
  reason = "Domain functions intentionally consume immutable state"
)]
pub fn update_bead(state: DomainState, updated_bead: Bead) -> Result<DomainState, DomainError> {
  // Ensure bead exists before updating
  if state.get_bead(updated_bead.id).is_none() {
    return Err(DomainError::BeadNotFound(updated_bead.id));
  }

  // Update beads vector
  let new_beads = state
    .beads
    .iter()
    .map(|bead| {
      if bead.id == updated_bead.id {
        updated_bead.clone()
      } else {
        bead.clone()
      }
    })
    .collect();

  Ok(DomainState { beads: new_beads })
}

/// Change bead status with validation (pure function)
///
/// # Errors
/// Returns a `DomainError` if the bead is not found or transition is invalid.
pub fn change_bead_status(
  state: DomainState,
  bead_id: BeadId,
  new_status: BeadStatus,
) -> Result<DomainState, DomainError> {
  // Find the bead
  let bead = state
    .get_bead(bead_id)
    .ok_or(DomainError::BeadNotFound(bead_id))?;

  // Validate transition
  validate_status_transition(bead.status, new_status)?;

  // Create updated bead
  let updated_bead = Bead {
    id: bead.id,
    title: bead.title.clone(),
    description: bead.description.clone(),
    priority: bead.priority,
    bead_type: bead.bead_type,
    created_by: bead.created_by,
    status: new_status,
    created_at: bead.created_at,
    updated_at: chrono::Utc::now(),
  };

  update_bead(state, updated_bead)
}

// Pipeline functions (functional composition example)

/// Process beads through a pipeline of operations
pub fn process_bead_pipeline(
  beads: Vector<Bead>,
  operations: &[fn(&Bead) -> Option<Bead>],
) -> Vector<Bead> {
  operations.iter().fold(beads, |current_beads, operation| {
    current_beads.iter().filter_map(operation).collect()
  })
}

/// Example pipeline operation: Filter high priority beads
#[must_use]
pub fn filter_high_priority(bead: &Bead) -> Option<Bead> {
  if bead.priority.is_high() {
    Some(bead.clone())
  } else {
    None
  }
}

/// Example pipeline operation: Filter non-blocked beads
#[must_use]
pub fn filter_non_blocked(bead: &Bead) -> Option<Bead> {
  if bead.status == BeadStatus::Blocked {
    None
  } else {
    Some(bead.clone())
  }
}

/// Generate bead report (pure function)
#[must_use]
pub fn generate_bead_report(state: &DomainState) -> String {
  let report_stats = state.statistics();

  format!(
    "Bead Report\n\
        ===========\n\
        Total Beads: {}\n\
        Open: {}\n\
        In Progress: {}\n\
        Blocked: {}\n\
        Deferred: {}\n\
        Closed: {}\n\
        Completion: {:.1}%\n\
        Blocked Beads: {}",
    report_stats.total_beads,
    report_stats.open_beads,
    report_stats.in_progress_beads,
    report_stats.blocked_beads,
    report_stats.deferred_beads,
    report_stats.closed_beads,
    report_stats.completion_percentage(),
    if report_stats.has_blocked_beads() {
      "Yes"
    } else {
      "No"
    }
  )
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::expect_used)]
  #![allow(clippy::panic)]
  #![allow(unused_variables)]
  #![allow(unused_imports)]

  use super::*;

  #[test]
  fn test_domain_state_new() {
    let state = DomainState::new();
    assert_eq!(state.beads.len(), 0);
  }

  #[test]
  fn test_validate_status_transition() {
    // Valid transitions
    assert!(validate_status_transition(BeadStatus::Open, BeadStatus::InProgress).is_ok());
    assert!(validate_status_transition(BeadStatus::InProgress, BeadStatus::Closed).is_ok());

    // Invalid transitions
    assert!(validate_status_transition(BeadStatus::Closed, BeadStatus::InProgress).is_err());
  }
}
