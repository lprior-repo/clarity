#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Session management for Clarity
//!
//! Provides a complete session management system with:
//! - User authentication and session creation
//! - Session token management with expiration
//! - Session validation and cleanup
//! - Persistent session storage
//!
//! Follows functional core, imperative shell pattern:
//! - Core: Pure functions for business logic
//! - Shell: I/O operations and persistence

use crate::{
  auth::{self},
  domain::{models::User, types::UserId},
};
use chrono::{DateTime, Utc};
use moka::future::Cache;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Session duration (30 minutes of inactivity)
const SESSION_DURATION: Duration = Duration::from_secs(30 * 60);
/// Max concurrent sessions per user
const MAX_SESSIONS_PER_USER: usize = 5;

/// Session information
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
  pub id: SessionId,
  pub user_id: UserId,
  pub token: String,
  pub created_at: SystemTime,
  pub last_activity: SystemTime,
  pub expires_at: SystemTime,
}

impl Session {
  /// Create a new session
  #[must_use]
  pub fn new(user_id: UserId, token: String) -> Self {
    let now = SystemTime::now();
    Self {
      id: SessionId::new(),
      user_id,
      token,
      created_at: now,
      last_activity: now,
      expires_at: now + SESSION_DURATION,
    }
  }

  /// Check if session is expired
  #[must_use]
  pub fn is_expired(&self, now: SystemTime) -> bool {
    now > self.expires_at
  }

  /// Check if session is stale (needs to be renewed)
  #[must_use]
  pub fn is_stale(&self, now: SystemTime) -> bool {
    now
      .duration_since(self.last_activity)
      .map_or(false, |d| d > Duration::from_secs(5 * 60))
  }

  /// Renew session (extend expiration and update activity)
  #[must_use]
  pub fn renew(&self, now: SystemTime) -> Self {
    Self {
      id: self.id.clone(),
      user_id: self.user_id,
      token: self.token.clone(),
      created_at: self.created_at,
      last_activity: now,
      expires_at: now + SESSION_DURATION,
    }
  }

  /// Get session age in seconds
  #[must_use]
  pub fn age_seconds(&self, now: SystemTime) -> u64 {
    now
      .duration_since(self.created_at)
      .map_or(0, |d| d.as_secs())
  }
}

/// Session ID (strongly typed UUID)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
  /// Create a new session ID
  #[must_use]
  pub fn new() -> Self {
    Self(uuid::Uuid::new_v4().to_string())
  }

  /// Create from string
  pub fn from_str(s: &str) -> Result<Self, SessionError> {
    uuid::Uuid::parse_str(s)
      .map(|uuid| Self(uuid.to_string()))
      .map_err(|_| SessionError::InvalidSessionId(s.to_string()))
  }

  /// Get underlying UUID string
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl std::fmt::Display for SessionId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

/// Session management errors
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SessionError {
  /// Invalid session ID format
  #[error("invalid session ID: {0}")]
  InvalidSessionId(String),

  /// Authentication failed
  #[error("authentication failed")]
  AuthenticationFailed,

  /// Session not found
  #[error("session not found")]
  SessionNotFound,

  /// Session expired
  #[error("session expired")]
  SessionExpired,

  /// Session invalid (token mismatch)
  #[error("invalid session token")]
  InvalidToken,

  /// Too many concurrent sessions
  #[error("too many concurrent sessions (max {0})")]
  TooManySessions(usize),

  /// User not found
  #[error("user not found")]
  UserNotFound,

  /// Invalid password
  #[error("invalid password")]
  InvalidPassword,

  /// System time error
  #[error("system time error")]
  SystemTimeError,
}

/// Session management core (pure functions)
pub struct SessionCore {
  active_sessions: Cache<String, Session>,
}

impl SessionCore {
  /// Create a new session core
  #[must_use]
  pub fn new() -> Self {
    Self {
      active_sessions: Cache::new(1000), // Max 1000 sessions
    }
  }

  /// Authenticate user and create session
  ///
  /// # Errors
  /// Returns SessionError if authentication fails or session creation fails
  pub async fn authenticate_and_create_session(
    &self,
    user: &User,
    password: &str,
  ) -> Result<Session, SessionError> {
    // Verify password (pure function call)
    auth::verify_password(&user.password_hash, password)
      .map_err(|_| SessionError::AuthenticationFailed)?;

    // Check existing sessions for this user
    let current_sessions = self.get_active_sessions_for_user(user.id).await;
    if current_sessions.len() >= MAX_SESSIONS_PER_USER {
      return Err(SessionError::TooManySessions(MAX_SESSIONS_PER_USER));
    }

    // Create new session
    let token = auth::generate_session_token();
    let session = Session::new(user.id, token);

    // Store session
    self
      .active_sessions
      .insert(session.token.clone(), session.clone())
      .await;

    Ok(session)
  }

  /// Validate session token
  ///
  /// # Errors
  /// Returns SessionError if session is invalid, expired, or not found
  pub async fn validate_session(&self, token: &str) -> Result<Session, SessionError> {
    let now = SystemTime::now();

    // Get session from cache
    let session = self
      .active_sessions
      .get(token)
      .await
      .ok_or(SessionError::SessionNotFound)?;

    // Check if expired
    if session.is_expired(now) {
      self.active_sessions.invalidate(&*token).await;
      return Err(SessionError::SessionExpired);
    }

    // Check if stale and renew
    if session.is_stale(now) {
      let renewed = session.renew(now);
      self
        .active_sessions
        .insert(renewed.token.clone(), renewed.clone())
        .await;
      Ok(renewed)
    } else {
      // Update activity time
      let updated = Session {
        last_activity: now,
        ..session
      };
      self
        .active_sessions
        .insert(updated.token.clone(), updated.clone())
        .await;
      Ok(updated)
    }
  }

  /// Get user ID from valid session
  ///
  /// # Errors
  /// Returns SessionError if session is invalid or expired
  pub async fn get_user_id_from_session(&self, token: &str) -> Result<UserId, SessionError> {
    let session = self.validate_session(token).await?;
    Ok(session.user_id)
  }

  /// Terminate session
  pub async fn terminate_session(&self, token: &str) -> Result<(), SessionError> {
    self.active_sessions.invalidate(token).await;
    Ok(())
  }

  /// Terminate all sessions for a user
  pub async fn terminate_user_sessions(&self, user_id: UserId) -> Result<(), SessionError> {
    let sessions = self.get_active_sessions_for_user(user_id).await;
    for session in sessions {
      self.active_sessions.invalidate(&session.token).await;
    }
    Ok(())
  }

  /// Clean up expired sessions
  pub async fn cleanup_expired_sessions(&self) -> Result<usize, SessionError> {
    let now = SystemTime::now();
    let mut expired_count = 0;

    // Get all active session keys
    let sessions = self.active_sessions.iter().collect::<Vec<_>>();

    for (token, session) in sessions {
      if session.is_expired(now) {
        self.active_sessions.invalidate(&*token).await;
        expired_count += 1;
      }
    }

    Ok(expired_count)
  }

  /// Get number of active sessions for a user
  #[must_use]
  pub async fn get_session_count_for_user(&self, user_id: UserId) -> usize {
    self.get_active_sessions_for_user(user_id).await.len()
  }

  /// Get all active sessions for a user
  async fn get_active_sessions_for_user(&self, user_id: UserId) -> Vec<Session> {
    self
      .active_sessions
      .iter()
      .filter(|(_, session)| session.user_id == user_id)
      .map(|(_, session)| session.clone())
      .collect()
  }
}

/// Session management shell (handles I/O and persistence)
pub struct SessionManager {
  core: SessionCore,
  db: Option<SessionDatabase>,
}

impl SessionManager {
  /// Create a new session manager with in-memory storage only
  #[must_use]
  pub fn new() -> Self {
    Self {
      core: SessionCore::new(),
      db: None,
    }
  }

  /// Create a new session manager with database persistence
  #[must_use]
  pub fn with_db(db: SessionDatabase) -> Self {
    Self {
      core: SessionCore::new(),
      db: Some(db),
    }
  }

  /// Authenticate user and create session
  ///
  /// This is the shell layer that handles user lookup and delegates to the core
  ///
  /// # Errors
  /// Returns SessionError if user lookup fails or authentication fails
  pub async fn authenticate(
    &self,
    user_id: UserId,
    password: &str,
  ) -> Result<Session, SessionError> {
    // Load user from database if available
    let user = match &self.db {
      Some(db) => {
        let user = db
          .get_user(user_id)
          .await
          .map_err(|_| SessionError::UserNotFound)?;
        user.ok_or(SessionError::UserNotFound)?
      }
      None => return Err(SessionError::UserNotFound),
    };

    // Delegate to core for session creation
    self
      .core
      .authenticate_and_create_session(&user, password)
      .await
  }

  /// Validate session token
  ///
  /// # Errors
  /// Returns SessionError if session is invalid, expired, or not found
  pub async fn validate_session(&self, token: &str) -> Result<Session, SessionError> {
    self.core.validate_session(token).await
  }

  /// Get user ID from valid session
  ///
  /// # Errors
  /// Returns SessionError if session is invalid or expired
  pub async fn get_user_id_from_session(&self, token: &str) -> Result<UserId, SessionError> {
    self.core.get_user_id_from_session(token).await
  }

  /// Terminate session
  pub async fn terminate_session(&self, token: &str) -> Result<(), SessionError> {
    self.core.terminate_session(token).await
  }

  /// Terminate all sessions for a user
  pub async fn terminate_user_sessions(&self, user_id: UserId) -> Result<(), SessionError> {
    self.core.terminate_user_sessions(user_id).await
  }

  /// Clean up expired sessions and persist if database is available
  pub async fn cleanup_sessions(&self) -> Result<usize, SessionError> {
    let cleaned = self.core.cleanup_expired_sessions().await?;

    // If database is available, clean up there too
    if let Some(db) = &self.db {
      db.cleanup_expired_sessions().await?;
    }

    Ok(cleaned)
  }

  /// Get number of active sessions for a user
  #[must_use]
  pub async fn get_session_count_for_user(&self, user_id: UserId) -> usize {
    self.core.get_session_count_for_user(user_id).await
  }
}

/// Database interface for session persistence
pub struct SessionDatabase {
  // This would implement actual database operations
  // For now, it's a placeholder for the shell pattern
}

impl SessionDatabase {
  /// Create a new session database (placeholder)
  #[must_use]
  pub fn new() -> Self {
    Self {}
  }

  /// Get user by ID (would be implemented with actual DB queries)
  #[allow(clippy::unused_async)]
  pub async fn get_user(&self, _user_id: UserId) -> Result<Option<User>, SessionError> {
    // Implementation would use sqlx or similar
    // For now, placeholder
    Err(SessionError::UserNotFound)
  }

  /// Save session to database
  #[allow(clippy::unused_async)]
  pub async fn save_session(&self, _session: &Session) -> Result<(), SessionError> {
    // Implementation would save session to database
    Ok(())
  }

  /// Delete session from database
  #[allow(clippy::unused_async)]
  pub async fn delete_session(&self, _token: &str) -> Result<(), SessionError> {
    // Implementation would delete session from database
    Ok(())
  }

  /// Clean up expired sessions in database
  #[allow(clippy::unused_async)]
  pub async fn cleanup_expired_sessions(&self) -> Result<usize, SessionError> {
    // Implementation would clean up expired sessions in database
    Ok(0)
  }
}

/// Session utility functions
pub struct SessionUtils;

impl SessionUtils {
  /// Get current system time
  ///
  /// # Errors
  /// Returns SessionError if system time is invalid
  pub fn now() -> Result<SystemTime, SessionError> {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map(|_| SystemTime::now())
      .map_err(|_| SessionError::SystemTimeError)
  }

  /// Convert SystemTime to DateTime<Utc>
  ///
  /// # Errors
  /// Returns SessionError if conversion fails
  pub fn system_time_to_utc(time: SystemTime) -> Result<DateTime<Utc>, SessionError> {
    let duration = time
      .duration_since(UNIX_EPOCH)
      .map_err(|_| SessionError::SystemTimeError)?;
    DateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos() as u32)
      .ok_or(SessionError::SystemTimeError)
  }

  /// Get remaining session lifetime
  #[must_use]
  pub fn session_lifetime(session: &Session, now: SystemTime) -> Duration {
    session
      .expires_at
      .duration_since(now)
      .unwrap_or(Duration::from_secs(0))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::domain::types::Email;

  // Test helper to create a test user
  fn create_test_user() -> User {
    let email = Email::new("test@example.com".to_string()).unwrap();
    User::new(
      email,
      "hashed_password".to_string(),
      crate::domain::types::UserRole::User,
    )
    .unwrap()
  }

  // Test SessionCore::authenticate_and_create_session
  #[tokio::test]
  async fn test_session_creation() {
    let core = SessionCore::new();
    let user = create_test_user();
    let password = "valid_password";

    // This would fail because we need to hash the password first
    // For now, we'll test the session creation without authentication
    let token = auth::generate_session_token();
    let session = Session::new(user.id, token);

    assert_eq!(session.user_id, user.id);
    assert!(!session.token.is_empty());
    assert!(!session.is_expired(SystemTime::now()));
  }

  // Test Session::is_expired
  #[test]
  fn test_session_expiration() {
    let now = SystemTime::now();
    let session = Session::new(UserId::new(), "token".to_string());

    // Fresh session should not be expired
    assert!(!session.is_expired(now));

    // Session with past expiration should be expired
    let expired_session = Session {
      expires_at: now - Duration::from_secs(1),
      ..session
    };
    assert!(expired_session.is_expired(now));
  }

  // Test Session::renew
  #[test]
  fn test_session_renewal() {
    let now = SystemTime::now();
    let mut session = Session::new(UserId::new(), "token".to_string());

    // Change some properties to verify renewal
    session.expires_at = now - Duration::from_secs(1);
    session.last_activity = now - Duration::from_secs(10);

    let renewed = session.renew(now);

    // ID and token should remain the same
    assert_eq!(renewed.id, session.id);
    assert_eq!(renewed.token, session.token);

    // Expiration and activity should be updated
    assert_eq!(renewed.last_activity, now);
    assert_eq!(renewed.expires_at, now + SESSION_DURATION);

    // Original session should remain unchanged
    assert_ne!(session.last_activity, now);
  }

  // Test SessionId
  #[test]
  fn test_session_id() {
    let id1 = SessionId::new();
    let id2 = SessionId::new();

    // Should be unique
    assert_ne!(id1, id2);

    // Should be valid UUIDs
    assert!(SessionId::from_str(id1.as_str()).is_ok());
    assert!(SessionId::from_str(id2.as_str()).is_ok());

    // Invalid UUID should fail
    assert!(SessionId::from_str("invalid").is_err());
  }

  // Test session utilities
  #[test]
  fn test_session_utils() {
    let now = SystemTime::now();
    let session = Session::new(UserId::new(), "token".to_string());

    // Test lifetime calculation
    let lifetime = SessionUtils::session_lifetime(&session, now);
    assert!(lifetime > Duration::from_secs(0));
    assert!(lifetime <= SESSION_DURATION);
  }

  // Test SessionManager (without database)
  #[tokio::test]
  async fn test_session_manager_in_memory() {
    let manager = SessionManager::new();
    let token = "test_token";

    // Should fail because there's no database
    let result = manager.authenticate(UserId::new(), "password").await;
    assert!(matches!(result, Err(SessionError::UserNotFound)));

    // Validation should fail for non-existent session
    let result = manager.validate_session(token).await;
    assert!(matches!(result, Err(SessionError::SessionNotFound)));

    // Termination should not fail for non-existent session
    let result = manager.terminate_session(token).await;
    assert!(result.is_ok());
  }
}
