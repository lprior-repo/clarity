//! Test session management functionality

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

use clarity_core::session_manager::{Session, SessionId, SessionManager};

#[tokio::test]
async fn test_session_id_creation() {
  let id1 = SessionId::new();
  let id2 = SessionId::new();

  // IDs should be unique
  assert_ne!(id1, id2);

  // IDs should be valid UUIDs
  assert!(SessionId::from_str(id1.as_str()).is_ok());
  assert!(SessionId::from_str(id2.as_str()).is_ok());
}

#[tokio::test]
async fn test_session_creation() {
  let session = Session::new("test_token_123".to_string());

  assert_eq!(session.token, "test_token_123");

  // Session should not be expired when created
  let now = std::time::SystemTime::now();
  assert!(!session.is_expired(now));

  // Session should not be stale when created
  assert!(!session.is_stale(now));
}

#[tokio::test]
async fn test_session_expiration() {
  let session = Session::new("test_token".to_string());
  let now = std::time::SystemTime::now();

  // Fresh session should not be expired
  assert!(!session.is_expired(now));

  // Session with past expiration should be expired
  let mut expired_session = session.clone();
  expired_session.expires_at = now - std::time::Duration::from_secs(1);
  assert!(expired_session.is_expired(now));
}

#[tokio::test]
async fn test_session_renewal() {
  let session = Session::new("test_token".to_string());
  let now = std::time::SystemTime::now();

  // Make session stale
  let mut stale_session = session.clone();
  stale_session.last_activity = now - std::time::Duration::from_secs(10);
  stale_session.expires_at = now - std::time::Duration::from_secs(1);

  let renewed = stale_session.renew(now);

  // ID and token should remain the same
  assert_eq!(renewed.id, stale_session.id);
  assert_eq!(renewed.token, stale_session.token);

  // Expiration and activity should be updated
  assert_eq!(renewed.last_activity, now);
  assert_eq!(
    renewed.expires_at,
    now + std::time::Duration::from_secs(30 * 60)
  );

  // Original session should remain unchanged
  assert_ne!(stale_session.last_activity, now);
  assert_ne!(stale_session.expires_at, renewed.expires_at);
}

#[tokio::test]
async fn test_session_manager_basic() {
  // Test session manager with in-memory storage
  let manager = SessionManager::new();
  let test_token = "test_session_token_456";

  // Validation should fail for non-existent session
  let result = manager.validate_session(test_token).await;
  assert!(matches!(
    result,
    Err(clarity_core::session_manager::SessionError::SessionNotFound)
  ));
}

#[tokio::test]
async fn test_session_age() {
  let session = Session::new("test_token".to_string());
  let now = std::time::SystemTime::now();

  // Test session age calculation
  let age = session.age_seconds(now);
  assert!(age < 5); // Should be just a few seconds old
}
