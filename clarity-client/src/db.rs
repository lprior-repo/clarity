#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Desktop database module for client-side `SQLite` access
//!
//! Migrated from rusqlite to `SQLx` for:
//! - Compile-time query checking
//! - Async/await support
//! - Parameterized queries (SQL injection prevention)
//! - Connection pooling with WAL mode

use anyhow::Result;
use clarity_core::auth;
use clarity_core::db::models::{Bead, BeadFilters, BeadId, Email, NewBead, User, UserId, UserRole};
use clarity_core::db::{sqlite_pool, DbError, DbResult, SqliteDbConfig};
use sqlx::{Row, SqlitePool};
use std::path::{Path, PathBuf};

/// Desktop database wrapper using `SQLx` connection pool
#[derive(Debug, Clone)]
pub struct DesktopDb {
  pool: SqlitePool,
  db_path: PathBuf,
}

impl DesktopDb {
  /// Create a new `DesktopDb` with default path (blocking)
  ///
  /// This is a convenience wrapper for use in synchronous contexts.
  /// It creates a tokio runtime if one doesn't exist.
  ///
  /// # Errors
  /// - Returns error if data directory cannot be determined
  /// - Returns error if database directory cannot be created
  /// - Returns error if pool creation fails
  pub fn new() -> Result<Self> {
    // Try to use existing runtime, or create a new one
    let rt = tokio::runtime::Handle::try_current()
      .or_else(|_| tokio::runtime::Runtime::new().map(|rt| rt.handle().clone()))
      .map_err(|e| anyhow::anyhow!("Failed to get runtime: {e}"))?;

    rt.block_on(Self::new_async())
  }

  /// Create a new `DesktopDb` with default path (async)
  ///
  /// # Errors
  /// - Returns error if data directory cannot be determined
  /// - Returns error if database directory cannot be created
  /// - Returns error if pool creation fails
  pub async fn new_async() -> Result<Self> {
    let data_dir = dirs::data_local_dir()
      .ok_or_else(|| anyhow::anyhow!("Failed to determine local data directory"))?;

    let app_dir = data_dir.join("clarity");
    tokio::fs::create_dir_all(&app_dir).await?;

    let db_path = app_dir.join("clarity.db");
    Self::with_path_async(db_path).await
  }

  /// Get the database file path
  #[must_use]
  pub fn db_path(&self) -> &Path {
    &self.db_path
  }

  /// Create a new `DesktopDb` with a specific path (blocking)
  ///
  /// # Errors
  /// - Returns error if database directory cannot be created
  /// - Returns error if pool creation fails
  pub fn with_path(path: PathBuf) -> Result<Self> {
    let rt = tokio::runtime::Handle::try_current()
      .or_else(|_| tokio::runtime::Runtime::new().map(|rt| rt.handle().clone()))
      .map_err(|e| anyhow::anyhow!("Failed to get runtime: {e}"))?;

    rt.block_on(Self::with_path_async(path))
  }

  /// Create a new `DesktopDb` with a specific path (async)
  ///
  /// # Errors
  /// - Returns error if database directory cannot be created
  /// - Returns error if pool creation fails
  pub async fn with_path_async(path: PathBuf) -> Result<Self> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
      tokio::fs::create_dir_all(parent).await?;
    }

    let db_path = path.clone();
    let database_url = format!("sqlite:{}", path.display());
    let config = SqliteDbConfig::new(database_url);
    let pool = sqlite_pool::create_sqlite_pool(&config).await?;

    // Run migrations
    Self::run_migrations(&pool).await?;

    Ok(Self { pool, db_path })
  }

  /// Create a new `DesktopDb` with an in-memory database (useful for testing)
  ///
  /// # Errors
  /// - Returns error if pool creation fails
  pub async fn in_memory() -> Result<Self> {
    let config = SqliteDbConfig::in_memory();
    let pool = sqlite_pool::create_sqlite_pool(&config).await?;

    // Run migrations
    Self::run_migrations(&pool).await?;

    Ok(Self {
      pool,
      db_path: PathBuf::from(":memory:"),
    })
  }

  /// Run database migrations
  ///
  /// # Errors
  /// - Returns error if migration SQL execution fails
  async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
      r"
            CREATE TABLE IF NOT EXISTS beads (
                id TEXT PRIMARY KEY NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL,
                priority INTEGER NOT NULL,
                bead_type TEXT NOT NULL,
                created_by TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_beads_status ON beads(status);
            CREATE INDEX IF NOT EXISTS idx_beads_type ON beads(bead_type);
            CREATE INDEX IF NOT EXISTS idx_beads_priority ON beads(priority);

            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY NOT NULL,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                role TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
            ",
    )
    .execute(pool)
    .await?;

    Ok(())
  }

  /// List all beads without filtering
  ///
  /// # Errors
  /// - Returns `DbError::Connection` if query execution fails
  /// - Returns `DbError::InvalidUuid` if ID parsing fails
  /// - Returns `DbError::Validation` if status/type parsing fails
  pub async fn list_beads(&self) -> DbResult<Vec<Bead>> {
    let rows = sqlx::query(
            r"
            SELECT id, title, description, status, priority, bead_type, created_by, created_at, updated_at
            FROM beads
            ORDER BY created_at DESC
            "
        )
        .fetch_all(&self.pool)
        .await?;

    rows.into_iter().map(Self::row_to_bead).collect()
  }

  /// List beads with filtering (SQL injection-safe using parameterized queries)
  ///
  /// # Errors
  /// - Returns `DbError::Connection` if query execution fails
  /// - Returns `DbError::InvalidUuid` if ID parsing fails
  /// - Returns `DbError::Validation` if status/type parsing fails
  pub async fn list_beads_filtered(&self, filters: &BeadFilters) -> DbResult<Vec<Bead>> {
    // Build query with parameterized bindings (SQL injection safe)
    let mut query = String::from(
            "SELECT id, title, description, status, priority, bead_type, created_by, created_at, updated_at
             FROM beads WHERE 1=1",
        );
    let mut bind_count = 0;

    // Add status filter
    if filters.status.is_some() {
      bind_count += 1;
      query.push_str(&format!(" AND status = ?{bind_count}"));
    }

    // Add bead_type filter
    if filters.bead_type.is_some() {
      bind_count += 1;
      query.push_str(&format!(" AND bead_type = ?{bind_count}"));
    }

    // Add priority filter
    if filters.priority.is_some() {
      bind_count += 1;
      query.push_str(&format!(" AND priority = ?{bind_count}"));
    }

    // Add search filter (parameterized LIKE query)
    if filters.search.is_some() {
      bind_count += 1;
      query.push_str(&format!(
        " AND (title LIKE ?{bind_count} OR description LIKE ?{bind_count})"
      ));
    }

    query.push_str(" ORDER BY created_at DESC");

    // Build and execute parameterized query
    let mut sql_query = sqlx::query(&query);

    // Bind parameters in order
    if let Some(ref status) = filters.status {
      sql_query = sql_query.bind(status);
    }
    if let Some(ref bead_type) = filters.bead_type {
      sql_query = sql_query.bind(bead_type);
    }
    if let Some(priority) = filters.priority {
      sql_query = sql_query.bind(priority);
    }
    if let Some(ref search) = filters.search {
      // Use parameterized search pattern (SQL injection safe)
      // We need to create a pattern that lives long enough
      let pattern = format!("%{search}%");
      sql_query = sql_query.bind(pattern);
    }

    let rows = sql_query.fetch_all(&self.pool).await?;

    rows.into_iter().map(Self::row_to_bead).collect()
  }

  /// Get a single bead by ID
  ///
  /// # Errors
  /// - Returns `DbError::NotFound` if bead does not exist
  /// - Returns `DbError::Connection` if query execution fails
  /// - Returns `DbError::InvalidUuid` if ID parsing fails
  /// - Returns `DbError::Validation` if status/type parsing fails
  pub async fn get_bead(&self, id: BeadId) -> DbResult<Bead> {
    let row = sqlx::query(
            r"
            SELECT id, title, description, status, priority, bead_type, created_by, created_at, updated_at
            FROM beads
            WHERE id = ?
            "
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| DbError::not_found("Bead", id.to_string()))?;

    Self::row_to_bead(row)
  }

  /// Create a new bead
  ///
  /// # Errors
  /// - Returns `DbError::Connection` if query execution fails
  /// - Returns `DbError::InvalidUuid` if ID parsing fails
  /// - Returns `DbError::Validation` if status/type parsing fails
  pub async fn create_bead(&self, bead: NewBead) -> DbResult<Bead> {
    let id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now();

    sqlx::query(
            r"
            INSERT INTO beads (id, title, description, status, priority, bead_type, created_by, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "
        )
        .bind(id.to_string())
        .bind(&bead.title)
        .bind(&bead.description)
        .bind(bead.status.as_str())
        .bind(bead.priority.0)
        .bind(bead.bead_type.as_str())
        .bind(bead.created_by.map(|u| u.to_string()))
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await?;

    self.get_bead(BeadId::from(id)).await
  }

  /// Update an existing bead
  ///
  /// # Errors
  /// - Returns `DbError::NotFound` if bead does not exist
  /// - Returns `DbError::Connection` if query execution fails
  /// - Returns `DbError::InvalidUuid` if ID parsing fails
  /// - Returns `DbError::Validation` if status/type parsing fails
  pub async fn update_bead(&self, id: BeadId, bead: NewBead) -> DbResult<Bead> {
    let now = chrono::Utc::now();

    let result = sqlx::query(
            r"
            UPDATE beads
            SET title = ?, description = ?, status = ?, priority = ?, bead_type = ?, created_by = ?, updated_at = ?
            WHERE id = ?
            "
        )
        .bind(&bead.title)
        .bind(&bead.description)
        .bind(bead.status.as_str())
        .bind(bead.priority.0)
        .bind(bead.bead_type.as_str())
        .bind(bead.created_by.map(|u| u.to_string()))
        .bind(now.to_rfc3339())
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;

    if result.rows_affected() == 0 {
      return Err(DbError::not_found("Bead", id.to_string()));
    }

    self.get_bead(id).await
  }

  /// Delete a bead
  ///
  /// # Errors
  /// - Returns `DbError::NotFound` if bead does not exist
  /// - Returns `DbError::Connection` if query execution fails
  pub async fn delete_bead(&self, id: BeadId) -> DbResult<()> {
    let result = sqlx::query("DELETE FROM beads WHERE id = ?")
      .bind(id.to_string())
      .execute(&self.pool)
      .await?;

    if result.rows_affected() == 0 {
      return Err(DbError::not_found("Bead", id.to_string()));
    }

    Ok(())
  }

  /// Create an automatic backup before destructive operations
  ///
  /// This is a convenience wrapper around the backup module's `auto_backup`
  /// function. It creates a timestamped backup and applies retention policy.
  ///
  /// # Errors
  /// - Returns error if backup creation fails
  pub async fn create_auto_backup(&self) -> Result<PathBuf> {
    use crate::backup::{auto_backup, BackupOptions};

    let options = BackupOptions::default();
    auto_backup(&self.db_path, &options)
      .await
      .map_err(|e| anyhow::anyhow!("Backup creation failed: {e}"))
  }

  // ========== Synchronous Wrappers for Bead Operations ==========
  //
  // These methods provide blocking versions of the async bead operations.
  // They're used by UI components that can't use async directly.
  //
  // Each method uses the current tokio runtime if available, or creates
  // a new one if needed.

  /// List all beads (blocking version)
  ///
  /// # Errors
  /// - Returns `DbError::Connection` if query execution fails
  /// - Returns `DbError::InvalidUuid` if ID parsing fails
  /// - Returns `DbError::Validation` if status/type parsing fails
  pub fn list_beads_sync(&self) -> DbResult<Vec<Bead>> {
    let rt = tokio::runtime::Handle::try_current()
      .or_else(|_| tokio::runtime::Runtime::new().map(|rt| rt.handle().clone()))
      .map_err(|e| DbError::Connection(e.into()))?;

    rt.block_on(self.list_beads())
  }

  /// List beads with filtering (blocking version)
  ///
  /// # Errors
  /// - Returns `DbError::Connection` if query execution fails
  /// - Returns `DbError::InvalidUuid` if ID parsing fails
  /// - Returns `DbError::Validation` if status/type parsing fails
  pub fn list_beads_filtered_sync(&self, filters: &BeadFilters) -> DbResult<Vec<Bead>> {
    let rt = tokio::runtime::Handle::try_current()
      .or_else(|_| tokio::runtime::Runtime::new().map(|rt| rt.handle().clone()))
      .map_err(|e| DbError::Connection(e.into()))?;

    rt.block_on(self.list_beads_filtered(filters))
  }

  /// Get a single bead by ID (blocking version)
  ///
  /// # Errors
  /// - Returns `DbError::NotFound` if bead does not exist
  /// - Returns `DbError::Connection` if query execution fails
  /// - Returns `DbError::InvalidUuid` if ID parsing fails
  /// - Returns `DbError::Validation` if status/type parsing fails
  pub fn get_bead_sync(&self, id: BeadId) -> DbResult<Bead> {
    let rt = tokio::runtime::Handle::try_current()
      .or_else(|_| tokio::runtime::Runtime::new().map(|rt| rt.handle().clone()))
      .map_err(|e| DbError::Connection(e.into()))?;

    rt.block_on(self.get_bead(id))
  }

  /// Create a new bead (blocking version)
  ///
  /// # Errors
  /// - Returns `DbError::Connection` if query execution fails
  /// - Returns `DbError::InvalidUuid` if ID parsing fails
  /// - Returns `DbError::Validation` if status/type parsing fails
  pub fn create_bead_sync(&self, bead: NewBead) -> DbResult<Bead> {
    let rt = tokio::runtime::Handle::try_current()
      .or_else(|_| tokio::runtime::Runtime::new().map(|rt| rt.handle().clone()))
      .map_err(|e| DbError::Connection(e.into()))?;

    rt.block_on(self.create_bead(bead))
  }

  /// Update an existing bead (blocking version)
  ///
  /// # Errors
  /// - Returns `DbError::NotFound` if bead does not exist
  /// - Returns `DbError::Connection` if query execution fails
  /// - Returns `DbError::InvalidUuid` if ID parsing fails
  /// - Returns `DbError::Validation` if status/type parsing fails
  pub fn update_bead_sync(&self, id: BeadId, bead: NewBead) -> DbResult<Bead> {
    let rt = tokio::runtime::Handle::try_current()
      .or_else(|_| tokio::runtime::Runtime::new().map(|rt| rt.handle().clone()))
      .map_err(|e| DbError::Connection(e.into()))?;

    rt.block_on(self.update_bead(id, bead))
  }

  /// Delete a bead (blocking version)
  ///
  /// # Errors
  /// - Returns `DbError::NotFound` if bead does not exist
  /// - Returns `DbError::Connection` if query execution fails
  pub fn delete_bead_sync(&self, id: BeadId) -> DbResult<()> {
    let rt = tokio::runtime::Handle::try_current()
      .or_else(|_| tokio::runtime::Runtime::new().map(|rt| rt.handle().clone()))
      .map_err(|e| DbError::Connection(e.into()))?;

    rt.block_on(self.delete_bead(id))
  }

  /// Helper: Convert a query row to Bead
  ///
  /// # Errors
  /// - Returns `DbError::InvalidUuid` if ID parsing fails
  /// - Returns `DbError::Validation` if status/type parsing fails
  fn row_to_bead(row: sqlx::sqlite::SqliteRow) -> DbResult<Bead> {
    let id_str: String = row
      .try_get("id")
      .map_err(DbError::Connection)?;
    let title: String = row
      .try_get("title")
      .map_err(DbError::Connection)?;
    let description: Option<String> = row
      .try_get("description")
      .map_err(DbError::Connection)?;
    let status_str: String = row
      .try_get("status")
      .map_err(DbError::Connection)?;
    let priority_val: i16 = row
      .try_get("priority")
      .map_err(DbError::Connection)?;
    let type_str: String = row
      .try_get("bead_type")
      .map_err(DbError::Connection)?;
    let created_by_str: Option<String> = row
      .try_get("created_by")
      .map_err(DbError::Connection)?;
    let created_at_str: String = row
      .try_get("created_at")
      .map_err(DbError::Connection)?;
    let updated_at_str: String = row
      .try_get("updated_at")
      .map_err(DbError::Connection)?;

    let id = BeadId::from_str(&id_str)?;
    let status = status_str.parse()?;
    let bead_type = type_str.parse()?;
    let priority = clarity_core::db::models::BeadPriority::new(priority_val)?;

    let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
      .map_err(|e| DbError::Validation(format!("Invalid created_at format: {e}")))?
      .with_timezone(&chrono::Utc);
    let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
      .map_err(|e| DbError::Validation(format!("Invalid updated_at format: {e}")))?
      .with_timezone(&chrono::Utc);

    let created_by = created_by_str
      .map(|s| clarity_core::db::models::UserId::from_str(&s))
      .transpose()?;

    Ok(Bead {
      id,
      title,
      description,
      status,
      priority,
      bead_type,
      created_by,
      created_at,
      updated_at,
    })
  }

  // ===== User Authentication Methods =====

  /// Create a new user with a hashed password
  ///
  /// # Errors
  /// - Returns `DbError::Connection` if query execution fails
  /// - Returns `DbError::Validation` if email is invalid
  pub async fn create_user(&self, email: Email, password: &str, role: UserRole) -> DbResult<User> {
    // Hash the password using Argon2id
    let password_hash = auth::hash_password(password)
      .map_err(|e| DbError::Validation(format!("Password hashing failed: {e}")))?;

    let id = UserId::new();
    let now = chrono::Utc::now();

    sqlx::query(
      r"
            INSERT INTO users (id, email, password_hash, role, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ",
    )
    .bind(id.to_string())
    .bind(email.as_str())
    .bind(&password_hash)
    .bind(role.to_string())
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(&self.pool)
    .await
    .map_err(|e| {
      // Check for unique constraint violation
      if e.to_string().contains("UNIQUE constraint failed") {
        DbError::Validation(format!("Email {email} already exists"))
      } else {
        DbError::Connection(e)
      }
    })?;

    self.get_user(id).await
  }

  /// Get a user by ID
  ///
  /// # Errors
  /// - Returns `DbError::NotFound` if user does not exist
  /// - Returns `DbError::Connection` if query execution fails
  pub async fn get_user(&self, id: UserId) -> DbResult<User> {
    let row = sqlx::query(
      r"
            SELECT id, email, password_hash, role, created_at, updated_at
            FROM users
            WHERE id = ?
            ",
    )
    .bind(id.to_string())
    .fetch_optional(&self.pool)
    .await?
    .ok_or_else(|| DbError::not_found("User", id.to_string()))?;

    Self::row_to_user(row)
  }

  /// Get a user by email
  ///
  /// # Errors
  /// - Returns `DbError::NotFound` if user does not exist
  /// - Returns `DbError::Connection` if query execution fails
  pub async fn get_user_by_email(&self, email: &Email) -> DbResult<User> {
    let row = sqlx::query(
      r"
            SELECT id, email, password_hash, role, created_at, updated_at
            FROM users
            WHERE email = ?
            ",
    )
    .bind(email.as_str())
    .fetch_optional(&self.pool)
    .await?
    .ok_or_else(|| DbError::not_found("User", email.to_string()))?;

    Self::row_to_user(row)
  }

  /// Verify user credentials (email + password)
  ///
  /// Returns the user if credentials are valid.
  ///
  /// # Errors
  /// - Returns `DbError::NotFound` if user doesn't exist
  /// - Returns `DbError::Validation` if password is incorrect
  pub async fn verify_user(&self, email: &Email, password: &str) -> DbResult<User> {
    // Get the user by email
    let user = self.get_user_by_email(email).await?;

    // Verify the password
    let is_valid = auth::verify_password(&user.password_hash, password)
      .map_err(|e| DbError::Validation(format!("Password verification failed: {e}")))?;

    if is_valid {
      Ok(user)
    } else {
      Err(DbError::Validation("Invalid password".to_string()))
    }
  }

  /// List all users
  ///
  /// # Errors
  /// - Returns `DbError::Connection` if query execution fails
  pub async fn list_users(&self) -> DbResult<Vec<User>> {
    let rows = sqlx::query(
      r"
            SELECT id, email, password_hash, role, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            ",
    )
    .fetch_all(&self.pool)
    .await?;

    rows.into_iter().map(Self::row_to_user).collect()
  }

  /// Update user password
  ///
  /// # Errors
  /// - Returns `DbError::NotFound` if user doesn't exist
  /// - Returns `DbError::Connection` if query execution fails
  pub async fn update_user_password(&self, id: UserId, new_password: &str) -> DbResult<()> {
    let password_hash = auth::hash_password(new_password)
      .map_err(|e| DbError::Validation(format!("Password hashing failed: {e}")))?;

    let now = chrono::Utc::now();

    let result = sqlx::query(
      r"
            UPDATE users
            SET password_hash = ?, updated_at = ?
            WHERE id = ?
            ",
    )
    .bind(&password_hash)
    .bind(now.to_rfc3339())
    .bind(id.to_string())
    .execute(&self.pool)
    .await?;

    if result.rows_affected() == 0 {
      return Err(DbError::not_found("User", id.to_string()));
    }

    Ok(())
  }

  /// Delete a user
  ///
  /// # Errors
  /// - Returns `DbError::NotFound` if user doesn't exist
  /// - Returns `DbError::Connection` if query execution fails
  pub async fn delete_user(&self, id: UserId) -> DbResult<()> {
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
      .bind(id.to_string())
      .execute(&self.pool)
      .await?;

    if result.rows_affected() == 0 {
      return Err(DbError::not_found("User", id.to_string()));
    }

    Ok(())
  }

  /// Helper: Convert a query row to User
  ///
  /// # Errors
  /// - Returns `DbError::InvalidUuid` if ID parsing fails
  /// - Returns `DbError::Validation` if email or role parsing fails
  fn row_to_user(row: sqlx::sqlite::SqliteRow) -> DbResult<User> {
    let id_str: String = row
      .try_get("id")
      .map_err(DbError::Connection)?;
    let email_str: String = row
      .try_get("email")
      .map_err(DbError::Connection)?;
    let password_hash: String = row
      .try_get("password_hash")
      .map_err(DbError::Connection)?;
    let role_str: String = row
      .try_get("role")
      .map_err(DbError::Connection)?;
    let created_at_str: String = row
      .try_get("created_at")
      .map_err(DbError::Connection)?;
    let updated_at_str: String = row
      .try_get("updated_at")
      .map_err(DbError::Connection)?;

    let id = UserId::from_str(&id_str)?;
    let email = Email::new(email_str)?;
    let role = role_str.parse()?;

    let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
      .map_err(|e| DbError::Validation(format!("Invalid created_at format: {e}")))?
      .with_timezone(&chrono::Utc);
    let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
      .map_err(|e| DbError::Validation(format!("Invalid updated_at format: {e}")))?
      .with_timezone(&chrono::Utc);

    Ok(User {
      id,
      email,
      password_hash,
      role,
      created_at,
      updated_at,
    })
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::expect_used)]
  #![allow(clippy::unwrap_used)]
  use super::*;
  use clarity_core::db::models::{BeadPriority, BeadStatus, BeadType};

  #[tokio::test]
  async fn test_create_in_memory_db() {
    let db = DesktopDb::in_memory()
      .await
      .expect("Failed to create in-memory database");

    // Verify we can list beads (should be empty)
    let beads = db.list_beads().await.expect("Failed to list beads");
    assert!(beads.is_empty(), "New database should have no beads");
  }

  #[tokio::test]
  async fn test_create_and_list_beads() {
    let db = DesktopDb::in_memory()
      .await
      .expect("Failed to create in-memory database");

    // Create a bead
    let new_bead = NewBead {
      title: "Test Bead".to_string(),
      description: Some("Test Description".to_string()),
      status: BeadStatus::Open,
      priority: BeadPriority::HIGH,
      bead_type: BeadType::Feature,
      created_by: None,
    };

    let created = db
      .create_bead(new_bead.clone())
      .await
      .expect("Failed to create bead");

    // Verify bead was created
    assert_eq!(created.title, "Test Bead");
    assert_eq!(created.description, Some("Test Description".to_string()));
    assert_eq!(created.status, BeadStatus::Open);

    // List beads
    let beads = db.list_beads().await.expect("Failed to list beads");
    assert_eq!(beads.len(), 1);
    assert_eq!(beads[0].id, created.id);
  }

  #[tokio::test]
  async fn test_get_bead() {
    let db = DesktopDb::in_memory()
      .await
      .expect("Failed to create in-memory database");

    // Create a bead
    let new_bead = NewBead {
      title: "Test Bead".to_string(),
      description: Some("Test Description".to_string()),
      status: BeadStatus::Open,
      priority: BeadPriority::MEDIUM,
      bead_type: BeadType::Bugfix,
      created_by: None,
    };

    let created = db
      .create_bead(new_bead)
      .await
      .expect("Failed to create bead");

    // Get the bead
    let retrieved = db.get_bead(created.id).await.expect("Failed to get bead");

    assert_eq!(retrieved.id, created.id);
    assert_eq!(retrieved.title, "Test Bead");
    assert_eq!(retrieved.status, BeadStatus::Open);
  }

  #[tokio::test]
  async fn test_get_nonexistent_bead() {
    let db = DesktopDb::in_memory()
      .await
      .expect("Failed to create in-memory database");

    let fake_id = BeadId::from(uuid::Uuid::new_v4());
    let result = db.get_bead(fake_id).await;

    assert!(result.is_err(), "Getting nonexistent bead should fail");
    match result {
      Err(DbError::NotFound { entity, id }) => {
        assert_eq!(entity, "Bead");
        assert_eq!(id, fake_id.to_string());
      }
      _ => panic!("Expected NotFound error"),
    }
  }

  #[tokio::test]
  async fn test_update_bead() {
    let db = DesktopDb::in_memory()
      .await
      .expect("Failed to create in-memory database");

    // Create a bead
    let new_bead = NewBead {
      title: "Original Title".to_string(),
      description: Some("Original Description".to_string()),
      status: BeadStatus::Open,
      priority: BeadPriority::LOW,
      bead_type: BeadType::Docs,
      created_by: None,
    };

    let created = db
      .create_bead(new_bead)
      .await
      .expect("Failed to create bead");

    // Update the bead
    let updated_bead = NewBead {
      title: "Updated Title".to_string(),
      description: Some("Updated Description".to_string()),
      status: BeadStatus::Closed,
      priority: BeadPriority::HIGH,
      bead_type: BeadType::Feature,
      created_by: None,
    };

    let updated = db
      .update_bead(created.id, updated_bead)
      .await
      .expect("Failed to update bead");

    assert_eq!(updated.id, created.id);
    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.status, BeadStatus::Closed);

    // Verify update persisted
    let retrieved = db.get_bead(created.id).await.expect("Failed to get bead");
    assert_eq!(retrieved.title, "Updated Title");
    assert_eq!(retrieved.status, BeadStatus::Closed);
  }

  #[tokio::test]
  async fn test_delete_bead() {
    let db = DesktopDb::in_memory()
      .await
      .expect("Failed to create in-memory database");

    // Create a bead
    let new_bead = NewBead {
      title: "Test Bead".to_string(),
      description: None,
      status: BeadStatus::Open,
      priority: BeadPriority::MEDIUM,
      bead_type: BeadType::Refactor,
      created_by: None,
    };

    let created = db
      .create_bead(new_bead)
      .await
      .expect("Failed to create bead");

    // Delete the bead
    db.delete_bead(created.id)
      .await
      .expect("Failed to delete bead");

    // Verify bead is gone
    let result = db.get_bead(created.id).await;
    assert!(result.is_err(), "Deleted bead should not be found");

    // Verify list is empty
    let beads = db.list_beads().await.expect("Failed to list beads");
    assert!(beads.is_empty());
  }

  #[tokio::test]
  async fn test_filter_by_status() {
    let db = DesktopDb::in_memory()
      .await
      .expect("Failed to create in-memory database");

    // Create beads with different statuses
    let open_bead = NewBead {
      title: "Open Bead".to_string(),
      description: None,
      status: BeadStatus::Open,
      priority: BeadPriority::MEDIUM,
      bead_type: BeadType::Feature,
      created_by: None,
    };

    let closed_bead = NewBead {
      title: "Closed Bead".to_string(),
      description: None,
      status: BeadStatus::Closed,
      priority: BeadPriority::MEDIUM,
      bead_type: BeadType::Feature,
      created_by: None,
    };

    db.create_bead(open_bead)
      .await
      .expect("Failed to create bead");
    db.create_bead(closed_bead)
      .await
      .expect("Failed to create bead");

    // Filter by status
    let filters = BeadFilters {
      status: Some("open".to_string()),
      bead_type: None,
      priority: None,
      created_by: None,
      search: None,
    };

    let results = db
      .list_beads_filtered(&filters)
      .await
      .expect("Failed to filter beads");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Open Bead");
    assert_eq!(results[0].status, BeadStatus::Open);
  }

  #[tokio::test]
  async fn test_filter_by_search() {
    let db = DesktopDb::in_memory()
      .await
      .expect("Failed to create in-memory database");

    // Create beads
    let bug_bead = NewBead {
      title: "Fix critical bug".to_string(),
      description: Some("This is a bugfix".to_string()),
      status: BeadStatus::Open,
      priority: BeadPriority::HIGH,
      bead_type: BeadType::Bugfix,
      created_by: None,
    };

    let feature_bead = NewBead {
      title: "Add new feature".to_string(),
      description: Some("Feature description".to_string()),
      status: BeadStatus::Open,
      priority: BeadPriority::MEDIUM,
      bead_type: BeadType::Feature,
      created_by: None,
    };

    db.create_bead(bug_bead)
      .await
      .expect("Failed to create bead");
    db.create_bead(feature_bead)
      .await
      .expect("Failed to create bead");

    // Search for "bug" - should match title and description
    let filters = BeadFilters {
      status: None,
      bead_type: None,
      priority: None,
      created_by: None,
      search: Some("bug".to_string()),
    };

    let results = db
      .list_beads_filtered(&filters)
      .await
      .expect("Failed to filter beads");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Fix critical bug");
  }

  #[tokio::test]
  async fn test_sql_injection_prevention() {
    let db = DesktopDb::in_memory()
      .await
      .expect("Failed to create in-memory database");

    // Create a normal bead
    let new_bead = NewBead {
      title: "Normal Bead".to_string(),
      description: None,
      status: BeadStatus::Open,
      priority: BeadPriority::MEDIUM,
      bead_type: BeadType::Feature,
      created_by: None,
    };

    db.create_bead(new_bead)
      .await
      .expect("Failed to create bead");

    // Try SQL injection in search parameter
    let malicious_search = "'; DROP TABLE beads; --";

    let filters = BeadFilters {
      status: None,
      bead_type: None,
      priority: None,
      created_by: None,
      search: Some(malicious_search.to_string()),
    };

    // This should not cause an error, just return no results
    let results = db
      .list_beads_filtered(&filters)
      .await
      .expect("SQL injection attempt should not cause error");

    // Should return empty results (search for malicious string won't match)
    assert_eq!(results.len(), 0);

    // Verify table still exists by listing all beads
    let all_beads = db.list_beads().await.expect("Failed to list beads");
    assert_eq!(
      all_beads.len(),
      1,
      "Table should still exist after SQL injection attempt"
    );
  }
}
