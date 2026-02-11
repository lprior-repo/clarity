//! User service tests - focused on core functionality

use clarity_core::auth::{hash_password, validate_password_strength, verify_password};
use clarity_core::domain::{
  models::User,
  types::{Email, UserId, UserRole},
  user_service::*,
  DomainState,
};
use std::str::FromStr;

#[test]
fn test_create_user_success() {
  let state = DomainState::new();
  let email = "test@example.com".to_string();
  let password = "SecureP@ssw0rd!123".to_string();
  let role = UserRole::User;

  let (new_state, user) = create_user(state, email, password, role).unwrap();

  assert_eq!(user.email.as_str(), "test@example.com");
  assert!(user.is_user());
  assert_eq!(new_state.get_all_users().len(), 1);
}

#[test]
fn test_create_user_duplicate_email() {
  let state = DomainState::new();
  let email = "test@example.com".to_string();
  let password = "SecureP@ssw0rd!123".to_string();
  let role = UserRole::User;

  // First user should succeed
  let (state, _) = create_user(state.clone(), email.clone(), password.clone(), role).unwrap();

  // Second user with same email should fail
  let result = create_user(state, email, password, role);
  assert!(matches!(result, Err(UserError::EmailExists(_))));
}

#[test]
fn test_create_user_weak_password() {
  let state = DomainState::new();
  let email = "test@example.com".to_string();
  let password = "weak".to_string(); // Too weak
  let role = UserRole::User;

  let result = create_user(state, email, password, role);
  assert!(matches!(result, Err(UserError::InvalidPassword(_))));
}

#[test]
fn test_get_user_by_id() {
  let state = DomainState::new();
  let email = "test@example.com".to_string();
  let password = "SecureP@ssw0rd!123".to_string();
  let role = UserRole::User;

  let (state, created_user) = create_user(state, email, password, role).unwrap();
  let user_id = created_user.id;

  let (new_state, retrieved_user) = get_user_by_id(state, user_id).unwrap();

  assert!(retrieved_user.is_some());
  assert_eq!(retrieved_user.unwrap().id, user_id);
  assert_eq!(new_state.get_all_users().len(), 1); // State unchanged
}

#[test]
fn test_get_user_by_email() {
  let state = DomainState::new();
  let email = "test@example.com".to_string();
  let password = "SecureP@ssw0rd!123".to_string();
  let role = UserRole::User;

  let (state, created_user) = create_user(state.clone(), email.clone(), password, role).unwrap();

  let (new_state, retrieved_user) = get_user_by_email(new_state, email).unwrap();

  assert!(retrieved_user.is_some());
  assert_eq!(retrieved_user.unwrap().email.as_str(), "test@example.com");
}

#[test]
fn test_update_user_email() {
  let state = DomainState::new();
  let email = "test@example.com".to_string();
  let password = "SecureP@ssw0rd!123".to_string();
  let role = UserRole::User;

  let (state, user) = create_user(state, email.clone(), password, role).unwrap();

  let new_email = "updated@example.com".to_string();
  let (new_state, updated_user) = update_user_email(new_state, user.id, new_email).unwrap();

  assert_eq!(updated_user.email.as_str(), "updated@example.com");
  assert_eq!(new_state.get_all_users().len(), 1);
}

#[test]
fn test_update_user_role() {
  let state = DomainState::new();
  let email = "test@example.com".to_string();
  let password = "SecureP@ssw0rd!123".to_string();
  let role = UserRole::User;

  let (state, user) = create_user(state, email, password, role).unwrap();

  let (new_state, updated_user) = update_user_role(new_state, user.id, UserRole::Admin).unwrap();

  assert!(updated_user.is_admin());
}

#[test]
fn test_update_user_password() {
  let state = DomainState::new();
  let email = "test@example.com".to_string();
  let password = "SecureP@ssw0rd!123".to_string();
  let role = UserRole::User;

  let (state, user) = create_user(state, email, password, role).unwrap();

  let current_password = "SecureP@ssw0rd!123".to_string();
  let new_password = "NewSecureP@ssw0rd!456".to_string();
  let (new_state, updated_user) =
    update_user_password(new_state, user.id, current_password, new_password).unwrap();

  assert_eq!(new_state.get_all_users().len(), 1);
}

#[test]
fn test_delete_user() {
  let state = DomainState::new();
  let email = "test@example.com".to_string();
  let password = "SecureP@ssw0rd!123".to_string();
  let role = UserRole::User;

  let (state, user) = create_user(state, email, password, role).unwrap();

  let (new_state, result) = get_user_by_id(new_state, user.id).unwrap();
  assert!(result.is_some());

  let final_state = delete_user(new_state, user.id).unwrap();

  let (final_state, result) = get_user_by_id(final_state, user.id).unwrap();
  assert!(result.is_none());
  assert_eq!(final_state.get_all_users().len(), 0);
}

#[test]
fn test_get_users_by_role() {
  let state = DomainState::new();
  let email1 = "admin@example.com".to_string();
  let email2 = "user@example.com".to_string();
  let password = "SecureP@ssw0rd!123".to_string();

  // Create admin user
  let (state, _) = create_user(
    state.clone(),
    email1.clone(),
    password.clone(),
    UserRole::Admin,
  )
  .unwrap();

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
  let password = "SecureP@ssw0rd!123".to_string();
  let role = UserRole::User;

  let (state, _) = create_user(state, email, password, role).unwrap();
  let report = generate_user_report(state);

  assert!(report.contains("Total Users: 1"));
  assert!(report.contains("Admin Users: 0"));
  assert!(report.contains("Regular Users: 1"));
}

#[test]
fn test_user_to_summary() {
  let email = Email::new("test@example.com".to_string()).unwrap();
  let password_hash = "hash".to_string();
  let role = UserRole::User;

  let user = User::new(email, password_hash, role).unwrap();
  let summary = user_to_summary(&user);

  assert_eq!(summary.id, user.id);
  assert_eq!(summary.email, user.email);
  assert_eq!(summary.role, user.role);
  assert!(!summary.is_admin());
}

#[test]
fn test_user_exists_by_email() {
  let state = DomainState::new();
  let email = "test@example.com".to_string();
  let password = "SecureP@ssw0rd!123".to_string();
  let role = UserRole::User;

  // Initially, user should not exist
  let (state, exists_before) = user_exists_by_email(state.clone(), email.clone()).unwrap();
  assert!(!exists_before);

  // Create user
  let (state, _) = create_user(state, email, password, role).unwrap();

  // Now user should exist
  let (state, exists_after) = user_exists_by_email(state, email).unwrap();
  assert!(exists_after);
}

#[test]
fn test_get_user_count() {
  let state = DomainState::new();
  let email1 = "user1@example.com".to_string();
  let email2 = "user2@example.com".to_string();
  let password = "SecureP@ssw0rd!123".to_string();
  let role = UserRole::User;

  // Initially count should be 0
  let (state, count_initial) = get_user_count(state).unwrap();
  assert_eq!(count_initial, 0);

  // Create first user
  let (state, _) = create_user(state.clone(), email1, password.clone(), role).unwrap();

  // Count should be 1
  let (state, count_one) = get_user_count(state).unwrap();
  assert_eq!(count_one, 1);

  // Create second user
  let (state, _) = create_user(state, email2, password, role).unwrap();

  // Count should be 2
  let (_, count_two) = get_user_count(state).unwrap();
  assert_eq!(count_two, 2);
}

#[test]
fn test_user_pipeline_operations() {
  let state = DomainState::new();
  let email1 = "admin@example.com".to_string();
  let email2 = "user@example.com".to_string();
  let password = "SecureP@ssw0rd!123".to_string();

  // Create admin user
  let (state, _) = create_user(
    state.clone(),
    email1.clone(),
    password.clone(),
    UserRole::Admin,
  )
  .unwrap();

  // Create regular user
  let (state, _) = create_user(state, email2, password, UserRole::User).unwrap();

  let all_users = state.get_all_users();

  // Test admin filter pipeline
  let admin_users = process_user_pipeline(all_users, &[filter_admin_users]);

  assert_eq!(admin_users.len(), 1);

  // Check that the filtered user is indeed an admin
  let admin_user = &admin_users[0];
  assert_eq!(admin_user.email.as_str(), "admin@example.com");
  assert!(admin_user.is_admin());
}

#[test]
fn test_password_validation() {
  // Test strong password
  let strong_password = "SecureP@ssw0rd!123";
  assert!(validate_password_strength(strong_password).is_ok());

  // Test weak password (too short)
  let short_password = "short";
  assert!(validate_password_strength(short_password).is_err());

  // Test weak password (no lowercase)
  let no_lower = "PASSWORD123!";
  assert!(validate_password_strength(no_lower).is_err());

  // Test weak password (no uppercase)
  let no_upper = "password123!";
  assert!(validate_password_strength(no_upper).is_err());

  // Test weak password (no digit)
  let no_digit = "Password!";
  assert!(validate_password_strength(no_digit).is_err());

  // Test weak password (no special char)
  let no_special = "Password123";
  assert!(validate_password_strength(no_special).is_err());
}

#[test]
fn test_password_hashing_and_verification() {
  let password = "SecureP@ssw0rd!123";

  // Hash password
  let hash = hash_password(password).unwrap();
  assert!(!hash.is_empty());

  // Verify correct password
  let is_valid = verify_password(&hash, password).unwrap();
  assert!(is_valid);

  // Verify incorrect password
  let is_invalid = verify_password(&hash, "wrong-password").unwrap();
  assert!(!is_invalid);
}

#[test]
fn test_email_validation() {
  // Valid emails
  let valid_emails = vec![
    "test@example.com",
    "user.name@domain.co.uk",
    "user+tag@example.com",
  ];

  for email in valid_emails {
    let email = Email::new(email.to_string()).unwrap();
    assert!(!email.as_str().is_empty());
  }

  // Invalid emails
  let invalid_emails = vec!["notanemail", "@example.com", "user@", "user@.com", ""];

  for email in invalid_emails {
    let result = Email::new(email.to_string());
    assert!(result.is_err());
  }
}
