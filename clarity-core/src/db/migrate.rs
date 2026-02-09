#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Database migrations

use crate::db::error::{DbError, DbResult};
use sqlx::SqlitePool;

/// Run all pending migrations for SQLite
///
/// # Errors
/// - Returns a `DbError::Migration` if migrations fail to execute
pub async fn run_migrations(pool: &SqlitePool) -> DbResult<()> {
  // Use schema initialization instead of migrations for now
  crate::db::schema::init_schema(pool).await
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_migration_module_exists() {
    // This test verifies the module compiles
    // Actual migration tests require a database
  }
}
