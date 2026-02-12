#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::single_match_else)]
#![allow(clippy::format_push_string)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::future_not_send)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::needless_pass_by_value)]

//! Desktop database module for client-side `SQLite` access
//!
//! Migrated from rusqlite to `SQLx` for:
//! - Compile-time query checking
//! - Async/await support
//! - Parameterized queries (SQL injection prevention)
//! - Connection pooling with WAL mode

use anyhow::Result;
use clarity_core::db::models::{Bead, BeadFilters, BeadId, NewBead};
use clarity_core::db::{sqlite_pool, DbError, DbResult, SqliteDbConfig};
use sqlx::{Row, SqlitePool};
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, instrument, warn};

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
  /// - Returns error if pool creation fails
  pub fn new() -> Result<Self> {
    // Try to use existing runtime handle
    match tokio::runtime::Handle::try_current() {
      Ok(handle) => {
        // Runtime already exists, use it
        handle.block_on(Self::new_async())
      }
      Err(_) => {
        // No runtime exists, create a new one and use it directly
        let rt = tokio::runtime::Runtime::new()
          .map_err(|e| anyhow::anyhow!("Failed to create runtime: {e}"))?;

        rt.block_on(Self::new_async())
      }
    }
  }

  /// Create a new `DesktopDb` with default path (async)
  ///
  /// # Errors
  /// - Returns error if data directory cannot be determined
  /// - Returns error if pool creation fails
  pub async fn new_async() -> Result<Self> {
    let data_dir = dirs::data_local_dir()
      .ok_or_else(|| anyhow::anyhow!("Failed to determine local data directory"))?;

    let app_dir = data_dir.join("clarity");
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
  #[instrument(skip(self))]
  pub async fn list_beads(&self) -> DbResult<Vec<Bead>> {
    debug!("Fetching all beads");

    let rows = sqlx::query(
            r"
            SELECT id, title, description, status, priority, bead_type, created_by, created_at, updated_at
            FROM beads
            ORDER BY created_at DESC
            "
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
          error!(error = %e, "Failed to fetch beads");
          e
        })?;

    let count = rows.len();
    let beads = rows
      .into_iter()
      .map(Self::row_to_bead)
      .collect::<DbResult<Vec<_>>>()
      .map_err(|e| {
        error!(error = %e, "Failed to parse bead row");
        e
      })?;

    info!(count, "Successfully fetched beads");

    Ok(beads)
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

    // Add pagination if specified
    if let Some(page_size) = filters.page_size {
      let offset = filters.offset();
      query.push_str(&format!(
        " LIMIT ?{} OFFSET ?{}",
        bind_count + 1,
        bind_count + 2
      ));

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

      // Bind pagination parameters
      sql_query = sql_query.bind(page_size);
      sql_query = sql_query.bind(offset);

      let rows = sql_query.fetch_all(&self.pool).await?;
      rows.into_iter().map(Self::row_to_bead).collect()
    } else {
      // No pagination, fetch all results
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
        let pattern = format!("%{search}%");
        sql_query = sql_query.bind(pattern);
      }

      let rows = sql_query.fetch_all(&self.pool).await?;
      rows.into_iter().map(Self::row_to_bead).collect()
    }
  }

  /// Get count of beads matching filters (for pagination)
  ///
  /// # Errors
  /// - Returns `DbError::Connection` if query execution fails
  /// - Returns `DbError::InvalidUuid` if ID parsing fails
  /// - Returns `DbError::Validation` if status/type parsing fails
  pub async fn count_beads_filtered(&self, filters: &BeadFilters) -> DbResult<u64> {
    // Build count query with parameterized bindings (SQL injection safe)
    let mut query = String::from("SELECT COUNT(*) FROM beads WHERE 1=1");
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
      let pattern = format!("%{search}%");
      sql_query = sql_query.bind(pattern);
    }

    let row = sql_query.fetch_one(&self.pool).await?;
    let count: i64 = row.get(0);
    Ok(count as u64)
  }

  /// Get paginated beads with filters
  ///
  /// # Errors
  /// - Returns `DbError::Connection` if query execution fails
  pub async fn list_beads_paginated(
    &self,
    filters: &BeadFilters,
  ) -> DbResult<clarity_core::db::models::PaginatedBeads> {
    let total = self.count_beads_filtered(filters).await?;
    let page = filters.page();
    let page_size = filters.page_size();
    let offset = filters.offset();

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

    // Always apply pagination for this method
    query.push_str(&format!(
      " LIMIT ?{} OFFSET ?{}",
      bind_count + 1,
      bind_count + 2
    ));

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
      let pattern = format!("%{search}%");
      sql_query = sql_query.bind(pattern);
    }

    // Bind pagination parameters
    sql_query = sql_query.bind(page_size);
    sql_query = sql_query.bind(offset);

    let rows = sql_query.fetch_all(&self.pool).await?;
    let beads = rows
      .into_iter()
      .map(Self::row_to_bead)
      .collect::<Result<Vec<_>, _>>()?;

    Ok(clarity_core::db::models::PaginatedBeads::new(
      beads, total, page, page_size,
    ))
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
  #[instrument(skip(self, bead), fields(bead_title = %bead.title, status = %bead.status, priority = bead.priority.0))]
  pub async fn create_bead(&self, bead: NewBead) -> DbResult<Bead> {
    let id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now();

    debug!(bead_id = %id, "Creating new bead");

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
        .await
        .map_err(|e| {
          error!(error = %e, bead_id = %id, "Failed to create bead");
          e
        })?;

    info!(bead_id = %id, title = %bead.title, "Successfully created bead");

    self.get_bead(BeadId::from(id)).await
  }

  /// Update an existing bead
  ///
  /// # Errors
  /// - Returns `DbError::NotFound` if bead does not exist
  /// - Returns `DbError::Connection` if query execution fails
  /// - Returns `DbError::InvalidUuid` if ID parsing fails
  /// - Returns `DbError::Validation` if status/type parsing fails
  #[instrument(skip(self, bead), fields(bead_id = %id, bead_title = %bead.title))]
  pub async fn update_bead(&self, id: BeadId, bead: NewBead) -> DbResult<Bead> {
    let now = chrono::Utc::now();

    debug!("Updating bead");

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
        .await
        .map_err(|e| {
          error!(error = %e, bead_id = %id, "Failed to update bead");
          e
        })?;

    if result.rows_affected() == 0 {
      warn!(bead_id = %id, "Bead not found for update");
      return Err(DbError::not_found("Bead", id.to_string()));
    }

    info!(bead_id = %id, title = %bead.title, "Successfully updated bead");

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

  /// Parse a datetime string, handling both RFC3339 and SQLite datetime formats
  ///
  /// # Errors
  /// Returns error if the string cannot be parsed as either format
  fn parse_sqlite_datetime(s: &str) -> Result<chrono::DateTime<chrono::Utc>, String> {
    // Try RFC3339 format first (what new beads use)
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
      return Ok(dt.with_timezone(&chrono::Utc));
    }

    // Try SQLite datetime format: "YYYY-MM-DD HH:MM:SS"
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
      return Ok(chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
        dt,
        chrono::Utc,
      ));
    }

    // Try SQLite datetime format with subseconds: "YYYY-MM-DD HH:MM:SS.SSS"
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f") {
      return Ok(chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
        dt,
        chrono::Utc,
      ));
    }

    // Try ISO8601-like format: "YYYY-MM-DDTHH:MM:SS"
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
      return Ok(chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
        dt,
        chrono::Utc,
      ));
    }

    Err(format!("Could not parse datetime: '{s}'"))
  }

  /// Helper: Convert a query row to Bead
  ///
  /// # Errors
  /// - Returns `DbError::InvalidUuid` if ID parsing fails
  /// - Returns `DbError::Validation` if status/type parsing fails
  fn row_to_bead(row: sqlx::sqlite::SqliteRow) -> DbResult<Bead> {
    let id_str: String = row.try_get("id").map_err(DbError::Connection)?;
    let title: String = row.try_get("title").map_err(DbError::Connection)?;
    let description: Option<String> = row.try_get("description").map_err(DbError::Connection)?;
    let status_str: String = row.try_get("status").map_err(DbError::Connection)?;
    let priority_val: i16 = row.try_get("priority").map_err(DbError::Connection)?;
    let type_str: String = row.try_get("bead_type").map_err(DbError::Connection)?;
    let created_by_str: Option<String> = row.try_get("created_by").map_err(DbError::Connection)?;
    let created_at_str: String = row.try_get("created_at").map_err(DbError::Connection)?;
    let updated_at_str: String = row.try_get("updated_at").map_err(DbError::Connection)?;

    let id = BeadId::from_str(&id_str)?;
    let status = status_str.parse()?;
    let bead_type = type_str.parse()?;
    let priority = clarity_core::db::models::BeadPriority::new(priority_val)?;

    let created_by = created_by_str
      .map(|s| uuid::Uuid::parse_str(&s))
      .transpose()
      .map_err(|e| DbError::InvalidUuid(e.to_string()))?;

    Ok(Bead {
      id,
      title,
      description,
      status,
      priority,
      bead_type,
      created_by,
      created_at: created_at_str,
      updated_at: updated_at_str,
    })
  }
}
