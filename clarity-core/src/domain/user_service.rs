#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! User CRUD operations (Functional Core)

use super::{
  models::User,
  types::{Email, UserId, UserRole, ValidationError},
  DomainState,
};
use crate::auth::{hash_password, validate_password_strength, verify_password};
use rpds::Vector;
use std::collections::HashMap;

/// User CRUD errors (domain errors - use thiserror)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserError {
  /// User not found
  UserNotFound(UserId),

  /// Email already exists
  EmailExists(String),

  /// Invalid email format
  InvalidEmail(String),

  /// Invalid password
  InvalidPassword(String),

  /// Insufficient permissions
  InsufficientPermissions,

  /// Invalid role
  InvalidRole(String),

  /// Password update failed
  PasswordUpdateFailed,

  /// User creation failed
  CreationFailed(String),

  /// User update failed
  UpdateFailed(String),

  /// Validation error
  ValidationError(String),
}

impl From<ValidationError> for UserError {
  fn from(err: ValidationError) -> Self {
    match err {
      ValidationError::InvalidEmail(email) => UserError::InvalidEmail(email),
      _ => UserError::ValidationError(err.to_string()),
    }
  }
}

impl std::fmt::Display for UserError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::UserNotFound(id) => write!(f, "User {} not found", id),
      Self::EmailExists(email) => write!(f, "Email {} already exists", email),
      Self::InvalidEmail(email) => write!(f, "Invalid email: {}", email),
      Self::InvalidPassword(msg) => write!(f, "Invalid password: {}", msg),
      Self::InsufficientPermissions => write!(f, "Insufficient permissions"),
      Self::InvalidRole(role) => write!(f, "Invalid role: {}", role),
      Self::PasswordUpdateFailed => write!(f, "Password update failed"),
      Self::CreationFailed(msg) => write!(f, "User creation failed: {}", msg),
      Self::UpdateFailed(msg) => write!(f, "User update failed: {}", msg),
      Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
    }
  }
}

impl std::error::Error for UserError {}

/// User CRUD result type
pub type UserResult<T> = Result<T, UserError>;

// User CRUD operations (functional core - pure functions)

/// Create a new user with validation
pub fn create_user(
  state: DomainState,
  email: String,
  password: String,
  role: UserRole,
) -> UserResult<(DomainState, User)> {
  // Validate email format
  let email = Email::new(email)?;

  // Validate password strength
  validate_password_strength(&password).map_err(|e| UserError::InvalidPassword(e.to_string()))?;

  // Check if email already exists
  if state.get_user_by_email(email.as_str()).is_some() {
    return Err(UserError::EmailExists(email.to_string()));
  }

  // Hash password
  let password_hash = hash_password(&password)
    .map_err(|_| UserError::CreationFailed("Password hashing failed".to_string()))?;

  // Create user (returns Result)
  let user =
    User::new(email, password_hash, role).map_err(|e| UserError::CreationFailed(e.to_string()))?;

  // Add user to state (pure function)
  let new_state = add_user(state, user.clone());

  Ok((new_state, user))
}

/// Get user by ID
pub fn get_user_by_id(
  state: DomainState,
  user_id: UserId,
) -> UserResult<(DomainState, Option<User>)> {
  let user = state.get_user(user_id).cloned();
  Ok((state, user))
}

/// Get user by email
pub fn get_user_by_email(
  state: DomainState,
  email: String,
) -> UserResult<(DomainState, Option<User>)> {
  let email = Email::new(email)?;

  let user = state.get_user_by_email(email.as_str()).cloned();
  Ok((state, user))
}

/// Get all users
pub fn get_all_users(state: DomainState) -> (DomainState, Vec<User>) {
  let users = state.get_all_users().iter().cloned().collect();
  (state, users)
}

/// Update user email
pub fn update_user_email(
  state: DomainState,
  user_id: UserId,
  new_email: String,
) -> UserResult<(DomainState, User)> {
  // Validate new email
  let new_email = Email::new(new_email)?;

  // Check if new email already exists (excluding current user)
  if let Some(existing_user) = state.get_user_by_email(new_email.as_str()) {
    if existing_user.id != user_id {
      return Err(UserError::EmailExists(new_email.to_string()));
    }
  }

  // Get current user
  let current_user = state
    .get_user(user_id)
    .ok_or(UserError::UserNotFound(user_id))?;

  // Create updated user (immutability)
  let updated_user = User {
    id: current_user.id,
    email: new_email,
    password_hash: current_user.password_hash.clone(),
    role: current_user.role,
    created_at: current_user.created_at,
    updated_at: chrono::Utc::now(),
  };

  // Update state
  let new_state = update_user(state, updated_user.clone());

  Ok((new_state, updated_user))
}

/// Update user role
pub fn update_user_role(
  state: DomainState,
  user_id: UserId,
  new_role: UserRole,
) -> UserResult<(DomainState, User)> {
  // Get current user
  let current_user = state
    .get_user(user_id)
    .ok_or(UserError::UserNotFound(user_id))?;

  // Validate role
  match new_role {
    UserRole::Admin | UserRole::User => (), // Valid
  }

  // Create updated user (immutability)
  let updated_user = User {
    id: current_user.id,
    email: current_user.email.clone(),
    password_hash: current_user.password_hash.clone(),
    role: new_role,
    created_at: current_user.created_at,
    updated_at: chrono::Utc::now(),
  };

  // Update state
  let new_state = update_user(state, updated_user.clone());

  Ok((new_state, updated_user))
}

/// Update user password
pub fn update_user_password(
  state: DomainState,
  user_id: UserId,
  current_password: String,
  new_password: String,
) -> UserResult<(DomainState, User)> {
  // Get current user
  let current_user = state
    .get_user(user_id)
    .ok_or(UserError::UserNotFound(user_id))?;

  // Verify current password
  verify_password(&current_user.password_hash, &current_password)
    .map_err(|_| UserError::InvalidPassword("Current password is incorrect".to_string()))?;

  // Validate new password strength
  validate_password_strength(&new_password)
    .map_err(|e| UserError::InvalidPassword(e.to_string()))?;

  // Hash new password
  let new_hash = hash_password(&new_password).map_err(|_| UserError::PasswordUpdateFailed)?;

  // Create updated user (immutability)
  let updated_user = User {
    id: current_user.id,
    email: current_user.email.clone(),
    password_hash: new_hash,
    role: current_user.role,
    created_at: current_user.created_at,
    updated_at: chrono::Utc::now(),
  };

  // Update state
  let new_state = update_user(state, updated_user.clone());

  Ok((new_state, updated_user))
}

/// Delete a user
pub fn delete_user(state: DomainState, user_id: UserId) -> UserResult<DomainState> {
  // Check if user exists
  if state.get_user(user_id).is_none() {
    return Err(UserError::UserNotFound(user_id));
  }

  // Filter out the user (pure function - no mut)
  let new_users: Vector<User> = state
    .users
    .iter()
    .filter(|user| user.id != user_id)
    .cloned()
    .collect();

  // Update user name mapping
  let new_user_names = new_users
    .iter()
    .map(|user| (user.email.as_str().to_string(), user.id))
    .collect();

  Ok(DomainState {
    beads: state.beads.clone(),
    users: new_users,
    user_names: new_user_names,
  })
}

/// Check if user exists by email
pub fn user_exists_by_email(state: DomainState, email: String) -> UserResult<(DomainState, bool)> {
  let email = Email::new(email)?;

  let exists = state.get_user_by_email(email.as_str()).is_some();
  Ok((state, exists))
}

/// Get users by role
pub fn get_users_by_role(state: DomainState, role: UserRole) -> (DomainState, Vec<User>) {
  let users = state
    .users
    .iter()
    .filter(|user| user.role == role)
    .cloned()
    .collect();

  (state, users)
}

/// Get user count
pub fn get_user_count(state: DomainState) -> (DomainState, usize) {
  (state.clone(), state.users.len())
}

// State transition functions (pure - return new state)

/// Add a user to the domain
pub fn add_user(state: DomainState, user: User) -> DomainState {
  // Update user name mapping
  let mut new_user_names = state.user_names.clone();
  new_user_names.insert(user.email.as_str().to_string(), user.id);

  DomainState {
    beads: state.beads.clone(),
    users: state.users.push_back(user),
    user_names: new_user_names,
  }
}

/// Update a user in the domain
pub fn update_user(state: DomainState, updated_user: User) -> DomainState {
  // Update users vector
  let new_users: Vector<User> = state
    .users
    .iter()
    .map(|user| {
      if user.id == updated_user.id {
        updated_user.clone()
      } else {
        user.clone()
      }
    })
    .collect();

  // Update user name mapping
  let mut new_user_names: HashMap<String, UserId> = HashMap::new();
  for user in &new_users {
    new_user_names.insert(user.email.as_str().to_string(), user.id);
  }

  DomainState {
    beads: state.beads.clone(),
    users: new_users,
    user_names: new_user_names,
  }
}

/// Set users for the domain (pure function)
pub fn with_users(state: DomainState, users: Vector<User>) -> DomainState {
  // Build user name to ID mapping
  let mut user_names = HashMap::new();
  for user in &users {
    user_names.insert(user.email.as_str().to_string(), user.id);
  }

  DomainState {
    beads: state.beads.clone(),
    users,
    user_names,
  }
}

// Pipeline functions (functional composition example)

/// Process users through a pipeline of operations
pub fn process_user_pipeline(
  users: Vector<User>,
  operations: &[fn(&User) -> Option<User>],
) -> Vector<User> {
  operations.iter().fold(users, |current_users, &operation| {
    current_users
      .iter()
      .filter_map(|user| operation(user))
      .collect()
  })
}

/// Example pipeline operation: Filter admin users
pub fn filter_admin_users(user: &User) -> Option<User> {
  if user.is_admin() {
    Some(user.clone())
  } else {
    None
  }
}

/// Example pipeline operation: Filter users created after a date
pub fn filter_users_after_date(user: &User, date: chrono::DateTime<chrono::Utc>) -> Option<User> {
  if user.created_at > date {
    Some(user.clone())
  } else {
    None
  }
}

/// Generate user report (pure function)
pub fn generate_user_report(state: DomainState) -> String {
  let all_users = get_all_users(state).1;
  let admin_count = all_users.iter().filter(|u| u.is_admin()).count();
  let user_count = all_users.iter().filter(|u| u.is_user()).count();

  format!(
    "User Report\n\
        ============\n\
        Total Users: {}\n\
        Admin Users: {}\n\
        Regular Users: {}\n\
        System Status: {}",
    all_users.len(),
    admin_count,
    user_count,
    if all_users.is_empty() {
      "No users"
    } else {
      "Active"
    }
  )
}

/// Convert user to summary (for security purposes - exclude sensitive data)
pub fn user_to_summary(user: &User) -> UserSummary {
  UserSummary {
    id: user.id,
    email: user.email.clone(),
    role: user.role,
    created_at: user.created_at,
    is_admin: user.is_admin(),
  }
}

/// User summary for display (excludes password hash)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserSummary {
  pub id: UserId,
  pub email: Email,
  pub role: UserRole,
  pub created_at: chrono::DateTime<chrono::Utc>,
  pub is_admin: bool,
}

impl UserSummary {
  pub const fn is_admin(&self) -> bool {
    self.is_admin
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::str::FromStr;

  #[test]
  fn test_create_user_success() {
    let state = DomainState::new();
    let email = "test@example.com".to_string();
    let password = "SecureP@ssw0rd!123";
    let role = UserRole::User;

    let (new_state, user) = create_user(state, email, password, role).unwrap();

    assert_eq!(user.email.as_str(), "test@example.com");
    assert!(user.is_user());
    assert_eq!(new_state.users.len(), 1);
  }

  #[test]
  fn test_create_user_duplicate_email() {
    let state = DomainState::new();
    let email = "test@example.com".to_string();
    let password = "SecureP@ssw0rd!123";
    let role = UserRole::User;

    // First user should succeed
    let (state, _) = create_user(state.clone(), email.clone(), password, role).unwrap();

    // Second user with same email should fail
    let result = create_user(state, email, password, role);
    assert!(matches!(result, Err(UserError::EmailExists(_))));
  }

  #[test]
  fn test_create_user_weak_password() {
    let state = DomainState::new();
    let email = "test@example.com".to_string();
    let password = "weak"; // Too weak
    let role = UserRole::User;

    let result = create_user(state, email, password, role);
    assert!(matches!(result, Err(UserError::InvalidPassword(_))));
  }

  #[test]
  fn test_get_user_by_id() {
    let state = DomainState::new();
    let email = "test@example.com".to_string();
    let password = "SecureP@ssw0rd!123";
    let role = UserRole::User;

    let (state, created_user) = create_user(state, email, password, role).unwrap();
    let user_id = created_user.id;

    let (new_state, retrieved_user) = get_user_by_id(state, user_id).unwrap();

    assert!(retrieved_user.is_some());
    assert_eq!(retrieved_user.unwrap().id, user_id);
    assert_eq!(new_state.users.len(), 1); // State unchanged
  }

  #[test]
  fn test_get_user_by_email() {
    let state = DomainState::new();
    let email = "test@example.com".to_string();
    let password = "SecureP@ssw0rd!123";
    let role = UserRole::User;

    let (state, created_user) = create_user(state, email.clone(), password, role).unwrap();

    let (new_state, retrieved_user) = get_user_by_email(new_state, email).unwrap();

    assert!(retrieved_user.is_some());
    assert_eq!(retrieved_user.unwrap().email.as_str(), "test@example.com");
  }

  #[test]
  fn test_update_user_email() {
    let state = DomainState::new();
    let email = "test@example.com".to_string();
    let password = "SecureP@ssw0rd!123";
    let role = UserRole::User;

    let (state, user) = create_user(state, email.clone(), password, role).unwrap();

    let new_email = "updated@example.com".to_string();
    let (new_state, updated_user) = update_user_email(new_state, user.id, new_email).unwrap();

    assert_eq!(updated_user.email.as_str(), "updated@example.com");
    assert_eq!(new_state.users.len(), 1);
  }

  #[test]
  fn test_update_user_role() {
    let state = DomainState::new();
    let email = "test@example.com".to_string();
    let password = "SecureP@ssw0rd!123";
    let role = UserRole::User;

    let (state, user) = create_user(state, email, password, role).unwrap();

    let (new_state, updated_user) = update_user_role(new_state, user.id, UserRole::Admin).unwrap();

    assert!(updated_user.is_admin());
  }

  #[test]
  fn test_update_user_password() {
    let state = DomainState::new();
    let email = "test@example.com".to_string();
    let password = "SecureP@ssw0rd!123";
    let role = UserRole::User;

    let (state, user) = create_user(state, email, password, role).unwrap();

    let current_password = "SecureP@ssw0rd!123";
    let new_password = "NewSecureP@ssw0rd!456";
    let (new_state, updated_user) =
      update_user_password(new_state, user.id, current_password, new_password).unwrap();

    assert_eq!(new_state.users.len(), 1);
    // Cannot directly verify hash, but we can verify the user exists
  }

  #[test]
  fn test_delete_user() {
    let state = DomainState::new();
    let email = "test@example.com".to_string();
    let password = "SecureP@ssw0rd!123";
    let role = UserRole::User;

    let (state, user) = create_user(state, email, password, role).unwrap();

    let (new_state, result) = get_user_by_id(new_state, user.id).unwrap();
    assert!(result.is_some());

    let final_state = delete_user(new_state, user.id).unwrap();

    let (final_state, result) = get_user_by_id(final_state, user.id).unwrap();
    assert!(result.is_none());
    assert_eq!(final_state.users.len(), 0);
  }

  #[test]
  fn test_get_users_by_role() {
    let state = DomainState::new();
    let email1 = "admin@example.com".to_string();
    let email2 = "user@example.com".to_string();
    let password = "SecureP@ssw0rd!123";

    // Create admin user
    let (state, _) = create_user(state.clone(), email1, password, UserRole::Admin).unwrap();

    // Create regular user
    let (state, _) = create_user(state, email2, password, UserRole::User).unwrap();

    let (_, admin_users) = get_users_by_role(state, UserRole::Admin);
    let (_, user_users) = get_users_by_role(state, UserRole::User);

    assert_eq!(admin_users.len(), 1);
    assert_eq!(user_users.len(), 1);
  }

  #[test]
  fn test_generate_user_report() {
    let state = DomainState::new();
    let email = "test@example.com".to_string();
    let password = "SecureP@ssw0rd!123";
    let role = UserRole::User;

    let (state, _) = create_user(state, email, password, role).unwrap();
    let report = generate_user_report(state);

    assert!(report.contains("Total Users: 1"));
    assert!(report.contains("Admin Users: 0"));
    assert!(report.contains("Regular Users: 1"));
  }

  #[test]
  fn test_user_to_summary() {
    let email = "test@example.com".to_string();
    let password_hash = "hash".to_string();
    let role = UserRole::User;

    let user = User::new(email, password_hash, role).unwrap();
    let summary = user_to_summary(&user);

    assert_eq!(summary.id, user.id);
    assert_eq!(summary.email, user.email);
    assert_eq!(summary.role, user.role);
    assert!(!summary.is_admin());
  }
}
