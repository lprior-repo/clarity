#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Database schema initialization
//!
//! Provides functions to initialize the database schema for all entities.
//!
//! **Note**: This module provides a manual schema initialization fallback.
//! The preferred way to initialize the database is through migrations
//! using `run_migrations()` or `create_and_migrate()` from the `migrate` module.
//! Migrations provide better tracking and rollback capabilities.

use crate::db::error::{DbError, DbResult};
use sqlx::SqlitePool;

/// Initialize the database schema
///
/// Creates all necessary tables if they don't exist.
///
/// # Errors
/// Returns `DbError` if:
/// - Database connection fails
/// - Table creation fails
pub async fn init_schema(pool: &SqlitePool) -> DbResult<()> {
  // Create users table
  sqlx::query(
    r"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'user',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        ",
  )
  .execute(pool)
  .await?;

  // Create beads table with foreign key and constraints
  sqlx::query(
    r"
        CREATE TABLE IF NOT EXISTS beads (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL DEFAULT 'open',
            priority INTEGER NOT NULL DEFAULT 2 CHECK (priority BETWEEN 1 AND 3),
            bead_type TEXT NOT NULL DEFAULT 'feature',
            created_by TEXT REFERENCES users(id) ON DELETE SET NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        ",
  )
  .execute(pool)
  .await?;

  // Create interviews table
  sqlx::query(
    r"
        CREATE TABLE IF NOT EXISTS interviews (
            id TEXT PRIMARY KEY,
            spec_name TEXT NOT NULL,
            questions TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        ",
  )
  .execute(pool)
  .await?;

  // Create specs table
  sqlx::query(
    r"
        CREATE TABLE IF NOT EXISTS specs (
            id TEXT PRIMARY KEY,
            name TEXT UNIQUE NOT NULL,
            description TEXT,
            schema TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        ",
  )
  .execute(pool)
  .await?;

  // Create indexes for common queries
  sqlx::query("CREATE INDEX IF NOT EXISTS idx_beads_status ON beads(status)")
    .execute(pool)
    .await?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_beads_type ON beads(bead_type)")
    .execute(pool)
    .await?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_beads_priority ON beads(priority)")
    .execute(pool)
    .await?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_beads_created_by ON beads(created_by)")
    .execute(pool)
    .await?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)")
    .execute(pool)
    .await?;

  sqlx::query("CREATE INDEX IF NOT EXISTS idx_interviews_spec_name ON interviews(spec_name)")
    .execute(pool)
    .await?;

  Ok(())
}

/// Check if database schema is initialized
///
/// # Errors
/// Returns `DbError` if:
/// - Database connection fails
/// - Query fails
pub async fn is_schema_initialized(pool: &SqlitePool) -> Result<bool, DbError> {
  let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='beads'")
    .fetch_optional(pool)
    .await?;

  Ok(result.is_some())
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_module_exists() {
    // This test verifies the module compiles
    // Actual schema tests require a database
  }
}
