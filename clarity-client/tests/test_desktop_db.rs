#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Integration tests for embedded SQLite database initialization
//!
//! Tests use in-memory databases for speed and isolation.

use anyhow::{Context, Result};
use clarity_core::db::migrate::run_migrations;
use clarity_core::db::sqlite_pool::{create_sqlite_pool, SqliteDbConfig};

/// Create an in-memory SQLite database config for testing
#[must_use]
pub fn in_memory_config() -> SqliteDbConfig {
  SqliteDbConfig::new("sqlite::memory:".to_string())
}

async fn initialize_in_memory_database() -> Result<SqliteDbConfig> {
  let config = in_memory_config();

  let pool = create_sqlite_pool(&config)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to create SQLite connection pool: {e}"))?;

  run_migrations(&pool)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to run database migrations: {e}"))?;

  pool.close().await;

  Ok(config)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_database_initialization() {
    let result = initialize_in_memory_database().await;
    assert!(result.is_ok(), "Database initialization should succeed");
  }

  #[tokio::test]
  async fn test_database_schema_creation() {
    let config = in_memory_config();

    let pool_result = create_sqlite_pool(&config).await;
    assert!(pool_result.is_ok(), "Should create pool");

    let pool = pool_result
      .map_err(|e| anyhow::anyhow!("Pool creation failed: {e}"))
      .context("Failed to create pool for test");

    let pool = match pool {
      Ok(p) => p,
      Err(e) => {
        panic!("Pool creation failed: {e:?}");
      }
    };

    let migrate_result = run_migrations(&pool).await;
    assert!(migrate_result.is_ok(), "Should run migrations");

    // Verify tables exist
    let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='beads'")
      .fetch_one(&pool)
      .await;

    assert!(result.is_ok(), "Should query sqlite_master");

    pool.close().await;
  }
}
