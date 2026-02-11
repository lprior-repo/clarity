//! Test session management functionality

use clarity_core::domain::types::UserId;
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
  let user_id = UserId::new();
  let session = Session::new(user_id, "test_token_123".to_string());

  assert_eq!(session.user_id, user_id);
  assert_eq!(session.token, "test_token_123");

  // Session should not be expired when created
  let now = std::time::SystemTime::now();
  assert!(!session.is_expired(now));

  // Session should not be stale when created
  assert!(!session.is_stale(now));
}

#[tokio::test]
async fn test_session_expiration() {
  let user_id = UserId::new();
  let session = Session::new(user_id, "test_token".to_string());
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
  let user_id = UserId::new();
  let mut session = Session::new(user_id, "test_token".to_string());
  let now = std::time::SystemTime::now();

  // Make session stale
  session.last_activity = now - std::time::Duration::from_secs(10);
  session.expires_at = now - std::time::Duration::from_secs(1);

  let renewed = session.renew(now);

  // ID and token should remain the same
  assert_eq!(renewed.id, session.id);
  assert_eq!(renewed.token, session.token);

  // Expiration and activity should be updated
  assert_eq!(renewed.last_activity, now);
  assert_eq!(
    renewed.expires_at,
    now + std::time::Duration::from_secs(30 * 60)
  );

  // Original session should remain unchanged
  assert_ne!(session.last_activity, now);
  assert_ne!(session.expires_at, renewed.expires_at);
}

#[tokio::test]
async fn test_session_lifetime() {
  let user_id = UserId::new();
  let session = Session::new(user_id, "test_token".to_string());
  let now = std::time::SystemTime::now();

  // Test session lifetime calculation
  let lifetime = crate::session_manager::SessionUtils::session_lifetime(&session, now);
  assert!(lifetime > std::time::Duration::from_secs(0));
  assert!(lifetime <= std::time::Duration::from_secs(30 * 60));
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

  // Termination should not fail for non-existent session
  let result = manager.terminate_session(test_token).await;
  assert!(result.is_ok());
}
