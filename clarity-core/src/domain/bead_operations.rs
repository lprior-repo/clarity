#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Functional operations for bead management
//!
//! This module provides pure functional operations for working with beads,
//! following the functional core, imperative shell pattern.

use crate::domain::models::{Bead, ModelError};
use crate::domain::types::{BeadId, BeadPriority, BeadStatus, BeadType, UserId};
use itertools::Itertools;
use rpds::Vector;
use std::collections::HashMap;

/// State for bead operations
#[derive(Clone, Debug, Default)]
pub struct BeadState {
  beads: Vector<Bead>,
}

/// Result type for bead operations
pub type BeadResult<T> = Result<T, BeadError>;

/// Bead operation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeadError {
  NotFound(BeadId),
  InvalidTransition { from: BeadStatus, to: BeadStatus },
  PermissionDenied,
  DuplicateId(BeadId),
}

impl std::fmt::Display for BeadError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::NotFound(id) => write!(f, "Bead not found: {id}"),
      Self::InvalidTransition { from, to } => {
        write!(f, "Cannot transition from {from} to {to}")
      }
      Self::PermissionDenied => write!(f, "Permission denied"),
      Self::DuplicateId(id) => write!(f, "Duplicate bead ID: {id}"),
    }
  }
}

impl std::error::Error for BeadError {}

impl BeadError {
  #[must_use]
  #[expect(
    clippy::missing_const_for_fn,
    reason = "Constructor kept non-const for consistency"
  )]
  pub fn not_found(id: BeadId) -> Self {
    Self::NotFound(id)
  }

  #[must_use]
  #[expect(
    clippy::missing_const_for_fn,
    reason = "Constructor kept non-const for consistency"
  )]
  pub fn invalid_transition(from: BeadStatus, to: BeadStatus) -> Self {
    Self::InvalidTransition { from, to }
  }
}

impl BeadState {
  /// Create a new empty bead state
  #[must_use]
  pub fn new() -> Self {
    Self {
      beads: Vector::new(),
    }
  }

  /// Add a new bead to the state
  ///
  /// # Errors
  /// Returns `BeadError::DuplicateId` when a bead with the same ID already exists.
  pub fn add_bead(self, bead: Bead) -> Result<Self, BeadError> {
    // Check for duplicate ID
    if self.beads.iter().any(|b| b.id == bead.id) {
      return Err(BeadError::DuplicateId(bead.id));
    }

    let beads = self.beads.push_back(bead);
    Ok(Self { beads })
  }

  /// Update a bead's status
  ///
  /// # Errors
  /// Returns `BeadError` when the bead is missing or the transition is invalid.
  pub fn update_status(self, bead_id: BeadId, new_status: BeadStatus) -> Result<Self, BeadError> {
    // Find the bead
    let bead = self
      .beads
      .iter()
      .find(|b| b.id == bead_id)
      .ok_or_else(|| BeadError::not_found(bead_id))?;

    // Validate transition
    if !bead.status.can_transition_to(new_status) {
      return Err(BeadError::invalid_transition(bead.status, new_status));
    }

    // Create updated bead with updated timestamp
    let updated_bead = Bead {
      id: bead.id,
      title: bead.title.clone(),
      description: bead.description.clone(),
      status: new_status,
      priority: bead.priority,
      bead_type: bead.bead_type,
      created_by: bead.created_by,
      created_at: bead.created_at,
      updated_at: chrono::Utc::now(),
    };

    // Replace the bead in the vector
    let beads = self
      .beads
      .iter()
      .filter(|b| b.id != bead_id)
      .cloned()
      .collect::<Vec<_>>()
      .into_iter()
      .collect::<Vector<_>>()
      .push_back(updated_bead);

    Ok(Self { beads })
  }

  /// Filter beads by status
  #[must_use]
  pub fn filter_by_status(&self, status: BeadStatus) -> Vec<Bead> {
    self
      .beads
      .iter()
      .filter(|bead| bead.status == status)
      .cloned()
      .collect()
  }

  /// Filter beads by priority
  #[must_use]
  pub fn filter_by_priority(&self, priority: BeadPriority) -> Vec<Bead> {
    self
      .beads
      .iter()
      .filter(|bead| bead.priority == priority)
      .cloned()
      .collect()
  }

  /// Filter beads by type
  #[must_use]
  pub fn filter_by_type(&self, bead_type: BeadType) -> Vec<Bead> {
    self
      .beads
      .iter()
      .filter(|bead| bead.bead_type == bead_type)
      .cloned()
      .collect()
  }

  /// Filter beads by creator
  #[must_use]
  pub fn filter_by_creator(&self, creator_id: UserId) -> Vec<Bead> {
    self
      .beads
      .iter()
      .filter(|bead| bead.created_by == Some(creator_id))
      .cloned()
      .collect()
  }

  /// Sort beads by priority (high to low)
  #[must_use]
  pub fn sort_by_priority(&self) -> Vec<Bead> {
    self
      .beads
      .iter()
      .cloned()
      .sorted_by_key(|a| a.priority.value())
      .rev()
      .collect()
  }

  /// Sort beads by creation date (newest first)
  #[must_use]
  pub fn sort_by_created_date(&self) -> Vec<Bead> {
    self
      .beads
      .iter()
      .cloned()
      .sorted_by_key(|a| a.created_at)
      .rev()
      .collect()
  }

  /// Get bead by ID
  #[must_use]
  pub fn get_bead(&self, bead_id: BeadId) -> Option<Bead> {
    self.beads.iter().find(|bead| bead.id == bead_id).cloned()
  }

  /// Get all beads
  #[must_use]
  pub fn get_all_beads(&self) -> Vec<Bead> {
    self.beads.iter().cloned().collect()
  }

  /// Get bead statistics
  #[must_use]
  pub fn get_statistics(&self) -> Statistics {
    let total = self.beads.len();
    let status_counts = self.beads.iter().fold(HashMap::new(), |mut acc, bead| {
      *acc.entry(bead.status).or_insert(0) += 1;
      acc
    });

    let priority_counts = self.beads.iter().fold(HashMap::new(), |mut acc, bead| {
      *acc.entry(bead.priority).or_insert(0) += 1;
      acc
    });

    let type_counts = self.beads.iter().fold(HashMap::new(), |mut acc, bead| {
      *acc.entry(bead.bead_type).or_insert(0) += 1;
      acc
    });

    Statistics {
      total,
      status_counts,
      priority_counts,
      type_counts,
    }
  }

  /// Remove a bead
  ///
  /// # Errors
  /// Returns `BeadError::NotFound` when the bead does not exist.
  pub fn remove_bead(self, bead_id: BeadId) -> Result<Self, BeadError> {
    let _bead = self
      .beads
      .iter()
      .find(|b| b.id == bead_id)
      .ok_or_else(|| BeadError::not_found(bead_id))?;

    // Filter out the bead from the vector
    let beads = self
      .beads
      .iter()
      .filter(|b| b.id != bead_id)
      .cloned()
      .collect::<Vector<_>>();

    Ok(Self { beads })
  }
}

/// Statistics for bead collection
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statistics {
  pub total: usize,
  pub status_counts: HashMap<BeadStatus, usize>,
  pub priority_counts: HashMap<BeadPriority, usize>,
  pub type_counts: HashMap<BeadType, usize>,
}

impl Statistics {
  /// Get the count of beads with a specific status
  #[must_use]
  pub fn count_by_status(&self, status: BeadStatus) -> usize {
    self.status_counts.get(&status).copied().unwrap_or(0)
  }

  /// Get the count of beads with a specific priority
  #[must_use]
  pub fn count_by_priority(&self, priority: BeadPriority) -> usize {
    self.priority_counts.get(&priority).copied().unwrap_or(0)
  }

  /// Get the count of beads with a specific type
  #[must_use]
  pub fn count_by_type(&self, bead_type: BeadType) -> usize {
    self.type_counts.get(&bead_type).copied().unwrap_or(0)
  }

  /// Calculate the percentage of beads with a specific status
  #[must_use]
  pub fn percentage_by_status(&self, status: BeadStatus) -> f64 {
    if self.total == 0 {
      0.0
    } else {
      let count_u32 = u32::try_from(self.count_by_status(status)).unwrap_or(u32::MAX);
      let total_u32 = u32::try_from(self.total).unwrap_or(u32::MAX);
      f64::from(count_u32) / f64::from(total_u32) * 100.0
    }
  }
}

/// Functional utilities for bead collections
pub struct BeadOperations;

impl BeadOperations {
  /// Create a new bead from parameters (pure function)
  #[expect(
    clippy::too_many_arguments,
    reason = "Public constructor mirrors domain bead fields"
  )]
  ///
  /// # Errors
  /// Returns `ModelError` when input validation fails.
  pub fn create_bead(
    title: String,
    description: Option<String>,
    status: BeadStatus,
    priority: BeadPriority,
    bead_type: BeadType,
    created_by: Option<UserId>,
  ) -> Result<Bead, ModelError> {
    Bead::new(title, description, status, priority, bead_type, created_by)
  }

  /// Validate bead title (pure function)
  ///
  /// # Errors
  /// Returns a `ModelError` if the title is empty or too long.
  pub fn validate_title(title: &str) -> Result<(), ModelError> {
    if title.trim().is_empty() {
      return Err(ModelError::InvalidTitle(
        "Title cannot be empty".to_string(),
      ));
    }
    if title.len() > Bead::MAX_TITLE_LENGTH {
      return Err(ModelError::InvalidTitle(format!(
        "Title too long: {} characters (max: {})",
        title.len(),
        Bead::MAX_TITLE_LENGTH
      )));
    }
    Ok(())
  }

  /// Check if a status transition is valid (pure function)
  #[must_use]
  #[expect(
    clippy::missing_const_for_fn,
    reason = "Kept non-const for API consistency"
  )]
  pub fn is_valid_transition(from: BeadStatus, to: BeadStatus) -> bool {
    from.can_transition_to(to)
  }

  /// Filter beads by multiple criteria using functional pipeline
  #[must_use]
  pub fn filter_beads(
    beads: &[Bead],
    status_filter: Option<BeadStatus>,
    priority_filter: Option<BeadPriority>,
    type_filter: Option<BeadType>,
    creator_filter: Option<UserId>,
  ) -> Vec<&Bead> {
    beads
      .iter()
      .filter(|bead| {
        status_filter.is_none_or(|s| bead.status == s)
          && priority_filter.is_none_or(|p| bead.priority == p)
          && type_filter.is_none_or(|t| bead.bead_type == t)
          && creator_filter.is_none_or(|c| bead.created_by == Some(c))
      })
      .collect()
  }

  /// Sort beads by multiple criteria using functional pipeline
  #[must_use]
  pub fn sort_beads(
    beads: &[Bead],
    by_priority: bool,
    by_date: bool,
    ascending: bool,
  ) -> Vec<&Bead> {
    let mut sorted: Vec<_> = beads.iter().collect();

    if by_priority && by_date {
      if ascending {
        sorted.sort_by(|a, b| {
          (a.priority.value(), a.created_at).cmp(&(b.priority.value(), b.created_at))
        });
      } else {
        sorted.sort_by(|a, b| {
          (b.priority.value(), b.created_at).cmp(&(a.priority.value(), a.created_at))
        });
      }
    } else if by_priority {
      if ascending {
        sorted.sort_by(|a, b| a.priority.value().cmp(&b.priority.value()));
      } else {
        sorted.sort_by(|a, b| b.priority.value().cmp(&a.priority.value()));
      }
    } else if by_date {
      if ascending {
        sorted.sort_by(|a, b| a.created_at.cmp(&b.created_at));
      } else {
        sorted.sort_by(|a, b| b.created_at.cmp(&a.created_at));
      }
    }

    sorted
  }

  /// Group beads by a criterion using functional pipeline
  #[must_use]
  pub fn group_beads_by<'a, K>(
    beads: &'a [Bead],
    key_fn: impl Fn(&'a Bead) -> K,
  ) -> HashMap<K, Vec<&'a Bead>>
  where
    K: std::hash::Hash + Eq,
  {
    beads.iter().fold(HashMap::new(), |mut acc, bead| {
      let key = key_fn(bead);
      acc.entry(key).or_insert_with(Vec::new).push(bead);
      acc
    })
  }

  /// Calculate statistics for a bead collection (pure function)
  #[must_use]
  pub fn calculate_statistics(beads: &[Bead]) -> Statistics {
    Statistics::from_collection(beads)
  }
}

impl Statistics {
  /// Create statistics from a bead collection
  fn from_collection(beads: &[Bead]) -> Self {
    let total = beads.len();
    let status_counts = beads.iter().fold(HashMap::new(), |mut acc, bead| {
      *acc.entry(bead.status).or_insert(0) += 1;
      acc
    });

    let priority_counts = beads.iter().fold(HashMap::new(), |mut acc, bead| {
      *acc.entry(bead.priority).or_insert(0) += 1;
      acc
    });

    let type_counts = beads.iter().fold(HashMap::new(), |mut acc, bead| {
      *acc.entry(bead.bead_type).or_insert(0) += 1;
      acc
    });

    Self {
      total,
      status_counts,
      priority_counts,
      type_counts,
    }
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::redundant_clone)]

  use super::*;
  use crate::domain::types::*;

  #[test]
  fn test_bead_state_new_empty() {
    let state = BeadState::new();
    assert_eq!(state.get_all_beads().len(), 0);
  }

  #[test]
  fn test_bead_state_add_bead() {
    let bead = Bead::new(
      "Test".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::MEDIUM,
      BeadType::Feature,
      None,
    )
    .unwrap();

    let state = BeadState::new().add_bead(bead.clone()).unwrap();
    assert_eq!(state.get_all_beads().len(), 1);
    assert_eq!(state.get_bead(bead.id).unwrap().title, "Test");
  }

  #[test]
  fn test_bead_state_duplicate_id() {
    let bead1 = Bead::new(
      "Test1".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::MEDIUM,
      BeadType::Feature,
      None,
    )
    .unwrap();

    // Create a second bead with the same ID
    let bead2 = Bead {
      id: bead1.id,
      title: "Test2".to_string(),
      description: None,
      status: BeadStatus::Open,
      priority: BeadPriority::MEDIUM,
      bead_type: BeadType::Feature,
      created_by: Some(UserId::new()),
      created_at: bead1.created_at,
      updated_at: bead1.created_at,
    };

    let state = BeadState::new().add_bead(bead1.clone()).unwrap();
    let result = state.add_bead(bead2);
    assert!(result.is_err());
  }

  #[test]
  fn test_bead_state_update_status() {
    let bead = Bead::new(
      "Test".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::MEDIUM,
      BeadType::Feature,
      None,
    )
    .unwrap();

    let state = BeadState::new().add_bead(bead.clone()).unwrap();
    let updated_state = state
      .update_status(bead.id, BeadStatus::InProgress)
      .unwrap();

    let updated_bead = updated_state.get_bead(bead.id).unwrap();
    assert_eq!(updated_bead.status, BeadStatus::InProgress);
  }

  #[test]
  fn test_bead_state_invalid_transition() {
    let bead = Bead::new(
      "Test".to_string(),
      None,
      BeadStatus::Closed,
      BeadPriority::MEDIUM,
      BeadType::Feature,
      None,
    )
    .unwrap();

    let state = BeadState::new().add_bead(bead.clone()).unwrap();
    let result = state.update_status(bead.id, BeadStatus::InProgress);
    assert!(result.is_err());
  }

  #[test]
  fn test_bead_state_filter_operations() {
    let bead1 = Bead::new(
      "Test1".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::HIGH,
      BeadType::Feature,
      None,
    )
    .unwrap();

    let bead2 = Bead::new(
      "Test2".to_string(),
      None,
      BeadStatus::InProgress,
      BeadPriority::MEDIUM,
      BeadType::Bugfix,
      None,
    )
    .unwrap();

    let state = BeadState::new()
      .add_bead(bead1.clone())
      .unwrap()
      .add_bead(bead2.clone())
      .unwrap();

    assert_eq!(state.filter_by_status(BeadStatus::Open).len(), 1);
    assert_eq!(state.filter_by_priority(BeadPriority::HIGH).len(), 1);
    assert_eq!(state.filter_by_type(BeadType::Feature).len(), 1);
  }

  #[test]
  fn test_bead_statistics() {
    let bead1 = Bead::new(
      "Test1".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::HIGH,
      BeadType::Feature,
      None,
    )
    .unwrap();

    let bead2 = Bead::new(
      "Test2".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::MEDIUM,
      BeadType::Bugfix,
      None,
    )
    .unwrap();

    let state = BeadState::new()
      .add_bead(bead1.clone())
      .unwrap()
      .add_bead(bead2.clone())
      .unwrap();

    let bead_stats = state.get_statistics();
    assert_eq!(bead_stats.total, 2);
    assert_eq!(bead_stats.count_by_status(BeadStatus::Open), 2);
    assert_eq!(bead_stats.count_by_priority(BeadPriority::HIGH), 1);
    assert_eq!(bead_stats.count_by_type(BeadType::Feature), 1);
  }

  #[test]
  fn test_bead_operations_filter() {
    let bead1 = Bead::new(
      "Test1".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::HIGH,
      BeadType::Feature,
      None,
    )
    .unwrap();

    let bead2 = Bead::new(
      "Test2".to_string(),
      None,
      BeadStatus::InProgress,
      BeadPriority::MEDIUM,
      BeadType::Bugfix,
      None,
    )
    .unwrap();

    let test_beads = vec![bead1, bead2];
    let filtered =
      BeadOperations::filter_beads(&test_beads, Some(BeadStatus::Open), None, None, None);

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].title, "Test1");
  }

  #[test]
  fn test_bead_operations_sort() {
    let bead1 = Bead::new(
      "Test1".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::HIGH,
      BeadType::Feature,
      None,
    )
    .unwrap();

    let bead2 = Bead::new(
      "Test2".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::LOW,
      BeadType::Bugfix,
      None,
    )
    .unwrap();

    let test_beads = vec![bead1, bead2];
    let sorted = BeadOperations::sort_beads(&test_beads, true, false, false);

    assert_eq!(sorted.len(), 2);
    assert_eq!(sorted[0].priority, BeadPriority::LOW);
    assert_eq!(sorted[1].priority, BeadPriority::HIGH);
  }

  #[test]
  fn test_bead_operations_group() {
    let bead1 = Bead::new(
      "Test1".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::HIGH,
      BeadType::Feature,
      None,
    )
    .unwrap();

    let bead2 = Bead::new(
      "Test2".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::HIGH,
      BeadType::Bugfix,
      None,
    )
    .unwrap();

    let bead3 = Bead::new(
      "Test3".to_string(),
      None,
      BeadStatus::InProgress,
      BeadPriority::MEDIUM,
      BeadType::Feature,
      None,
    )
    .unwrap();

    let test_beads = vec![bead1, bead2, bead3];
    let grouped = BeadOperations::group_beads_by(&test_beads, |bead| bead.status);

    assert_eq!(grouped[&BeadStatus::Open].len(), 2);
    assert_eq!(grouped[&BeadStatus::InProgress].len(), 1);
  }

  #[test]
  fn test_bead_operations_calculate_statistics() {
    let bead1 = Bead::new(
      "Test1".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::HIGH,
      BeadType::Feature,
      None,
    )
    .unwrap();

    let bead2 = Bead::new(
      "Test2".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::MEDIUM,
      BeadType::Bugfix,
      None,
    )
    .unwrap();

    let test_beads = vec![bead1, bead2];
    let bead_stats = BeadOperations::calculate_statistics(&test_beads);

    assert_eq!(bead_stats.total, 2);
    assert_eq!(bead_stats.count_by_status(BeadStatus::Open), 2);
    assert_eq!(bead_stats.count_by_priority(BeadPriority::HIGH), 1);
    assert_eq!(bead_stats.count_by_type(BeadType::Feature), 1);
  }
}
