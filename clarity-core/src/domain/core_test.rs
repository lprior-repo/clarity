#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Tests for the functional core module

use super::core::*;

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
        Some(UserId::new()),
    ).unwrap();

    let state = DomainState::new()
        .beads(Vector::new().push_back(bead));

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
    ).unwrap();

    let bead2 = Bead::new(
        "Closed Bead".to_string(),
        None,
        BeadStatus::Closed,
        BeadPriority::HIGH,
        BeadType::Bugfix,
        Some(UserId::new()),
    ).unwrap();

    let state = DomainState::new()
        .beads(Vector::new().push_back(bead1).push_back(bead2));

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
    ).unwrap();

    let bead2 = Bead::new(
        "".to_string(),
        None,
        BeadStatus::Closed,
        BeadPriority::HIGH,
        BeadType::Bugfix,
        Some(UserId::new()),
    ).unwrap();

    let state = DomainState::new()
        .beads(Vector::new().push_back(bead1).push_back(bead2));

    let stats = state.statistics();
    assert_eq!(stats.total_beads, 2);
    assert_eq!(stats.open_beads, 2);
    assert_eq!(stats.closed_beads, 0);
    assert_eq!(stats.completion_percentage(), 0.0);
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
    let user = User::new(email, "hash".to_string(), UserRole::Admin).unwrap();

    let bead = create_bead(
        "Test Bead".to_string(),
        "Description".to_string(),
        BeadPriority::HIGH,
        BeadType::Feature,
        Some(user_id),
    );

    assert!(bead.is_ok());
    let bead = bead.unwrap();
    assert_eq!(bead.title, "Test Bead");
    assert_eq!(bead.status, BeadStatus::Open);
}

#[test]
fn test_update_bead_priority() {
    let id = BeadId::new();
    let user_id = UserId::new();
    let email = "test@example.com".parse().unwrap();
    let user = User::new(email, "hash".to_string(), UserRole::Admin).unwrap();

    let original_bead = Bead::new(
        "Test Bead".to_string(),
        "Description".to_string(),
        BeadPriority::HIGH,
        BeadType::Feature,
        Some(user_id),
    ).unwrap();

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
    let user = User::new(email, "hash".to_string(), UserRole::Admin).unwrap();

    let bead = Bead::new(
        "Test Bead".to_string(),
        Some("Description".to_string()),
        BeadStatus::Open,
        BeadPriority::HIGH,
        BeadType::Feature,
        Some(user_id),
    ).unwrap();

    // Valid close
    let closed = close_bead(&bead);
    assert!(closed.is_ok());
    assert_eq!(closed.unwrap().status, BeadStatus::Closed);

    // Test with blocked bead (should fail)
    let blocked_bead = Bead::new(
        "Blocked Bead".to_string(),
        Some("Description".to_string()),
        BeadStatus::Blocked,
        BeadPriority::HIGH,
        BeadType::Feature,
        Some(user_id),
    ).unwrap();

    // Manually set status to blocked
    let mut blocked_bead_mut = blocked_bead.clone();
    blocked_bead_mut.status = BeadStatus::Blocked;
    let closed = close_bead(&blocked_bead_mut);
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
    ).unwrap();

    let bead2 = Bead::new(
        "Low Priority".to_string(),
        None,
        BeadStatus::Open,
        BeadPriority::LOW,
        BeadType::Bugfix,
        None,
    ).unwrap();

    let beads = Vector::new().push_back(bead1).push_back(bead2);
    let operations = vec![filter_high_priority, filter_non_blocked];
    let result = process_bead_pipeline(beads, &operations);

    assert_eq!(result.len(), 1);
}

#[test]
fn test_generate_bead_report() {
    let bead1 = create_bead(
        "Test".to_string(),
        Some("".to_string()),
        BeadPriority::MEDIUM,
        BeadType::Feature,
        None,
    ).unwrap();

    let bead2 = create_bead(
        "Test".to_string(),
        Some("".to_string()),
        BeadPriority::HIGH,
        BeadType::Bugfix,
        None,
    ).unwrap();

    let state = DomainState::new()
        .beads(Vector::new().push_back(bead1).push_back(bead2));

    let report = generate_bead_report(state);
    assert!(report.contains("Total Beads: 2"));
    assert!(report.contains("Completion: 0.0%"));
}