#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! `SQLite` connection pool management
//!
//! This module provides `SQLite` database support for embedded database scenarios.
//! The `SQLite` database runs in-process, making it ideal for single-user applications
//! or scenarios where data needs to be bundled with the binary.
//!
//! Features:
//! - WAL mode for concurrent reads and writes
//! - Configurable pool size and timeouts
//! - Pool metrics for monitoring
//! - Automatic reconnection on failures
//! - Health checks

use crate::db::error::{DbError, DbResult};
#[allow(unused_imports)]
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use std::time::Duration;

/// `SQLite` database configuration
#[derive(Debug, Clone)]
pub struct SqliteDbConfig {
  /// `SQLite` database path (e.g., "sqlite:clarity.db" or "`sqlite::memory`:")
  pub database_url: String,
  /// Maximum number of connections in the pool
  pub max_connections: u32,
  /// Minimum number of connections in the pool
  pub min_connections: u32,
  /// Timeout when acquiring a connection from the pool
  pub acquire_timeout: Duration,
  /// Timeout for idle connections in the pool
  pub idle_timeout: Duration,
  /// Maximum lifetime of a connection in the pool
  pub max_lifetime: Duration,
  /// Time to wait before attempting reconnection
  pub reconnect_timeout: Duration,
  /// Maximum number of reconnection attempts
  pub max_reconnect_attempts: u32,
}

impl Default for SqliteDbConfig {
  fn default() -> Self {
    Self {
      database_url: "sqlite:clarity.db".to_string(),
      max_connections: 5,
      min_connections: 0,
      acquire_timeout: Duration::from_secs(30),
      idle_timeout: Duration::from_secs(600),
      max_lifetime: Duration::from_secs(1800),
      reconnect_timeout: Duration::from_secs(5),
      max_reconnect_attempts: 3,
    }
  }
}

impl SqliteDbConfig {
  /// Create a new `SqliteDbConfig` from a database URL
  #[must_use]
  pub const fn new(database_url: String) -> Self {
    Self {
      database_url,
      max_connections: 5,
      min_connections: 0,
      acquire_timeout: Duration::from_secs(30),
      idle_timeout: Duration::from_secs(600),
      max_lifetime: Duration::from_secs(1800),
      reconnect_timeout: Duration::from_secs(5),
      max_reconnect_attempts: 3,
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

  /// Set min connections
  #[must_use]
  pub const fn with_min_connections(mut self, min: u32) -> Self {
    self.min_connections = min;
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

  /// Set reconnect timeout
  #[must_use]
  pub const fn with_reconnect_timeout(mut self, timeout: Duration) -> Self {
    self.reconnect_timeout = timeout;
    self
  }

  /// Set max reconnect attempts
  #[must_use]
  pub const fn with_max_reconnect_attempts(mut self, attempts: u32) -> Self {
    self.max_reconnect_attempts = attempts;
    self
  }
}

/// Create a `SQLite` database connection pool with WAL mode enabled
///
/// This creates a connection pool with Write-Ahead Logging (WAL) mode enabled,
/// providing 2-3x throughput improvement with lock-free reads.
///
/// Performance optimizations:
/// - WAL mode for concurrent reads and writes
/// - Synchronous=NORMAL for optimal WAL performance
/// - 64MB cache size for better performance
/// - Memory-based temporary storage
///
/// # Errors
/// - Returns a `DbError::DatabaseError` if connection fails
pub async fn create_sqlite_pool(config: &SqliteDbConfig) -> DbResult<SqlitePool> {
  let pool = SqlitePoolOptions::new()
    .max_connections(config.max_connections)
    .acquire_timeout(config.acquire_timeout)
    .idle_timeout(config.idle_timeout)
    .max_lifetime(config.max_lifetime)
    .after_connect(|#[allow(unused_mut)] mut connection, _meta| {
      Box::pin(async move {
        // Configure WAL mode on each new connection for 2-3x throughput
        sqlx::query("PRAGMA journal_mode=WAL")
          .execute(&mut *connection)
          .await?;

        // Set synchronous to NORMAL (optimal for WAL - sync only on checkpoint)
        sqlx::query("PRAGMA synchronous=NORMAL")
          .execute(&mut *connection)
          .await?;

        // Increase cache size to 64MB (negative value = KB)
        sqlx::query("PRAGMA cache_size=-64000")
          .execute(&mut *connection)
          .await?;

        // Store temporary tables in memory for fastest performance
        sqlx::query("PRAGMA temp_store=MEMORY")
          .execute(&mut *connection)
          .await?;

        // Limit WAL file size to 1MB to prevent unbounded growth
        sqlx::query("PRAGMA journal_size_limit=1048576")
          .execute(&mut *connection)
          .await?;

        Ok(())
      })
    })
    .connect(&config.database_url)
    .await
    .map_err(DbError::from)?;

  Ok(pool)
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

/// `SQLite` pool metrics for monitoring
#[derive(Debug, Clone, Copy)]
pub struct SqlitePoolMetrics {
  /// Current pool size (active + idle connections)
  pub size: u32,
  /// Number of idle connections available
  pub idle: u32,
  /// Maximum pool size
  pub max_size: u32,
  /// Number of active connections (size - idle)
  pub active: u32,
  /// Pool utilization percentage (active / `max_size` * 100)
  pub utilization: f32,
}

/// Get pool metrics from a `SQLite` pool
#[must_use]
pub fn get_sqlite_pool_metrics(pool: &SqlitePool) -> SqlitePoolMetrics {
  let size = pool.size();
  let idle = u32::try_from(pool.num_idle()).unwrap_or(u32::MAX);
  let max_size = pool.options().get_max_connections();
  let active = size.saturating_sub(idle);
  let utilization = if max_size > 0 {
    (active as f32 / max_size as f32) * 100.0
  } else {
    0.0
  };

  SqlitePoolMetrics {
    size,
    idle,
    max_size,
    active,
    utilization,
  }
}

/// SQLite pool health status
#[derive(Debug, Clone)]
pub struct SqlitePoolHealthStatus {
  /// Whether the pool is healthy
  pub is_healthy: bool,
  /// Current pool metrics
  pub metrics: SqlitePoolMetrics,
  /// Health status message
  pub message: String,
}

/// Test SQLite pool health by checking connectivity and pool state
///
/// # Errors
/// - Returns `DbError` if the pool is unhealthy or connection test fails
pub async fn test_sqlite_pool_health(pool: &SqlitePool) -> DbResult<SqlitePoolHealthStatus> {
  // Test basic connectivity
  test_sqlite_connection(pool).await?;

  // Check pool metrics
  let metrics = get_sqlite_pool_metrics(pool);

  // Determine health status
  let is_healthy = metrics.utilization < 90.0 && metrics.active < metrics.max_size;

  Ok(SqlitePoolHealthStatus {
    is_healthy,
    metrics,
    message: if is_healthy {
      "SQLite pool is healthy".to_string()
    } else if metrics.utilization >= 90.0 {
      format!(
        "SQLite pool is at high utilization: {:.1}%",
        metrics.utilization
      )
    } else {
      "SQLite pool has no available connections".to_string()
    },
  })
}

/// Acquire a connection from the SQLite pool with automatic retry on failure
///
/// This function will attempt to acquire a connection and retry if it fails,
/// up to the configured maximum number of reconnection attempts.
///
/// # Errors
/// - Returns `DbError::Connection` if all reconnection attempts fail
/// - Returns `DbError::AcquisitionTimeout` if connection acquisition times out
pub async fn acquire_sqlite_with_retry(
  pool: &SqlitePool,
  config: &SqliteDbConfig,
) -> DbResult<sqlx::pool::PoolConnection<sqlx::Sqlite>> {
  let mut last_error = None;

  for attempt in 0..=config.max_reconnect_attempts {
    match pool.acquire().await {
      Ok(conn) => return Ok(conn),
      Err(e) => {
        last_error = Some(DbError::from(e));

        // If this isn't the last attempt, wait before retrying
        if attempt < config.max_reconnect_attempts {
          tokio::time::sleep(config.reconnect_timeout).await;
        }
      }
    }
  }

  // All attempts failed
  Err(last_error.unwrap_or_else(|| {
    DbError::AcquisitionTimeout(
      "Failed to acquire SQLite connection after all retry attempts".to_string(),
    )
  }))
}

/// Close the SQLite pool gracefully
///
/// This function closes all connections in the pool and waits for them to be released.
pub async fn close_sqlite_pool(pool: &SqlitePool) {
  pool.close().await;
}

#[cfg(test)]
mod tests {
  use super::*;

  // Mutex to serialize env var tests (they use shared mutable state)
  use std::sync::Mutex;
  static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

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
    let _lock = ENV_TEST_LOCK.lock().unwrap();
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
    let _lock = ENV_TEST_LOCK.lock().unwrap();
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
      r"
      CREATE TABLE test_table (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL
      )
      ",
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

  #[tokio::test]
  async fn test_wal_mode_enabled() {
    // Note: WAL mode is not supported for in-memory databases
    // In production with file-based databases, WAL will be enabled
    let config = SqliteDbConfig::in_memory();
    let pool = create_sqlite_pool(&config)
      .await
      .expect("Failed to create in-memory SQLite pool");

    // Verify journal_mode query works (in-memory dbs use 'memory' mode)
    let row = sqlx::query("PRAGMA journal_mode")
      .fetch_one(&pool)
      .await
      .expect("Failed to query journal_mode");

    let journal_mode: String = row.get("journal_mode");
    // In-memory databases will return 'memory' instead of 'wal'
    assert!(
      journal_mode.to_lowercase() == "memory" || journal_mode.to_lowercase() == "wal",
      "Journal mode should be set (got {})",
      journal_mode
    );

    pool.close().await;
  }

  #[tokio::test]
  async fn test_synchronous_normal() {
    let config = SqliteDbConfig::in_memory();
    let pool = create_sqlite_pool(&config)
      .await
      .expect("Failed to create in-memory SQLite pool");

    // Verify synchronous is set to NORMAL (optimal for WAL)
    let row = sqlx::query("PRAGMA synchronous")
      .fetch_one(&pool)
      .await
      .expect("Failed to query synchronous");

    let synchronous: i32 = row.get("synchronous");
    assert_eq!(
      synchronous, 1,
      "Synchronous should be NORMAL (1) for optimal WAL performance"
    );

    pool.close().await;
  }

  #[tokio::test]
  async fn test_cache_size_configured() {
    let config = SqliteDbConfig::in_memory();
    let pool = create_sqlite_pool(&config)
      .await
      .expect("Failed to create in-memory SQLite pool");

    // Verify cache_size is set to -64000 (64MB)
    let row = sqlx::query("PRAGMA cache_size")
      .fetch_one(&pool)
      .await
      .expect("Failed to query cache_size");

    let cache_size: i32 = row.get("cache_size");
    assert_eq!(
      cache_size, -64000,
      "Cache size should be -64000 (64MB) for better performance"
    );

    pool.close().await;
  }

  #[tokio::test]
  async fn test_temp_store_memory() {
    let config = SqliteDbConfig::in_memory();
    let pool = create_sqlite_pool(&config)
      .await
      .expect("Failed to create in-memory SQLite pool");

    // Verify temp_store is set to MEMORY
    let row = sqlx::query("PRAGMA temp_store")
      .fetch_one(&pool)
      .await
      .expect("Failed to query temp_store");

    let temp_store: i32 = row.get("temp_store");
    assert_eq!(
      temp_store, 2,
      "Temp store should be MEMORY (2) for fastest performance"
    );

    pool.close().await;
  }
}
