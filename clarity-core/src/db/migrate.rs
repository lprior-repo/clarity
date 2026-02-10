#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Database migrations
//!
//! This module uses `SQLx`'s migrate system to manage database schema migrations.
//! Migrations are stored in the `migrations/` directory and are applied
//! automatically when the application starts.

use crate::db::error::{DbError, DbResult};
use sqlx::{migrate::MigrateDatabase, sqlite::Sqlite, SqlitePool};

/// Run all pending migrations for `SQLite`
///
/// This function creates the database if it doesn't exist and applies
/// any pending migrations. Migrations are tracked in the `_sqlx_migrations`
/// table to ensure they're only applied once.
///
/// # Errors
/// - Returns a `DbError::Migration` if migrations fail to execute
pub async fn run_migrations(pool: &SqlitePool) -> DbResult<()> {
  // Use the migrate! macro which embeds migrations at compile time
  // The migrations should be in the migrations/ directory relative to the crate root
  sqlx::migrate!()
    .run(pool)
    .await
    .map_err(DbError::from)
}

/// Create a new `SQLite` database file and run migrations
///
/// # Errors
/// - Returns a `DbError::DatabaseError` if database creation fails
/// - Returns a `DbError::Migration` if migrations fail
pub async fn create_and_migrate(database_url: &str) -> DbResult<SqlitePool> {
  // Create database if it doesn't exist (for file-based databases)
  if Sqlite::database_exists(database_url)
    .await
    .map_err(DbError::from)?
    && !database_url.starts_with("sqlite::memory:")
  {
    Sqlite::create_database(database_url)
      .await
      .map_err(DbError::from)?;
  }

  // Create connection pool
  let pool = SqlitePool::connect(database_url)
    .await
    .map_err(DbError::from)?;

  // Run migrations
  run_migrations(&pool).await?;

  Ok(pool)
}

/// Parse database URL and return the file path for file-based databases
///
/// # Errors
/// - Returns `DbError::Validation` if the URL format is invalid
pub fn get_database_path(database_url: &str) -> DbResult<std::path::PathBuf> {
  match database_url.strip_prefix("sqlite:") {
    Some(path) if !path.starts_with(':') => Ok(std::path::PathBuf::from(path)),
    _ => Err(DbError::Validation(
      "Invalid SQLite database URL format".into(),
    )),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_database_path_file() {
    let path = get_database_path("sqlite:/path/to/db.sqlite").unwrap();
    assert_eq!(path, std::path::PathBuf::from("/path/to/db.sqlite"));
  }

  #[test]
  fn test_get_database_path_relative() {
    let path = get_database_path("sqlite:clarity.db").unwrap();
    assert_eq!(path, std::path::PathBuf::from("clarity.db"));
  }

  #[test]
  fn test_get_database_path_memory() {
    let result = get_database_path("sqlite::memory:");
    assert!(result.is_err());
  }

  #[test]
  fn test_get_database_path_invalid() {
    let result = get_database_path("postgresql://localhost/db");
    assert!(result.is_err());
  }

  #[tokio::test]
  async fn test_run_migrations_in_memory() {
    let pool = SqlitePool::connect("sqlite::memory:")
      .await
      .expect("Failed to connect to in-memory database");

    let result = run_migrations(&pool).await;
    assert!(result.is_ok(), "Migrations should succeed: {result:?}");

    // Verify all tables were created
    let tables = ["beads", "users", "interviews", "specs"];
    for table in tables {
      let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?)",
      )
      .bind(table)
      .fetch_one(&pool)
      .await
      .expect("Failed to check if table exists");

      assert!(exists, "{table} table should exist after migrations");
    }

    // Verify indexes were created
    let indexes = [
      "idx_beads_status",
      "idx_beads_type",
      "idx_beads_priority",
      "idx_beads_created_by",
      "idx_users_email",
      "idx_interviews_spec_name",
    ];
    for index in indexes {
      let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='index' AND name=?)",
      )
      .bind(index)
      .fetch_one(&pool)
      .await
      .expect("Failed to check if index exists");

      assert!(exists, "{index} index should exist after migrations");
    }

    // Verify beads table has CHECK constraint on priority
    // This will fail if the constraint doesn't exist
    let result = sqlx::query("INSERT INTO beads (id, title, status, priority, bead_type, created_at, updated_at) VALUES ('test-id', 'Test', 'open', 5, 'feature', datetime('now'), datetime('now'))")
      .execute(&pool)
      .await;

    assert!(result.is_err(), "Should reject invalid priority (5) due to CHECK constraint");

    pool.close().await;
  }
}
