#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Database schema initialization
//!
//! Provides functions to initialize the database schema for all entities.

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
    // Create beads table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS beads (
            id TEXT PRIMARY KEY NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL DEFAULT 'open',
            priority INTEGER NOT NULL DEFAULT 2,
            bead_type TEXT NOT NULL DEFAULT 'feature',
            created_by TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'user',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
    )
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
    let result = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='beads'"
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exists() {
        // This test verifies the module compiles
        // Actual schema tests require a database
    }
}
