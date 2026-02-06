#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! SQLite connection pool management
//!
//! This module provides SQLite database support for embedded database scenarios.
//! The SQLite database runs in-process, making it ideal for single-user applications
//! or scenarios where data needs to be bundled with the binary.

use crate::db::error::{DbError, DbResult};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::time::Duration;

/// SQLite database configuration
#[derive(Debug, Clone)]
pub struct SqliteDbConfig {
  /// SQLite database path (e.g., "sqlite:clarity.db" or "sqlite::memory:")
  pub database_url: String,
  /// Maximum number of connections in the pool
  pub max_connections: u32,
  /// Timeout when acquiring a connection from the pool
  pub acquire_timeout: Duration,
  /// Timeout for idle connections in the pool
  pub idle_timeout: Duration,
  /// Maximum lifetime of a connection in the pool
  pub max_lifetime: Duration,
}

impl Default for SqliteDbConfig {
  fn default() -> Self {
    Self {
      database_url: "sqlite:clarity.db".to_string(),
      max_connections: 5,
      acquire_timeout: Duration::from_secs(30),
      idle_timeout: Duration::from_secs(600),
      max_lifetime: Duration::from_secs(1800),
    }
  }
}

impl SqliteDbConfig {
  /// Create a new `SqliteDbConfig` from a database URL
  #[must_use]
  pub fn new(database_url: String) -> Self {
    Self {
      database_url,
      ..Default::default()
    }
  }

  /// Create an in-memory `SQLite` database (useful for testing)
  #[must_use]
  pub fn in_memory() -> Self {
    Self::new("sqlite::memory:".to_string())
  }

  /// Create from environment variable `SQLITE_DATABASE_URL`
  ///
  /// # Errors
  /// - Returns `DbError::Validation` if the `SQLITE_DATABASE_URL` environment variable is not set
  pub fn from_env() -> DbResult<Self> {
    std::env::var("SQLITE_DATABASE_URL")
      .map(Self::new)
      .map_err(|_| DbError::Validation("SQLITE_DATABASE_URL environment variable not set".into()))
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

  /// Set idle timeout
  #[must_use]
  pub const fn with_idle_timeout(mut self, timeout: Duration) -> Self {
    self.idle_timeout = timeout;
    self
  }

  /// Set max lifetime
  #[must_use]
  pub const fn with_max_lifetime(mut self, lifetime: Duration) -> Self {
    self.max_lifetime = lifetime;
    self
  }
}

/// Create a `SQLite` database connection pool
///
/// # Errors
/// - Returns a `DbError::DatabaseError` if connection fails
pub async fn create_sqlite_pool(config: &SqliteDbConfig) -> DbResult<SqlitePool> {
  SqlitePoolOptions::new()
    .max_connections(config.max_connections)
    .acquire_timeout(config.acquire_timeout)
    .idle_timeout(config.idle_timeout)
    .max_lifetime(config.max_lifetime)
    .connect(&config.database_url)
    .await
    .map_err(DbError::from)
}

/// Test `SQLite` database connection
///
/// # Errors
/// - Returns a `DbError::DatabaseError` if the connection test fails
pub async fn test_sqlite_connection(pool: &SqlitePool) -> DbResult<()> {
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
  #[test]
  fn test_sqlite_config_default() {
    let config = SqliteDbConfig::default();
    assert_eq!(config.max_connections, 5);
    assert_eq!(config.database_url, "sqlite:clarity.db");
  }

  #[test]
  fn test_sqlite_config_new() {
    let config = SqliteDbConfig::new("sqlite:test.db".to_string());
    assert_eq!(config.database_url, "sqlite:test.db");
    assert_eq!(config.max_connections, 5);
  }

  #[test]
  fn test_sqlite_config_in_memory() {
    let config = SqliteDbConfig::in_memory();
    assert_eq!(config.database_url, "sqlite::memory:");
  }

  #[test]
  fn test_sqlite_config_with_max_connections() {
    let config = SqliteDbConfig::new("sqlite:test.db".to_string()).with_max_connections(10);
    assert_eq!(config.max_connections, 10);
  }

  #[test]
  #[allow(clippy::panic)]
  fn test_sqlite_config_from_env_missing() {
    std::env::remove_var("SQLITE_DATABASE_URL");
    let result = SqliteDbConfig::from_env();
    assert!(result.is_err());
    match result {
      Err(DbError::Validation(_)) => {}
      _ => panic!("Expected Validation error"),
    }
  }

  #[test]
  #[allow(clippy::expect_used)]
  fn test_sqlite_config_from_env_set() {
    std::env::set_var("SQLITE_DATABASE_URL", "sqlite:fromenv.db");
    let result = SqliteDbConfig::from_env();
    assert!(result.is_ok());
    let config = result.expect("Failed to get SqliteDbConfig from environment");
    assert_eq!(config.database_url, "sqlite:fromenv.db");
    std::env::remove_var("SQLITE_DATABASE_URL");
  }

  #[tokio::test]
  async fn test_sqlite_pool_in_memory() {
    let config = SqliteDbConfig::in_memory();
    let pool = create_sqlite_pool(&config)
      .await
      .expect("Failed to create in-memory SQLite pool");

    test_sqlite_connection(&pool)
      .await
      .expect("Failed to test SQLite connection");

    pool.close().await;
  }

  #[tokio::test]
  async fn test_sqlite_query_execution() {
    let config = SqliteDbConfig::in_memory();
    let pool = create_sqlite_pool(&config)
      .await
      .expect("Failed to create in-memory SQLite pool");

    sqlx::query(
      r#"
      CREATE TABLE test_table (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL
      )
      "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create table");

    sqlx::query("INSERT INTO test_table (name) VALUES (?)")
      .bind("test_name")
      .execute(&pool)
      .await
      .expect("Failed to insert data");

    let row = sqlx::query("SELECT name FROM test_table WHERE id = 1")
      .fetch_one(&pool)
      .await
      .expect("Failed to query data");

    let name: String = row.get("name");
    assert_eq!(name, "test_name");

    pool.close().await;
  }
}
