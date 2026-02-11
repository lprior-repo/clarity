#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Core data models with business logic

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::types::{BeadId, BeadPriority, BeadStatus, BeadType, Email, UserId, UserRole};

/// User account
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
  pub id: UserId,
  pub email: Email,
  pub password_hash: String,
  pub role: UserRole,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

impl User {
  pub const MAX_EMAIL_LENGTH: usize = 255;

  pub fn new(email: Email, password_hash: String, role: UserRole) -> Result<Self, ModelError> {
    if password_hash.is_empty() {
      return Err(ModelError::InvalidPassword(
        "Password cannot be empty".to_string(),
      ));
    }

    if email.as_str().len() > Self::MAX_EMAIL_LENGTH {
      return Err(ModelError::InvalidEmail(format!(
        "Email too long: {} characters (max: {})",
        email.as_str().len(),
        Self::MAX_EMAIL_LENGTH
      )));
    }

    let now = Utc::now();
    Ok(Self {
      id: UserId::new(),
      email,
      password_hash,
      role,
      created_at: now,
      updated_at: now,
    })
  }

  pub fn with_email(mut self, email: Email) -> Result<Self, ModelError> {
    if email.as_str().len() > Self::MAX_EMAIL_LENGTH {
      return Err(ModelError::InvalidEmail(format!(
        "Email too long: {} characters (max: {})",
        email.as_str().len(),
        Self::MAX_EMAIL_LENGTH
      )));
    }
    self.email = email;
    self.updated_at = Utc::now();
    Ok(self)
  }

  pub fn with_role(mut self, role: UserRole) -> Self {
    self.role = role;
    self.updated_at = Utc::now();
    self
  }

  pub fn update_password(self, new_hash: String) -> Result<Self, ModelError> {
    if new_hash.is_empty() {
      return Err(ModelError::InvalidPassword(
        "Password cannot be empty".to_string(),
      ));
    }
    let mut updated = self;
    updated.password_hash = new_hash;
    updated.updated_at = Utc::now();
    Ok(updated)
  }

  pub const fn is_admin(&self) -> bool {
    matches!(self.role, UserRole::Admin)
  }

  pub const fn is_user(&self) -> bool {
    matches!(self.role, UserRole::User)
  }

  pub fn can_modify(&self, target_user_id: &UserId) -> bool {
    self.is_admin() || self.id == *target_user_id
  }
}

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

  pub fn update_title(mut self, title: String) -> Result<Self, ModelError> {
    Self::validate_title(&title)?;
    self.title = title;
    self.updated_at = Utc::now();
    Ok(self)
  }

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

  pub fn with_priority(mut self, priority: BeadPriority) -> Self {
    self.priority = priority;
    self.updated_at = Utc::now();
    self
  }

  pub fn with_description(mut self, description: Option<String>) -> Self {
    self.description = description;
    self.updated_at = Utc::now();
    self
  }

  pub const fn is_open(&self) -> bool {
    matches!(self.status, BeadStatus::Open)
  }

  pub const fn is_in_progress(&self) -> bool {
    matches!(self.status, BeadStatus::InProgress)
  }

  pub const fn is_closed(&self) -> bool {
    matches!(self.status, BeadStatus::Closed)
  }

  pub const fn is_blocked(&self) -> bool {
    matches!(self.status, BeadStatus::Blocked)
  }

  pub fn can_modify(&self, user_id: &UserId, is_admin: bool) -> bool {
    is_admin || self.created_by == Some(*user_id)
  }
}

/// Model errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
  InvalidEmail(String),
  InvalidPassword(String),
  InvalidTitle(String),
  InvalidTransition { from: BeadStatus, to: BeadStatus },
}

impl std::fmt::Display for ModelError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::InvalidEmail(msg) => write!(f, "Invalid email: {}", msg),
      Self::InvalidPassword(msg) => write!(f, "Invalid password: {}", msg),
      Self::InvalidTitle(msg) => write!(f, "Invalid title: {}", msg),
      Self::InvalidTransition { from, to } => {
        write!(f, "Cannot transition from {} to {}", from, to)
      }
    }
  }
}

impl std::error::Error for ModelError {}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_user_new() {
    let email = Email::new("test@example.com".to_string()).unwrap();
    let user = User::new(email, "hash".to_string(), UserRole::User).unwrap();

    assert_eq!(user.email.as_str(), "test@example.com");
    assert!(user.is_user());
    assert!(!user.is_admin());
  }

  #[test]
  fn test_user_new_admin() {
    let email = Email::new("admin@example.com".to_string()).unwrap();
    let user = User::new(email, "hash".to_string(), UserRole::Admin).unwrap();

    assert!(user.is_admin());
  }

  #[test]
  fn test_user_empty_password() {
    let email = Email::new("test@example.com".to_string()).unwrap();
    let result = User::new(email, "".to_string(), UserRole::User);
    assert!(result.is_err());
  }

  #[test]
  fn test_user_can_modify() {
    let email = Email::new("test@example.com".to_string()).unwrap();
    let user = User::new(email, "hash".to_string(), UserRole::User).unwrap();

    assert!(user.can_modify(&user.id));

    let other_id = UserId::new();
    assert!(!user.can_modify(&other_id));
  }

  #[test]
  fn test_user_admin_can_modify_any() {
    let email = Email::new("admin@example.com".to_string()).unwrap();
    let admin = User::new(email, "hash".to_string(), UserRole::Admin).unwrap();

    let other_id = UserId::new();
    assert!(admin.can_modify(&other_id));
  }

  #[test]
  fn test_bead_new() {
    let bead = Bead::new(
      "Test Bead".to_string(),
      Some("Description".to_string()),
      BeadStatus::Open,
      BeadPriority::HIGH,
      BeadType::Feature,
      None,
    )
    .unwrap();

    assert_eq!(bead.title, "Test Bead");
    assert!(bead.is_open());
  }

  #[test]
  fn test_bead_empty_title() {
    let result = Bead::new(
      "".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::MEDIUM,
      BeadType::Feature,
      None,
    );
    assert!(result.is_err());
  }

  #[test]
  fn test_bead_transition_status() {
    let mut bead = Bead::new(
      "Test".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::MEDIUM,
      BeadType::Feature,
      None,
    )
    .unwrap();

    bead.transition_to(BeadStatus::InProgress).unwrap();
    assert!(bead.is_in_progress());

    bead.transition_to(BeadStatus::Closed).unwrap();
    assert!(bead.is_closed());
  }

  #[test]
  fn test_bead_invalid_transition() {
    let mut bead = Bead::new(
      "Test".to_string(),
      None,
      BeadStatus::Closed,
      BeadPriority::MEDIUM,
      BeadType::Feature,
      None,
    )
    .unwrap();

    let result = bead.transition_to(BeadStatus::InProgress);
    assert!(result.is_err());
  }

  #[test]
  fn test_bead_can_modify() {
    let creator_id = UserId::new();
    let bead = Bead::new(
      "Test".to_string(),
      None,
      BeadStatus::Open,
      BeadPriority::MEDIUM,
      BeadType::Feature,
      Some(creator_id),
    )
    .unwrap();

    assert!(bead.can_modify(&creator_id, false));

    let other_id = UserId::new();
    assert!(!bead.can_modify(&other_id, false));
    assert!(bead.can_modify(&other_id, true)); // Admin can modify
  }
}
