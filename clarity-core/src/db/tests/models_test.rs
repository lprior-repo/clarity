#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Tests for database models
//!
//! These tests verify that domain types are validated correctly

use crate::db::{BeadId, BeadPriority, DbError, Email, UserId};

#[test]
fn test_email_valid() {
    let valid_emails = vec![
        "user@example.com",
        "test.user@domain.co.uk",
        "admin@localhost.localdomain",
    ];

    for email in valid_emails {
        let result = Email::new(email.to_string());
        assert!(
            result.is_ok(),
            "Email '{}' should be valid, got error: {:?}",
            email,
            result
        );
        assert_eq!(result.unwrap().as_str(), email);
    }
}

#[test]
fn test_email_invalid() {
    let invalid_emails = vec!["notanemail", "@example.com", "user@", "user@.com"];

    for email in invalid_emails {
        let result = Email::new(email.to_string());
        assert!(
            result.is_err(),
            "Email '{}' should be invalid, but got success: {:?}",
            email,
            result
        );

        match result {
            Err(DbError::InvalidEmail(_)) => {}
            _ => panic!("Expected InvalidEmail error for '{}'", email),
        }
    }
}

#[test]
fn test_user_id_from_str_valid() {
    let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
    let result = UserId::from_str(uuid_str);

    match result {
        Ok(user_id) => {
            assert_eq!(user_id.as_uuid().to_string(), uuid_str);
        }
        Err(e) => {
            panic!("Expected valid UserId, got error: {:?}", e);
        }
    }
}

#[test]
fn test_user_id_from_str_invalid() {
    let invalid_uuid = "not-a-uuid";
    let result = UserId::from_str(invalid_uuid);

    assert!(result.is_err());
    match result {
        Err(DbError::InvalidUuid(_)) => {}
        _ => panic!("Expected InvalidUuid error"),
    }
}

#[test]
fn test_bead_id_from_str_valid() {
    let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
    let result = BeadId::from_str(uuid_str);

    match result {
        Ok(bead_id) => {
            assert_eq!(bead_id.as_uuid().to_string(), uuid_str);
        }
        Err(e) => {
            panic!("Expected valid BeadId, got error: {:?}", e);
        }
    }
}

#[test]
fn test_bead_id_from_str_invalid() {
    let invalid_uuid = "not-a-uuid";
    let result = BeadId::from_str(invalid_uuid);

    assert!(result.is_err());
    match result {
        Err(DbError::InvalidUuid(_)) => {}
        _ => panic!("Expected InvalidUuid error"),
    }
}

#[test]
fn test_bead_priority_valid() {
    let priorities = vec![1, 2, 3];

    for priority in priorities {
        let result = BeadPriority::new(priority);
        assert!(
            result.is_ok(),
            "Priority {} should be valid, got error: {:?}",
            priority,
            result
        );
        assert_eq!(result.unwrap().0, priority);
    }
}

#[test]
fn test_bead_priority_invalid() {
    let invalid_priorities = vec![0, 4, -1, 100];

    for priority in invalid_priorities {
        let result = BeadPriority::new(priority);
        assert!(
            result.is_err(),
            "Priority {} should be invalid, but got success: {:?}",
            priority,
            result
        );

        match result {
            Err(DbError::Validation(_)) => {}
            _ => panic!("Expected Validation error for priority {}", priority),
        }
    }
}

#[test]
fn test_bead_priority_constants() {
    assert_eq!(BeadPriority::HIGH.0, 1);
    assert_eq!(BeadPriority::MEDIUM.0, 2);
    assert_eq!(BeadPriority::LOW.0, 3);
}

#[test]
fn test_user_id_new_generates_unique() {
    let id1 = UserId::new();
    let id2 = UserId::new();

    assert_ne!(id1, id2, "UserIds should be unique");
}

#[test]
fn test_bead_id_new_generates_unique() {
    let id1 = BeadId::new();
    let id2 = BeadId::new();

    assert_ne!(id1, id2, "BeadIds should be unique");
}
