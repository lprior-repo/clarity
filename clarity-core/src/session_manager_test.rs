#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Test session manager functionality

use crate::domain::types::UserId;
use crate::session_manager::{Session, SessionCore, SessionId, SessionManager};

#[tokio::test]
async fn test_session_id_creation() {
  let id1 = SessionId::new();
  let id2 = SessionId::new();

  assert_ne!(id1, id2);
  assert!(SessionId::from_str(id1.as_str()).is_ok());
}

#[tokio::test]
async fn test_session_creation() {
  let user_id = UserId::new();
  let session = Session::new(user_id, "test_token".to_string());

  assert_eq!(session.user_id, user_id);
  assert_eq!(session.token, "test_token");
  assert!(!session.is_expired(std::time::SystemTime::now()));
}

#[tokio::test]
async fn test_session_expiration() {
  let user_id = UserId::new();
  let session = Session::new(user_id, "test_token".to_string());

  // Fresh session should not be expired
  assert!(!session.is_expired(std::time::SystemTime::now()));

  // Session with past expiration should be expired
  let expired_session = Session {
    expires_at: std::time::SystemTime::now() - std::time::Duration::from_secs(1),
    ..session
  };
  assert!(expired_session.is_expired(std::time::SystemTime::now()));
}

#[tokio::test]
async fn test_session_renewal() {
  let user_id = UserId::new();
  let mut session = Session::new(user_id, "test_token".to_string());

  let now = std::time::SystemTime::now();

  // Change some properties to verify renewal
  session.expires_at = now - std::time::Duration::from_secs(1);
  session.last_activity = now - std::time::Duration::from_secs(10);

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
}

#[tokio::test]
async fn test_session_manager_in_memory() {
  let manager = SessionManager::new();
  let token = "test_token";

  // Should fail because there's no database
  let result = manager.authenticate(UserId::new(), "password").await;
  assert!(matches!(
    result,
    Err(crate::session_manager::SessionError::UserNotFound)
  ));

  // Validation should fail for non-existent session
  let result = manager.validate_session(token).await;
  assert!(matches!(
    result,
    Err(crate::session_manager::SessionError::SessionNotFound)
  ));

  // Termination should not fail for non-existent session
  let result = manager.terminate_session(token).await;
  assert!(result.is_ok());
}
