#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Session management for Clarity
//!
//! Provides a complete session management system with:
//! - Session token management with expiration
//! - Session validation and cleanup
//! - Persistent session storage
//!
//! Follows functional core, imperative shell pattern:
//! - Core: Pure functions for business logic
//! - Shell: I/O operations and persistence

use chrono::{DateTime, Utc};
use moka::future::Cache;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Session duration (30 minutes of inactivity)
const SESSION_DURATION: Duration = Duration::from_secs(30 * 60);

/// Session information
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
  pub id: SessionId,
  pub token: String,
  pub created_at: SystemTime,
  pub last_activity: SystemTime,
  pub expires_at: SystemTime,
}

impl Session {
  /// Create a new session
  #[must_use]
  pub fn new(token: String) -> Self {
    let now = SystemTime::now();
    Self {
      id: SessionId::new(),
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

  /// Create a new session and store it
  pub async fn create_session(&self, token: String) -> Session {
    let session = Session::new(token);
    self
      .active_sessions
      .insert(session.token.clone(), session.clone())
      .await;
    session
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

  /// Terminate session
  pub async fn terminate_session(&self, token: &str) -> Result<(), SessionError> {
    self.active_sessions.invalidate(token).await;
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

  /// Get number of active sessions
  #[must_use]
  pub async fn get_session_count(&self) -> usize {
    self.active_sessions.entry_count() as usize
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

  /// Create a new session
  pub async fn create_session(&self, token: String) -> Session {
    self.core.create_session(token).await
  }

  /// Validate session token
  ///
  /// # Errors
  /// Returns SessionError if session is invalid, expired, or not found
  pub async fn validate_session(&self, token: &str) -> Result<Session, SessionError> {
    self.core.validate_session(token).await
  }

  /// Terminate session
  pub async fn terminate_session(&self, token: &str) -> Result<(), SessionError> {
    self.core.terminate_session(token).await
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

  /// Get number of active sessions
  #[must_use]
  pub async fn get_session_count(&self) -> usize {
    self.core.get_session_count().await
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
      .map_or(Duration::from_secs(0), |d| d)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // Test SessionCore::create_session
  #[tokio::test]
  async fn test_session_creation() {
    let core = SessionCore::new();
    let token = "test_token".to_string();
    let session = core.create_session(token.clone()).await;

    assert_eq!(session.token, token);
    assert!(!session.is_expired(SystemTime::now()));
  }

  // Test Session::is_expired
  #[test]
  fn test_session_expiration() {
    let now = SystemTime::now();
    let session = Session::new("token".to_string());

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
    let session = Session::new("token".to_string());

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
    assert_eq!(renewed.expires_at, now + SESSION_DURATION);

    // Original session should remain unchanged
    assert_ne!(stale_session.last_activity, now);
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
    let session = Session::new("token".to_string());

    // Test lifetime calculation
    let lifetime = SessionUtils::session_lifetime(&session, now);
    assert!(lifetime > Duration::from_secs(0));
    assert!(lifetime <= SESSION_DURATION);
  }

  // Test SessionManager (without database)
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
    assert!(matches!(result, Err(SessionError::SessionNotFound)));
  }
}
