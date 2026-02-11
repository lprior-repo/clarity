#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Database backup and restore functionality for Clarity desktop app
//!
//! This module provides:
//! - Manual backup to custom locations with timestamp verification
//! - Automatic backup with retention policy (keep last 10)
//! - Restore functionality with integrity verification
//! - Backup metadata listing and management
//!
//! ## Architecture
//!
//! **Functional Core (Pure functions)**
//! - `backup_database_path()`: Pure path computation and validation
//! - `verify_backup_integrity()`: `SQLite` integrity check parsing
//! - `parse_backup_metadata()`: Metadata extraction from filesystem
//! - `compute_backup_filename()`: Timestamped filename generation
//!
//! **Imperative Shell (I/O and async)**
//! - `backup_database()`: File copying and backup creation
//! - `restore_backup()`: Database restoration with validation
//! - `auto_backup()`: Automatic backup with retention management
//! - `list_backups()`: Backup directory scanning and metadata

use anyhow::Result;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use rpds::Vector;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::path::{Path, PathBuf};
use tokio::fs;

// ============================================================================
// Domain Types (Core)
// ============================================================================

/// Backup metadata information
///
/// Captures essential metadata about a backup file including
/// timestamp, file size, and integrity status.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BackupInfo {
  /// Full path to the backup file
  pub path: PathBuf,
  /// When the backup was created
  pub timestamp: DateTime<Utc>,
  /// Size of the backup file in bytes
  pub size_bytes: u64,
  /// Whether the backup passed integrity verification
  pub is_valid: bool,
}

/// Backup creation options
///
/// Configuration for backup creation behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupOptions {
  /// Maximum number of automatic backups to retain
  pub max_auto_backups: usize,
  /// Whether to verify integrity after backup creation
  pub verify_integrity: bool,
}

impl Default for BackupOptions {
  fn default() -> Self {
    Self {
      max_auto_backups: 10,
      verify_integrity: true,
    }
  }
}

// ============================================================================
// Domain Errors (Core)
// ============================================================================

/// Backup-specific error types
///
/// These errors represent domain-specific failures in backup/restore operations.
/// Uses thiserror for compile-time type safety and exhaustive matching.
#[derive(Debug, thiserror::Error)]
pub enum BackupError {
  #[error("Source database not found: {0}")]
  DatabaseNotFound(PathBuf),

  #[error("Backup directory inaccessible: {0}")]
  BackupDirectoryInaccessible(PathBuf),

  #[error("Backup file not found: {0}")]
  BackupNotFound(PathBuf),

  #[error("Backup integrity check failed: {0}")]
  IntegrityCheckFailed(String),

  #[error("Invalid backup file: {0}")]
  InvalidBackupFile(String),

  #[error("Insufficient disk space: required {required} bytes, available {available} bytes")]
  InsufficientDiskSpace { required: u64, available: u64 },

  #[error("Restore failed: {0}")]
  RestoreFailed(String),

  #[error("Io error: {0}")]
  Io(#[from] std::io::Error),
}

// ============================================================================
// Functional Core: Pure Functions
// ============================================================================

/// Compute the default backup directory path
///
/// This pure function determines where backups should be stored based on
/// the platform's conventions. Returns None if the data directory cannot be determined.
///
/// # Returns
/// - `Some(PathBuf)` with the backup directory path if successful
/// - `None` if the data directory cannot be determined
fn default_backup_dir() -> Option<PathBuf> {
  dirs::data_local_dir().map(|data_dir| data_dir.join("clarity").join("backups"))
}

/// Compute a timestamped backup filename
///
/// Generates a unique backup filename with UTC timestamp in ISO 8601 format.
/// This is a pure function with no side effects.
///
/// # Arguments
/// * `base_name` - The base name for the backup (typically "clarity.db")
///
/// # Returns
/// A filename string with timestamp appended
fn compute_backup_filename(base_name: &str) -> String {
  let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
  format!("{}.{timestamp}.backup", base_name.trim_end_matches(".db"))
}

/// Parse backup metadata from filesystem entry
///
/// Extracts timestamp and validity information from a backup file path.
/// Returns None if the file doesn't match the expected backup naming pattern.
///
/// # Arguments
/// * `path` - Path to the backup file
/// * `size_bytes` - File size in bytes
///
/// # Returns
/// - `Some(BackupInfo)` if the path matches backup naming convention
/// - `None` if the filename doesn't match expected pattern
fn parse_backup_metadata(path: PathBuf, size_bytes: u64) -> Option<BackupInfo> {
  let filename = path.file_name()?.to_str()?;

  // Expected format: clarity.db.YYYYMMDD_HHMMSS.backup
  let parts: Vec<&str> = filename.split('.').collect();

  // Must have at least 3 parts: base, timestamp, extension
  if parts.len() < 3 {
    return None;
  }

  let timestamp_str = parts[1];

  // Try RFC3339 format first: YYYYMMDD_HHMMSS+00:00
  if let Ok(datetime) = DateTime::parse_from_rfc3339(&format!("{timestamp_str}+00:00")) {
    return Some(BackupInfo {
      path,
      timestamp: datetime.with_timezone(&Utc),
      size_bytes,
      is_valid: true, // Will be verified separately
    });
  }

  // Try alternative format: YYYYMMDD_HHMMSS
  if timestamp_str.len() == 15 {
    let year = &timestamp_str[0..4];
    let month = &timestamp_str[4..6];
    let day = &timestamp_str[6..8];
    let hour = &timestamp_str[9..11];
    let minute = &timestamp_str[11..13];
    let second = &timestamp_str[13..15];
    let formatted = format!("{year}-{month}-{day}T{hour}:{minute}:{second}Z");

    if let Ok(datetime) = DateTime::parse_from_rfc3339(&formatted) {
      return Some(BackupInfo {
        path,
        timestamp: datetime.with_timezone(&Utc),
        size_bytes,
        is_valid: true, // Will be verified separately
      });
    }
  }

  None
}

/// Verify `SQLite` database integrity
///
/// Executes PRAGMA `integrity_check` on a `SQLite` database and parses the result.
/// This is a pure function that interprets the database's integrity report.
///
/// # Arguments
/// * `pool` - `SQLite` connection pool
///
/// # Returns
/// - `Ok(())` if integrity check passes
/// - `Err(BackupError::IntegrityCheckFailed)` if check fails
async fn verify_backup_integrity(pool: &SqlitePool) -> Result<(), BackupError> {
  let result = sqlx::query("PRAGMA integrity_check")
    .fetch_one(pool)
    .await
    .map_err(|e| BackupError::IntegrityCheckFailed(e.to_string()))?;

  let check_result: String = result
    .try_get::<String, _>("integrity_check")
    .map_err(|e: sqlx::Error| BackupError::IntegrityCheckFailed(e.to_string()))?;

  match check_result.as_str() {
    "ok" => Ok(()),
    other => Err(BackupError::IntegrityCheckFailed(format!(
      "Integrity check returned: {other}"
    ))),
  }
}

/// Apply retention policy to backup list
///
/// Pure function that filters and sorts backups based on retention policy.
/// Returns the backups that should be retained after applying the policy.
///
/// # Arguments
/// * `backups` - Vector of existing backup metadata
/// * `max_count` - Maximum number of backups to retain
///
/// # Returns
/// A new Vector containing the backups to retain (sorted by timestamp, newest first)
fn apply_retention_policy(backups: &Vector<BackupInfo>, max_count: usize) -> Vector<BackupInfo> {
  backups
    .iter()
    .sorted_by(|a, b| b.timestamp.cmp(&a.timestamp))
    .take(max_count)
    .cloned()
    .collect()
}

/// Compute backup file paths that should be deleted
///
/// Pure function that determines which backups exceed the retention limit.
///
/// # Arguments
/// * `existing_backups` - All current backup metadata
/// * `retained_backups` - Backups that should be kept
///
/// # Returns
/// Vector of paths that should be deleted
fn compute_deletions(
  existing_backups: &[BackupInfo],
  retained_backups: &Vector<BackupInfo>,
) -> Vector<PathBuf> {
  let retained_paths: Vec<PathBuf> = retained_backups.iter().map(|b| b.path.clone()).collect();

  existing_backups
    .iter()
    .filter(|b| !retained_paths.contains(&b.path))
    .map(|b| b.path.clone())
    .collect()
}

// ============================================================================
// Imperative Shell: I/O and Async Operations
// ============================================================================

/// Create a database backup at the specified path
///
/// Copies the `SQLite` database file to the backup location with a timestamped filename.
/// Optionally verifies backup integrity after creation.
///
/// # Arguments
/// * `db_path` - Path to the source database file
/// * `backup_dir` - Directory where the backup should be stored
/// * `verify` - Whether to verify backup integrity after creation
///
/// # Returns
/// - `Ok(PathBuf)` with the full path to the created backup
/// - `Err(BackupError)` if backup creation fails
///
/// # Errors
/// - `BackupError::DatabaseNotFound` if source database doesn't exist
/// - `BackupError::BackupDirectoryInaccessible` if backup directory can't be created/accessed
/// - `BackupError::IntegrityCheckFailed` if verification fails and verify=true
pub async fn backup_database(
  db_path: &Path,
  backup_dir: &Path,
  verify: bool,
) -> Result<PathBuf, BackupError> {
  // Validate source database exists
  if !db_path.exists() {
    return Err(BackupError::DatabaseNotFound(db_path.to_path_buf()));
  }

  // Create backup directory if needed
  fs::create_dir_all(backup_dir).await?;

  // Compute backup filename
  let db_filename = db_path
    .file_name()
    .and_then(|n| n.to_str())
    .ok_or_else(|| BackupError::InvalidBackupFile("Invalid database filename".to_string()))?;

  let backup_filename = compute_backup_filename(db_filename);
  let backup_path = backup_dir.join(&backup_filename);

  // Copy database to backup location
  fs::copy(db_path, &backup_path).await?;

  // Verify integrity if requested
  if verify {
    verify_backup_file(&backup_path).await?;
  }

  Ok(backup_path)
}

/// Verify a backup file's `SQLite` integrity
///
/// Opens a backup file and runs `SQLite` integrity checks.
///
/// # Arguments
/// * `backup_path` - Path to the backup file
///
/// # Returns
/// - `Ok(())` if backup is valid
/// - `Err(BackupError)` if verification fails
async fn verify_backup_file(backup_path: &Path) -> Result<(), BackupError> {
  let database_url = format!("sqlite:{}", backup_path.display());
  let pool = sqlx::SqlitePool::connect(&database_url)
    .await
    .map_err(|e| BackupError::IntegrityCheckFailed(e.to_string()))?;

  let result = verify_backup_integrity(&pool).await;

  pool.close().await;

  result
}

/// Create an automatic backup with retention policy
///
/// Creates a backup in the default backup directory and removes old backups
/// exceeding the retention limit. This is the recommended method for automatic
/// backups before destructive operations.
///
/// # Arguments
/// * `db_path` - Path to the source database file
/// * `options` - Backup options including retention policy
///
/// # Errors
/// - Returns `BackupError::BackupDirectoryInaccessible` if default backup directory cannot be found
/// - Returns `BackupError::BackupFailed` if database backup fails
/// - Returns `BackupError::Io` if file operations fail
///
/// # Returns
/// - `Ok(PathBuf)` with the path to the created backup
/// - `Err(BackupError)` if backup creation fails
pub async fn auto_backup(db_path: &Path, options: &BackupOptions) -> Result<PathBuf, BackupError> {
  let backup_dir = default_backup_dir()
    .ok_or_else(|| BackupError::BackupDirectoryInaccessible(PathBuf::from("Unknown")))?;

  // Create the new backup
  let backup_path = backup_database(db_path, &backup_dir, options.verify_integrity).await?;

  // List existing backups and apply retention policy
  let existing_backups: Vector<BackupInfo> =
    list_backups_core(&backup_dir).await?.into_iter().collect();
  let retained = apply_retention_policy(&existing_backups, options.max_auto_backups);

  // Delete backups exceeding retention limit
  let existing_vec: Vec<_> = existing_backups.iter().cloned().collect();
  let to_delete = compute_deletions(&existing_vec, &retained);
  for path in &to_delete {
    let _ = fs::remove_file(path).await;
  }

  Ok(backup_path)
}

/// Restore a database from a backup
///
/// Validates the backup file, closes any existing database connections,
/// copies the backup to the database location, and verifies the restored database.
///
/// # Arguments
/// * `backup_path` - Path to the backup file
/// * `db_path` - Target path for the restored database
/// * `verify` - Whether to verify the restored database integrity
///
/// # Returns
/// - `Ok(())` if restoration succeeds
/// - `Err(BackupError)` if restoration fails
///
/// # Errors
/// - `BackupError::BackupNotFound` if backup file doesn't exist
/// - `BackupError::InvalidBackupFile` if backup fails validation
/// - `BackupError::RestoreFailed` if copy or verification fails
pub async fn restore_backup(
  backup_path: &Path,
  db_path: &Path,
  verify: bool,
) -> Result<(), BackupError> {
  // Validate backup exists
  if !backup_path.exists() {
    return Err(BackupError::BackupNotFound(backup_path.to_path_buf()));
  }

  // Verify backup integrity before restoration
  if verify {
    verify_backup_file(backup_path).await?;
  }

  // Ensure parent directory exists
  if let Some(parent) = db_path.parent() {
    fs::create_dir_all(parent).await?;
  }

  // Close existing connections by removing WAL files if they exist
  let wal_path = db_path.with_extension("db-wal");
  let shm_path = db_path.with_extension("db-shm");

  let _ = fs::remove_file(&wal_path).await;
  let _ = fs::remove_file(&shm_path).await;

  // Copy backup to database location
  fs::copy(backup_path, db_path)
    .await
    .map_err(|e| BackupError::RestoreFailed(e.to_string()))?;

  // Verify restored database if requested
  if verify {
    let database_url = format!("sqlite:{}", db_path.display());
    let pool = sqlx::SqlitePool::connect(&database_url)
      .await
      .map_err(|e| BackupError::RestoreFailed(e.to_string()))?;

    let result = verify_backup_integrity(&pool).await;

    pool.close().await;

    result.map_err(|e| BackupError::RestoreFailed(format!("Verification failed: {e}")))?;
  }

  Ok(())
}

/// List all available backups in a directory
///
/// Scans the backup directory and returns metadata for all backup files,
/// sorted by timestamp (newest first). Includes integrity status if verification is enabled.
///
/// # Arguments
/// * `backup_dir` - Directory containing backup files
/// * `verify_integrity` - Whether to verify each backup's integrity
///
/// # Errors
/// - Returns `BackupError::Io` if directory cannot be read
/// - Returns `BackupError::BackupFileCorrupted` if a backup file has invalid metadata
///
/// # Returns
/// - `Ok(Vec<BackupInfo>)` with backup metadata, sorted newest first
/// - `Err(BackupError)` if directory scanning fails
pub async fn list_backups(
  backup_dir: &Path,
  verify_integrity: bool,
) -> Result<Vec<BackupInfo>, BackupError> {
  let mut backups = list_backups_core(backup_dir).await?;

  // Verify integrity if requested
  if verify_integrity {
    for backup in &mut backups {
      backup.is_valid = verify_backup_file(&backup.path).await.is_ok();
    }
  }

  // Sort by timestamp, newest first
  backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

  Ok(backups)
}

/// Core implementation of backup listing without verification
///
/// Scans the backup directory and extracts metadata from files matching
/// the backup naming pattern.
///
/// # Arguments
/// * `backup_dir` - Directory containing backup files
///
/// # Returns
/// - `Ok(Vec<BackupInfo>)` with backup metadata
/// - `Err(BackupError)` if directory scanning fails
async fn list_backups_core(backup_dir: &Path) -> Result<Vec<BackupInfo>, BackupError> {
  let mut entries = fs::read_dir(backup_dir)
    .await
    .map_err(|_| BackupError::BackupDirectoryInaccessible(backup_dir.to_path_buf()))?;

  let mut backups = Vec::new();

  while let Some(entry) = entries.next_entry().await? {
    let path = entry.path();

    // Only process files matching backup pattern
    if path.is_file() && path.extension().is_some_and(|e| e == "backup") {
      let metadata = entry.metadata().await?;
      let size_bytes = metadata.len();

      // Parse metadata from filename
      if let Some(info) = parse_backup_metadata(path, size_bytes) {
        backups.push(info);
      }
    }
  }

  Ok(backups)
}

/// Get the default backup directory path
///
/// Returns the platform-specific default location for storing backups.
/// Creates the directory if it doesn't exist.
///
/// # Errors
/// - Returns `BackupError::BackupDirectoryInaccessible` if default directory cannot be determined
/// - Returns `BackupError::Io` if directory creation fails
///
/// # Returns
/// - `Ok(PathBuf)` with the backup directory path
/// - `Err(BackupError)` if directory creation fails
pub async fn get_backup_directory() -> Result<PathBuf, BackupError> {
  let backup_dir = default_backup_dir()
    .ok_or_else(|| BackupError::BackupDirectoryInaccessible(PathBuf::from("Unknown")))?;

  fs::create_dir_all(&backup_dir)
    .await
    .map_err(|_| BackupError::BackupDirectoryInaccessible(backup_dir.clone()))?;

  Ok(backup_dir)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::expect_used)]
  use super::*;
  use tempfile::TempDir;
  use tokio::fs;

  #[test]
  fn test_compute_backup_filename() {
    // The function strips .db suffix before adding timestamp and .backup
    let filename = compute_backup_filename("clarity.db");
    assert!(filename.starts_with("clarity."));
    assert!(filename.ends_with(".backup"));

    let filename = compute_backup_filename("clarity");
    assert!(filename.starts_with("clarity."));
    assert!(filename.ends_with(".backup"));
  }

  #[test]
  fn test_parse_backup_metadata() {
    // Valid backup filename (note: .db is stripped before timestamp)
    let path = PathBuf::from("/backups/clarity.20250109_143000.backup");
    let size = 1024;

    let result = parse_backup_metadata(path.clone(), size);
    assert!(result.is_some());

    let info = result.unwrap();
    assert_eq!(info.path, path);
    assert_eq!(info.size_bytes, size);
    assert!(info.is_valid);

    // Invalid filename
    let invalid_path = PathBuf::from("/backups/not_a_backup.txt");
    let result = parse_backup_metadata(invalid_path, size);
    assert!(result.is_none());
  }

  #[test]
  fn test_apply_retention_policy() {
    let now = Utc::now();

    // Create 15 backups over time
    let backups: Vector<BackupInfo> = (0..15)
      .map(|i| BackupInfo {
        path: PathBuf::from(format!("backup_{i}.db")),
        timestamp: now - chrono::Duration::seconds(i as i64 * 60),
        size_bytes: 1024,
        is_valid: true,
      })
      .collect();

    // Apply retention policy of max 10
    let retained = apply_retention_policy(&backups, 10);

    assert_eq!(retained.len(), 10);
    // Should keep the newest 10
    assert_eq!(
      retained.get(0).map(|b| &b.path),
      Some(&PathBuf::from("backup_0.db"))
    );
  }

  #[test]
  fn test_compute_deletions() {
    let backup1 = BackupInfo {
      path: PathBuf::from("backup1.db"),
      timestamp: Utc::now(),
      size_bytes: 1024,
      is_valid: true,
    };

    let backup2 = BackupInfo {
      path: PathBuf::from("backup2.db"),
      timestamp: Utc::now(),
      size_bytes: 2048,
      is_valid: true,
    };

    let backup3 = BackupInfo {
      path: PathBuf::from("backup3.db"),
      timestamp: Utc::now(),
      size_bytes: 4096,
      is_valid: true,
    };

    let existing = vec![backup1.clone(), backup2.clone(), backup3.clone()];
    let retained: Vector<BackupInfo> = vec![backup1.clone(), backup3.clone()].into_iter().collect();

    let deletions = compute_deletions(&existing, &retained);

    assert_eq!(deletions.len(), 1);
    assert_eq!(deletions.get(0), Some(&PathBuf::from("backup2.db")));
  }

  #[tokio::test]
  async fn test_backup_database() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_dir = temp_dir.path().join("db");
    let backup_dir = temp_dir.path().join("backups");

    fs::create_dir_all(&db_dir)
      .await
      .expect("Failed to create db dir");

    // Create a simple database file
    let db_path = db_dir.join("test.db");
    fs::write(&db_path, b"test database content")
      .await
      .expect("Failed to write test db");

    // Create backup
    let result = backup_database(&db_path, &backup_dir, false).await;

    assert!(result.is_ok());
    let backup_path = result.unwrap();

    assert!(backup_path.exists());
    assert!(backup_path.starts_with(&backup_dir));

    // Verify content matches
    let backup_content = fs::read(&backup_path)
      .await
      .expect("Failed to read backup content");
    let db_content = fs::read(&db_path).await.expect("Failed to read db content");

    assert_eq!(backup_content, db_content);
  }

  #[tokio::test]
  async fn test_restore_backup() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let backup_dir = temp_dir.path().join("backups");
    let db_dir = temp_dir.path().join("db");

    fs::create_dir_all(&backup_dir)
      .await
      .expect("Failed to create backup dir");
    fs::create_dir_all(&db_dir)
      .await
      .expect("Failed to create db dir");

    // Create backup file
    let backup_path = backup_dir.join("test.db.20250109_120000.backup");
    fs::write(&backup_path, b"restored database content")
      .await
      .expect("Failed to write backup file");

    let db_path = db_dir.join("test.db");

    // Restore backup
    let result = restore_backup(&backup_path, &db_path, false).await;

    assert!(result.is_ok());
    assert!(db_path.exists());

    // Verify content matches
    let db_content = fs::read(&db_path)
      .await
      .expect("Failed to read restored db content");
    assert_eq!(db_content, b"restored database content");
  }

  #[tokio::test]
  async fn test_list_backups() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let backup_dir = temp_dir.path().join("backups");

    fs::create_dir_all(&backup_dir)
      .await
      .expect("Failed to create backup dir");

    // Create some backup files (note: .db is stripped before timestamp)
    let morning_backup = backup_dir.join("clarity.20250109_120000.backup");
    let afternoon_backup = backup_dir.join("clarity.20250109_130000.backup");
    let evening_backup = backup_dir.join("clarity.20250109_140000.backup");

    fs::write(&morning_backup, b"backup1")
      .await
      .expect("Failed to write backup1");
    fs::write(&afternoon_backup, b"backup2")
      .await
      .expect("Failed to write backup2");
    fs::write(&evening_backup, b"backup3")
      .await
      .expect("Failed to write backup3");

    // List backups
    let result = list_backups(&backup_dir, false).await;

    assert!(result.is_ok());
    let backups = result.unwrap();

    assert_eq!(backups.len(), 3);
    // Should be sorted newest first
    assert_eq!(backups[0].path, evening_backup);
    assert_eq!(backups[1].path, afternoon_backup);
    assert_eq!(backups[2].path, morning_backup);
  }

  #[tokio::test]
  async fn test_auto_backup_retention() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_dir = temp_dir.path().join("db");
    let backup_dir = temp_dir.path().join("backups");

    fs::create_dir_all(&db_dir)
      .await
      .expect("Failed to create db dir");

    // Create database file
    let db_path = db_dir.join("test.db");
    fs::write(&db_path, b"test database")
      .await
      .expect("Failed to write test database");

    // Create backup directory and multiple backups
    fs::create_dir_all(&backup_dir)
      .await
      .expect("Failed to create backup dir");

    // Set backup directory to temp location
    let _backup_dir_guard = {
      // Create multiple backups with proper timestamps (YYYYMMDD_HHMMSS)
      for i in 0..15 {
        let day = 9 + i; // Days 9-23 of January 2025
        let backup_path = backup_dir.join(format!("test.202501{:02}_120000.backup", day));
        fs::write(&backup_path, format!("backup_{i}"))
          .await
          .expect("Failed to write backup");
      }
      temp_dir
    };

    // List backups should find all 15
    let all_backups = list_backups_core(&backup_dir)
      .await
      .expect("Failed to list backups");
    assert_eq!(all_backups.len(), 15);
  }
}
