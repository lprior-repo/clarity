#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Core data models with business logic

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::types::{BeadId, BeadPriority, BeadStatus, BeadType, UserId};

/// New bead (without id and timestamps)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBead {
  pub title: String,
  pub description: Option<String>,
  pub status: BeadStatus,
  pub priority: BeadPriority,
  pub bead_type: BeadType,
  pub created_by: Option<UserId>,
}

impl NewBead {
  /// Convert a `NewBead` into a persisted `Bead`.
  ///
  /// # Errors
  /// Returns `ModelError` when bead validation fails.
  pub fn into_bead(self) -> Result<Bead, ModelError> {
    Bead::new(
      self.title,
      self.description,
      self.status,
      self.priority,
      self.bead_type,
      self.created_by,
    )
  }
}

/// Bead (issue/task)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bead {
  pub id: BeadId,
  pub title: String,
  pub description: Option<String>,
  pub status: BeadStatus,
  pub priority: BeadPriority,
  pub bead_type: BeadType,
  pub created_by: Option<UserId>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl Bead {
  pub const MAX_TITLE_LENGTH: usize = 255;

  #[expect(
    clippy::too_many_arguments,
    reason = "Bead constructor mirrors persisted data fields"
  )]
  ///
  /// # Errors
  /// Returns `ModelError` when the title is invalid.
  pub fn new(
    title: String,
    description: Option<String>,
    status: BeadStatus,
    priority: BeadPriority,
    bead_type: BeadType,
    created_by: Option<UserId>,
  ) -> Result<Self, ModelError> {
    Self::validate_title(&title)?;

    let now = Utc::now();
    Ok(Self {
      id: BeadId::new(),
      title,
      description,
      status,
      priority,
      bead_type,
      created_by,
      created_at: now,
      updated_at: now,
    })
  }

  fn validate_title(title: &str) -> Result<(), ModelError> {
    if title.trim().is_empty() {
      return Err(ModelError::InvalidTitle(
        "Title cannot be empty".to_string(),
      ));
    }
    if title.len() > Self::MAX_TITLE_LENGTH {
      return Err(ModelError::InvalidTitle(format!(
        "Title too long: {} characters (max: {})",
        title.len(),
        Self::MAX_TITLE_LENGTH
      )));
    }
    Ok(())
  }

  ///
  /// # Errors
  /// Returns `ModelError` when the new title is invalid.
  pub fn update_title(mut self, title: String) -> Result<Self, ModelError> {
    Self::validate_title(&title)?;
    self.title = title;
    self.updated_at = Utc::now();
    Ok(self)
  }

  ///
  /// # Errors
  /// Returns `ModelError` when the status transition is invalid.
  pub fn transition_to(mut self, new_status: BeadStatus) -> Result<Self, ModelError> {
    if !self.status.can_transition_to(new_status) {
      return Err(ModelError::InvalidTransition {
        from: self.status,
        to: new_status,
      });
    }
    self.status = new_status;
    self.updated_at = Utc::now();
    Ok(self)
  }

  #[must_use]
  pub fn with_priority(mut self, priority: BeadPriority) -> Self {
    self.priority = priority;
    self.updated_at = Utc::now();
    self
  }

  #[must_use]
  pub fn with_description(mut self, description: Option<String>) -> Self {
    self.description = description;
    self.updated_at = Utc::now();
    self
  }

  #[must_use]
  pub const fn is_open(&self) -> bool {
    matches!(self.status, BeadStatus::Open)
  }

  #[must_use]
  pub const fn is_in_progress(&self) -> bool {
    matches!(self.status, BeadStatus::InProgress)
  }

  #[must_use]
  pub const fn is_closed(&self) -> bool {
    matches!(self.status, BeadStatus::Closed)
  }

  #[must_use]
  pub const fn is_blocked(&self) -> bool {
    matches!(self.status, BeadStatus::Blocked)
  }

  #[must_use]
  pub fn can_modify(&self, user_id: &UserId, is_admin: bool) -> bool {
    is_admin || self.created_by == Some(*user_id)
  }
}

/// Model errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
  InvalidTitle(String),
  InvalidTransition { from: BeadStatus, to: BeadStatus },
}

impl std::fmt::Display for ModelError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::InvalidTitle(msg) => write!(f, "Invalid title: {msg}"),
      Self::InvalidTransition { from, to } => {
        write!(f, "Cannot transition from {from} to {to}")
      }
    }
  }
}

impl std::error::Error for ModelError {}

impl From<super::types::ValidationError> for ModelError {
  fn from(err: super::types::ValidationError) -> Self {
    Self::InvalidTitle(err.to_string())
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[allow(clippy::panic)]
#[allow(unused_variables)]
#[allow(unused_imports)]
mod tests {

  use super::*;

  #[test]
  fn test_bead_new() -> Result<(), ModelError> {
    let bead = Bead::new(
      "Test Bead".to_string(),
      Some("Description".to_string()),
      BeadStatus::Open,
      BeadPriority::HIGH,
      BeadType::Feature,
      None,
    )?;

    assert_eq!(bead.title, "Test Bead");
    assert!(bead.is_open());
    Ok(())
  }

  #[test]
  fn test_bead_empty_title() {
    let result = Bead::new(
      String::new(),
      None,
      BeadStatus::Open,
      BeadPriority::MEDIUM,
      BeadType::Feature,
      None,
    );
    assert!(result.is_err());
  }

  #[test]
  fn test_bead_transition_status() -> Result<(), ModelError> {
    let bead = Bead::new(
      "Test".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::MEDIUM,
      BeadType::Feature,
      None,
    )?;

    // transition_to consumes self and returns a new Self - functional pattern
    let bead = bead.transition_to(BeadStatus::InProgress)?;
    assert!(bead.is_in_progress());

    let bead = bead.transition_to(BeadStatus::Closed)?;
    assert!(bead.is_closed());
    Ok(())
  }

  #[test]
  fn test_bead_invalid_transition() -> Result<(), ModelError> {
    let bead = Bead::new(
      "Test".to_string(),
      None,
      BeadStatus::Closed,
      BeadPriority::MEDIUM,
      BeadType::Feature,
      None,
    )?;

    let result = bead.transition_to(BeadStatus::InProgress);
    assert!(result.is_err());
    Ok(())
  }

  #[test]
  fn test_bead_can_modify() -> Result<(), ModelError> {
    let creator_id = UserId::new();
    let bead = Bead::new(
      "Test".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::MEDIUM,
      BeadType::Feature,
      Some(creator_id),
    )?;

    assert!(bead.can_modify(&creator_id, false));

    let other_id = UserId::new();
    assert!(!bead.can_modify(&other_id, false));
    assert!(bead.can_modify(&other_id, true)); // Admin can modify
    Ok(())
  }
}
