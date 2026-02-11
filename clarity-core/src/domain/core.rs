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
use super::{Bead, BeadId, BeadPriority, BeadStatus, BeadType, User, UserId};
use rpds::Vector;
use std::collections::HashMap;

/// Core domain error types (pure, thiserror-based)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
  /// Invalid bead status transition
  InvalidStatusTransition { from: BeadStatus, to: BeadStatus },

  /// Bead not found
  BeadNotFound(BeadId),

  /// User not found
  UserNotFound(UserId),

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
      Self::UserNotFound(id) => write!(f, "User {id} not found"),
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

  /// All users in the system (persistent Vector)
  pub users: Vector<User>,

  /// User name to ID mapping (for efficient lookup)
  pub user_names: HashMap<String, UserId>,
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
      users: Vector::new(),
      user_names: HashMap::new(),
    }
  }

  /// Create domain state with initial users
  #[must_use]
  pub fn with_initial_users(users: Vec<User>) -> Self {
    let mut user_names = HashMap::new();

    // Build user name to ID mapping
    for user in &users {
      // Note: User doesn't have name field, so we use email
      user_names.insert(user.email.as_str().to_string(), user.id);
    }

    Self {
      beads: Vector::new(),
      users: users.into_iter().collect(),
      user_names,
    }
  }

  /// Get bead by ID (pure function - returns Option)
  #[must_use]
  pub fn get_bead(&self, id: BeadId) -> Option<&Bead> {
    self.beads.iter().find(|bead| bead.id == id)
  }

  /// Get user by ID (pure function - returns Option)
  #[must_use]
  pub fn get_user(&self, id: UserId) -> Option<&User> {
    self.users.iter().find(|user| user.id == id)
  }

  /// Get user by email (pure function - returns Option)
  #[must_use]
  pub fn get_user_by_email(&self, email: &str) -> Option<&User> {
    self.user_names.get(email).and_then(|&id| self.get_user(id))
  }

  /// Get all beads (pure function - returns new Vector)
  #[must_use]
  pub fn get_all_beads(&self) -> Vector<Bead> {
    self.beads.clone()
  }

  /// Get all users (pure function - returns new Vector)
  #[must_use]
  pub fn get_all_users(&self) -> Vector<User> {
    self.users.clone()
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

  /// Count beads by status (pure function - returns HashMap)
  #[must_use]
  pub fn count_by_status(&self) -> HashMap<BeadStatus, usize> {
    self.beads.iter().fold(HashMap::new(), |mut counts, bead| {
      // Using a mutable fold here for performance,
      // but this could be made fully immutable with iterators
      *counts.entry(bead.status).or_default() += 1;
      counts
    })
  }

  /// Get statistics (pure function - returns DomainStats)
  #[must_use]
  pub fn statistics(&self) -> DomainStats {
    let total = self.beads.len();
    let by_status = self.count_by_status();

    DomainStats {
      total_beads: total,
      open_beads: by_status.get(&BeadStatus::Open).copied().unwrap_or(0),
      in_progress_beads: by_status.get(&BeadStatus::InProgress).copied().unwrap_or(0),
      blocked_beads: by_status.get(&BeadStatus::Blocked).copied().unwrap_or(0),
      deferred_beads: by_status.get(&BeadStatus::Deferred).copied().unwrap_or(0),
      closed_beads: by_status.get(&BeadStatus::Closed).copied().unwrap_or(0),
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
      self.closed_beads as f64 / self.total_beads as f64 * 100.0
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
/// Returns a DomainError if the transition is not allowed
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
/// Returns a DomainError if the tag length is invalid
pub fn validate_tag_length(tag: &str) -> Result<(), DomainError> {
  if tag.is_empty() || tag.len() > 50 {
    return Err(DomainError::InvalidTagLength(tag.to_string()));
  }
  Ok(())
}

/// Create a new bead with validation (pure function)
///
/// # Errors
/// Returns a ModelError if bead creation fails
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
/// Returns a DomainError if the priority downgrade is not allowed
pub fn update_bead_priority(bead: &Bead, new_priority: BeadPriority) -> Result<Bead, DomainError> {
  // Business rule: Cannot downgrade high priority
  if bead.priority == BeadPriority::HIGH && new_priority.value() > bead.priority.value() {
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
/// Returns a DomainError if the bead is blocked or transition is invalid
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
pub fn add_bead(state: DomainState, bead: Bead) -> DomainState {
  DomainState {
    beads: state.beads.push_back(bead),
    users: state.users,
    user_names: state.user_names,
  }
}

/// Set beads for the domain (pure function)
#[must_use]
pub fn with_beads(state: DomainState, beads: Vector<Bead>) -> DomainState {
  DomainState {
    beads,
    users: state.users,
    user_names: state.user_names,
  }
}

/// Update a bead in the domain (pure function)
///
/// # Errors
/// Returns a DomainError if the bead is not found
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

  Ok(DomainState {
    beads: new_beads,
    users: state.users,
    user_names: state.user_names,
  })
}

/// Change bead status with validation (pure function)
///
/// # Errors
/// Returns a DomainError if the bead is not found or transition is invalid
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
  use super::*;
  use crate::domain::UserRole;
  use std::str::FromStr;

  #[test]
  fn test_domain_state_new() {
    let state = DomainState::new();
    assert_eq!(state.beads.len(), 0);
    assert_eq!(state.users.len(), 0);
  }

  #[test]
  fn test_domain_state_with_initial_users() {
    let email1 = "alice@example.com".parse().unwrap();
    let email2 = "bob@example.com".parse().unwrap();
    let users = vec![
      User::new(email1.clone(), "hash".to_string(), UserRole::Admin).unwrap(),
      User::new(email2.clone(), "hash".to_string(), UserRole::User).unwrap(),
    ];

    let state = DomainState::with_initial_users(users);
    assert_eq!(state.users.len(), 2);
    assert!(state.get_user_by_email(email1.as_str()).is_some());
    assert!(state.get_user_by_email(email2.as_str()).is_some());
  }

  #[test]
  fn test_get_bead() {
    let bead_id = BeadId::new();
    let bead = Bead::new(
      "Test Bead".to_string(),
      Some("Description".to_string()),
      BeadStatus::Open,
      BeadPriority::MEDIUM,
      BeadType::Feature,
      Some(bead_id),
    )
    .unwrap();

    let state = DomainState::new().beads(Vector::new().push_back(bead));

    assert!(state.get_bead(bead_id).is_some());
  }

  #[test]
  fn test_filter_by_status() {
    let bead1 = Bead::new(
      "Open Bead".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::MEDIUM,
      BeadType::Feature,
      Some(UserId::new()),
    )
    .unwrap();

    let bead2 = Bead::new(
      "Closed Bead".to_string(),
      None,
      BeadStatus::Closed,
      BeadPriority::HIGH,
      BeadType::Bugfix,
      Some(UserId::new()),
    )
    .unwrap();

    let state = DomainState::new().beads(Vector::new().push_back(bead1).push_back(bead2));

    let open_beads = state.filter_by_status(BeadStatus::Open);
    let closed_beads = state.filter_by_status(BeadStatus::Closed);

    assert_eq!(open_beads.len(), 1);
    assert_eq!(closed_beads.len(), 1);
  }

  #[test]
  fn test_statistics() {
    let bead1 = Bead::new(
      "".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::MEDIUM,
      BeadType::Feature,
      Some(UserId::new()),
    )
    .unwrap();

    let bead2 = Bead::new(
      "".to_string(),
      None,
      BeadStatus::Closed,
      BeadPriority::HIGH,
      BeadType::Bugfix,
      Some(UserId::new()),
    )
    .unwrap();

    let state = DomainState::new().beads(Vector::new().push_back(bead1).push_back(bead2));

    let report_stats = state.statistics();
    assert_eq!(report_stats.total_beads, 2);
    assert_eq!(report_stats.open_beads, 1);
    assert_eq!(report_stats.closed_beads, 1);
    assert_eq!(report_stats.completion_percentage(), 50.0);
  }

  #[test]
  fn test_validate_status_transition() {
    // Valid transitions
    assert!(validate_status_transition(BeadStatus::Open, BeadStatus::InProgress).is_ok());
    assert!(validate_status_transition(BeadStatus::InProgress, BeadStatus::Closed).is_ok());

    // Invalid transitions
    assert!(validate_status_transition(BeadStatus::Closed, BeadStatus::InProgress).is_err());
  }

  #[test]
  fn test_create_bead() {
    let id = BeadId::new();
    let user_id = UserId::new();
    let email = "test@example.com".parse().unwrap();
    let user = User::new(email, "password_hash".to_string(), UserRole::Admin).unwrap();

    let bead = create_bead(
      "Test Bead".to_string(),
      Some("Description".to_string()),
      BeadPriority::HIGH,
      BeadType::Feature,
      Some(user_id),
    )
    .unwrap();

    assert!(bead.is_ok());
    let bead = bead.unwrap();
    assert_eq!(bead.title(), "Test Bead");
    assert_eq!(bead.status(), BeadStatus::Open);
  }

  #[test]
  fn test_update_bead_priority() {
    let id = BeadId::new();
    let user_id = UserId::new();
    let email = "test@example.com".parse().unwrap();
    let user = User::new(email, "password_hash".to_string(), UserRole::Admin).unwrap();

    let original_bead = Bead::new(
      "Test Bead".to_string(),
      Some("Description".to_string()),
      BeadStatus::Open,
      BeadPriority::HIGH,
      BeadType::Feature,
      Some(user_id),
    )
    .unwrap();

    // Valid update (same priority)
    let updated = update_bead_priority(&original_bead, BeadPriority::HIGH);
    assert!(updated.is_ok());

    // Invalid update (high to medium)
    let updated = update_bead_priority(&original_bead, BeadPriority::MEDIUM);
    assert!(updated.is_err());
  }

  #[test]
  fn test_close_bead() {
    let id = BeadId::new();
    let user_id = UserId::new();
    let email = "test@example.com".parse().unwrap();
    let user = User::new(email, "password_hash".to_string(), UserRole::Admin).unwrap();

    let bead = Bead::new(
      "Test Bead".to_string(),
      Some("Description".to_string()),
      BeadStatus::Open,
      BeadPriority::HIGH,
      BeadType::Feature,
      Some(user_id),
    )
    .unwrap();

    // Valid close
    let closed = close_bead(&bead);
    assert!(closed.is_ok());
    assert_eq!(closed.unwrap().status(), BeadStatus::Closed);

    // Test with blocked bead (should fail)
    let blocked_bead = Bead::new(
      "Blocked Bead".to_string(),
      Some("Description".to_string()),
      BeadStatus::Blocked,
      BeadPriority::HIGH,
      BeadType::Feature,
      Some(user_id),
    )
    .unwrap();

    let closed = close_bead(&blocked_bead);
    assert!(closed.is_err());
  }

  #[test]
  fn test_process_bead_pipeline() {
    let bead1 = Bead::new(
      "High Priority".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::HIGH,
      BeadType::Feature,
      None,
    )
    .unwrap();

    let bead2 = Bead::new(
      "Low Priority".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::LOW,
      BeadType::Bugfix,
      None,
    )
    .unwrap();

    let beads = Vector::new().push_back(bead1).push_back(bead2);

    let operations = vec![filter_high_priority, filter_non_blocked];
    let result = process_bead_pipeline(beads, &operations);

    assert_eq!(result.len(), 1);
  }

  #[test]
  fn test_generate_bead_report() {
    let state = DomainState::new().beads(
      Vector::new()
        .push_back(
          create_bead(
            "Test".to_string(),
            Some("".to_string()),
            BeadPriority::MEDIUM,
            BeadType::Feature,
            Some(UserId::new()),
          )
          .unwrap(),
        )
        .push_back(
          create_bead(
            "Test".to_string(),
            Some("".to_string()),
            BeadPriority::HIGH,
            BeadType::Bugfix,
            Some(UserId::new()),
          )
          .unwrap(),
        ),
    );

    let report = generate_bead_report(state);
    assert!(report.contains("Total Beads: 2"));
    assert!(report.contains("Completion: 50.0%"));
  }
}
