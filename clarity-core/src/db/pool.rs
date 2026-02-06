#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Database connection pool management

use crate::db::error::{DbError, DbResult};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DbConfig {
  pub database_url: String,
  pub max_connections: u32,
  pub acquire_timeout: Duration,
  pub idle_timeout: Duration,
  pub max_lifetime: Duration,
}

impl Default for DbConfig {
  fn default() -> Self {
    Self {
      database_url: "postgresql://localhost/clarity".to_string(),
      max_connections: 10,
      acquire_timeout: Duration::from_secs(30),
      idle_timeout: Duration::from_secs(600),
      max_lifetime: Duration::from_secs(1800),
    }
  }
}

impl DbConfig {
  /// Create a new `DbConfig` from a database URL
  #[must_use]
  pub fn new(database_url: String) -> Self {
    Self {
      database_url,
      ..Default::default()
    }
  }

  /// Create from environment variable `DATABASE_URL`
  ///
  /// # Errors
  /// - Returns `DbError::Validation` if the `DATABASE_URL` environment variable is not set
  pub fn from_env() -> DbResult<Self> {
    std::env::var("DATABASE_URL")
      .map(Self::new)
      .map_err(|_| DbError::Validation("DATABASE_URL environment variable not set".into()))
  }

  /// Set max connections
  #[must_use]
  pub const fn with_max_connections(mut self, max: u32) -> Self {
    self.max_connections = max;
    self
  }

  /// Set acquire timeout
  #[must_use]
  pub const fn with_acquire_timeout(mut self, timeout: Duration) -> Self {
    self.acquire_timeout = timeout;
    self
  }
}

/// Create a database connection pool
///
/// # Errors
/// - Returns a `DbError::DatabaseError` if connection fails
pub async fn create_pool(config: &DbConfig) -> DbResult<PgPool> {
  PgPoolOptions::new()
    .max_connections(config.max_connections)
    .acquire_timeout(config.acquire_timeout)
    .idle_timeout(config.idle_timeout)
    .max_lifetime(config.max_lifetime)
    .connect(&config.database_url)
    .await
    .map_err(DbError::from)
}

/// Test database connection
///
/// # Errors
/// - Returns a `DbError::DatabaseError` if the connection test fails
pub async fn test_connection(pool: &PgPool) -> DbResult<()> {
  sqlx::query("SELECT 1")
    .fetch_one(pool)
    .await
    .map(|_| ())
    .map_err(DbError::from)
}

#[cfg(test)]
mod tests {
  use super::*;
  #[allow(clippy::expect_used)]
  #[allow(clippy::panic)]
  #[allow(clippy::unwrap_used)]
  #[test]
  fn test_db_config_default() {
    let config = DbConfig::default();
    assert_eq!(config.max_connections, 10);
    assert_eq!(config.database_url, "postgresql://localhost/clarity");
  }

  #[test]
  fn test_db_config_new() {
    let config = DbConfig::new("postgresql://localhost/test".to_string());
    assert_eq!(config.database_url, "postgresql://localhost/test");
    assert_eq!(config.max_connections, 10);
  }

  #[test]
  fn test_db_config_with_max_connections() {
    let config = DbConfig::new("postgresql://localhost/test".to_string()).with_max_connections(20);
    assert_eq!(config.max_connections, 20);
  }

  #[test]
  #[allow(clippy::panic)]
  fn test_db_config_from_env_missing() {
    // Remove DATABASE_URL if it exists
    std::env::remove_var("DATABASE_URL");
    let result = DbConfig::from_env();
    assert!(result.is_err());
    match result {
      Err(DbError::Validation(_)) => {}
      _ => panic!("Expected Validation error"),
    }
  }

  #[test]
  #[allow(clippy::expect_used)]
  fn test_db_config_from_env_set() {
    std::env::set_var("DATABASE_URL", "postgresql://localhost/fromenv");
    let result = DbConfig::from_env();
    assert!(result.is_ok());
    let config = result.expect("Failed to get DbConfig from environment");
    assert_eq!(config.database_url, "postgresql://localhost/fromenv");
    std::env::remove_var("DATABASE_URL");
  }
}
