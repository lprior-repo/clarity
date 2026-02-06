#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Database migrations

use crate::db::error::{DbError, DbResult};
use sqlx::PgPool;

/// Run all pending migrations
///
/// # Errors
/// - Returns a `DbError::Migration` if migrations fail to execute
pub async fn run_migrations(pool: &PgPool) -> DbResult<()> {
  sqlx::migrate!("./migrations")
    .run(pool)
    .await
    .map_err(|e| DbError::Migration(format!("Migration failed: {e}")))
}

/// Get migration version information
///
/// # Errors
/// - Returns a `DbError::DatabaseError` if the query fails
pub async fn get_migration_version(pool: &PgPool) -> DbResult<Option<i64>> {
  let result =
    sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version DESC LIMIT 1")
      .fetch_optional(pool)
      .await
      .map_err(DbError::from)?;

  Ok(result)
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_migration_module_exists() {
    // This test verifies the module compiles
    // Actual migration tests require a database
  }
}
