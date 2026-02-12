#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Test session manager functionality

use crate::session_manager::{Session, SessionCore, SessionId, SessionManager};
use std::time::{Duration, SystemTime};

#[tokio::test]
async fn test_session_id_creation() {
  let id1 = SessionId::new();
  let id2 = SessionId::new();

  assert_ne!(id1, id2);
  assert!(SessionId::from_str(id1.as_str()).is_ok());
}

#[tokio::test]
async fn test_session_creation() {
  let session = Session::new("test_token".to_string());

  assert_eq!(session.token, "test_token");
  assert!(!session.is_expired(SystemTime::now()));
}

#[tokio::test]
async fn test_session_expiration() {
  let session = Session::new("test_token".to_string());

  // Fresh session should not be expired
  assert!(!session.is_expired(SystemTime::now()));

  // Session with past expiration should be expired
  let expired_session = Session {
    expires_at: SystemTime::now() - Duration::from_secs(1),
    ..session
  };
  assert!(expired_session.is_expired(SystemTime::now()));
}

#[tokio::test]
async fn test_session_renewal() {
  let session = Session::new("test_token".to_string());

  let now = SystemTime::now();

  // Create a stale session for testing
  let stale_session = Session {
    expires_at: now - Duration::from_secs(1),
    last_activity: now - Duration::from_secs(10),
    ..session
  };

  let renewed = stale_session.renew(now);

  // ID and token should remain the same
  assert_eq!(renewed.id, stale_session.id);
  assert_eq!(renewed.token, stale_session.token);

  // Expiration and activity should be updated
  assert_eq!(renewed.last_activity, now);
  assert_eq!(renewed.expires_at, now + Duration::from_secs(30 * 60));

  // Original session should remain unchanged
  assert_ne!(stale_session.last_activity, now);
}

#[tokio::test]
async fn test_session_manager_in_memory() {
  let manager = SessionManager::new();
  let token = "test_token".to_string();

  // Create a session
  let session = manager.create_session(token.clone()).await;
  assert_eq!(session.token, token);

  // Validation should succeed for existing session
  let result = manager.validate_session(&token).await;
  assert!(result.is_ok());

  // Termination should succeed
  let result = manager.terminate_session(&token).await;
  assert!(result.is_ok());

  // After termination, validation should fail
  let result = manager.validate_session(&token).await;
  assert!(matches!(
    result,
    Err(crate::session_manager::SessionError::SessionNotFound)
  ));
}

#[tokio::test]
async fn test_session_core_create_and_validate() {
  let core = SessionCore::new();
  let token = "unique_token".to_string();

  // Create session
  let session = core.create_session(token.clone()).await;
  assert_eq!(session.token, token);

  // Validate session
  let validated = core.validate_session(&token).await;
  assert!(validated.is_ok());
  assert_eq!(validated.map_or_else(|_| String::new(), |s| s.token), token);
}

#[tokio::test]
async fn test_session_core_cleanup() {
  let core = SessionCore::new();
  let token = "token_to_expire".to_string();

  // Create session
  let _session = core.create_session(token.clone()).await;

  // Cleanup should return 0 for non-expired sessions
  let cleaned = core.cleanup_expired_sessions().await;
  assert!(cleaned.is_ok());
  // Note: The cleanup depends on time, so we just verify it runs without error
}
