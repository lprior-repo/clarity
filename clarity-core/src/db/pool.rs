#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Database connection pool management
//!
//! This module provides `PostgreSQL` connection pooling with advanced features:
//! - Configurable pool size and timeouts
//! - Pool metrics for monitoring
//! - Automatic reconnection on failures
//! - Health checks

use crate::db::error::{DbError, DbResult};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DbConfig {
  /// `PostgreSQL` connection URL
  pub database_url: String,
  /// Maximum number of connections in the pool
  pub max_connections: u32,
  /// Minimum number of connections in the pool
  pub min_connections: u32,
  /// Timeout when acquiring a connection from the pool
  pub acquire_timeout: Duration,
  /// Timeout for idle connections before being closed
  pub idle_timeout: Duration,
  /// Maximum lifetime of a connection before being recycled
  pub max_lifetime: Duration,
  /// Time to wait before attempting reconnection
  pub reconnect_timeout: Duration,
  /// Maximum number of reconnection attempts
  pub max_reconnect_attempts: u32,
}

impl Default for DbConfig {
  fn default() -> Self {
    Self {
      database_url: "postgresql://localhost/clarity".to_string(),
      max_connections: 10,
      min_connections: 0,
      acquire_timeout: Duration::from_secs(30),
      idle_timeout: Duration::from_secs(600),
      max_lifetime: Duration::from_secs(1800),
      reconnect_timeout: Duration::from_secs(5),
      max_reconnect_attempts: 3,
    }
  }
}

impl DbConfig {
  /// Create a new `DbConfig` from a database URL
  #[must_use]
  pub const fn new(database_url: String) -> Self {
    Self {
      database_url,
      max_connections: 10,
      min_connections: 0,
      acquire_timeout: Duration::from_secs(30),
      idle_timeout: Duration::from_secs(600),
      max_lifetime: Duration::from_secs(1800),
      reconnect_timeout: Duration::from_secs(5),
      max_reconnect_attempts: 3,
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

/// Pool metrics for monitoring
#[derive(Debug, Clone, Copy)]
pub struct PoolMetrics {
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

/// Get pool metrics from a `PostgreSQL` pool
#[must_use]
pub fn get_pool_metrics(pool: &PgPool) -> PoolMetrics {
  let size = pool.size();
  let idle = u32::try_from(pool.num_idle()).unwrap_or(u32::MAX);
  let max_size = pool.options().get_max_connections();
  let active = size.saturating_sub(idle);
  let utilization = if max_size > 0 {
    (active as f32 / max_size as f32) * 100.0
  } else {
    0.0
  };

  PoolMetrics {
    size,
    idle,
    max_size,
    active,
    utilization,
  }
}

/// Pool health status
#[derive(Debug, Clone)]
pub struct PoolHealthStatus {
  /// Whether the pool is healthy
  pub is_healthy: bool,
  /// Current pool metrics
  pub metrics: PoolMetrics,
  /// Health status message
  pub message: String,
}

/// Test pool health by checking connectivity and pool state
///
/// # Errors
/// - Returns `DbError` if the pool is unhealthy or connection test fails
pub async fn test_pool_health(pool: &PgPool) -> DbResult<PoolHealthStatus> {
  // Test basic connectivity
  test_connection(pool).await?;

  // Check pool metrics
  let metrics = get_pool_metrics(pool);

  // Determine health status
  let is_healthy = metrics.utilization < 90.0 && metrics.active < metrics.max_size;

  let message = if is_healthy {
    "Pool is healthy".to_string()
  } else if metrics.utilization >= 90.0 {
    format!("Pool is at high utilization: {:.1}%", metrics.utilization)
  } else {
    "Pool has no available connections".to_string()
  };

  Ok(PoolHealthStatus {
    is_healthy,
    metrics,
    message,
  })
}

/// Acquire a connection from the pool with automatic retry on failure
///
/// This function will attempt to acquire a connection and retry if it fails,
/// up to the configured maximum number of reconnection attempts.
///
/// # Errors
/// - Returns `DbError::Connection` if all reconnection attempts fail
/// - Returns `DbError::AcquisitionTimeout` if connection acquisition times out
pub async fn acquire_with_retry(
  pool: &PgPool,
  config: &DbConfig,
) -> DbResult<sqlx::pool::PoolConnection<sqlx::Postgres>> {
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
    DbError::AcquisitionTimeout("Failed to acquire connection after all retry attempts".to_string())
  }))
}

/// Close the pool gracefully
///
/// This function closes all connections in the pool and waits for them to be released.
pub async fn close_pool(pool: &PgPool) {
  pool.close().await;
}

#[cfg(test)]
mod tests {
  use super::*;

  // Mutex to serialize env var tests (they use shared mutable state)
  use std::sync::Mutex;
  static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

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
    let _lock = ENV_TEST_LOCK.lock().unwrap();
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
    let _lock = ENV_TEST_LOCK.lock().unwrap();
    std::env::set_var("DATABASE_URL", "postgresql://localhost/fromenv");
    let result = DbConfig::from_env();
    assert!(result.is_ok());
    let config = result.expect("Failed to get DbConfig from environment");
    assert_eq!(config.database_url, "postgresql://localhost/fromenv");
    std::env::remove_var("DATABASE_URL");
  }

  #[test]
  fn test_db_config_with_min_connections() {
    let config = DbConfig::new("postgresql://localhost/test".to_string()).with_min_connections(2);
    assert_eq!(config.min_connections, 2);
  }

  #[test]
  fn test_db_config_with_reconnect_settings() {
    let config = DbConfig::new("postgresql://localhost/test".to_string())
      .with_reconnect_timeout(Duration::from_secs(10))
      .with_max_reconnect_attempts(5);

    assert_eq!(config.reconnect_timeout, Duration::from_secs(10));
    assert_eq!(config.max_reconnect_attempts, 5);
  }

  #[test]
  fn test_db_config_with_all_timeouts() {
    let config = DbConfig::new("postgresql://localhost/test".to_string())
      .with_acquire_timeout(Duration::from_secs(60))
      .with_idle_timeout(Duration::from_secs(300))
      .with_max_lifetime(Duration::from_secs(3600));

    assert_eq!(config.acquire_timeout, Duration::from_secs(60));
    assert_eq!(config.idle_timeout, Duration::from_secs(300));
    assert_eq!(config.max_lifetime, Duration::from_secs(3600));
  }
}
