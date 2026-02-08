#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Bundled SQLite database with compile-time embedding
//!
//! This module provides a SQLite database that is embedded in the binary
//! using `include_bytes!()` and extracted to a cache directory at runtime.
//! The extraction is atomic and crash-safe.

use crate::db::error::{DbError, DbResult};
use std::path::PathBuf;
use std::sync::OnceLock;

/// Embedded SQLite database bytes
///
/// This includes the database file at compile time.
/// The database file will be created in assets/bundled.db
const BUNDLED_DB: &[u8] = include_bytes!("../../assets/bundled.db");

/// Cache directory for extracted database
static BUNDLED_DB_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Get the path to the extracted bundled database
///
/// This function ensures the database is extracted to the cache directory
/// before returning the path. The extraction is atomic and crash-safe.
/// Subsequent calls will return the cached path without re-extracting.
///
/// # Errors
/// - Returns `DbError::BundledDbExtraction` if atomic extraction fails
/// - Returns `DbError::BundledDbExtraction` if cache directory cannot be created
pub fn get_bundled_db_path() -> DbResult<PathBuf> {
  BUNDLED_DB_PATH
    .get_or_try_init(extract_database_atomically)
    .map(|path| path.clone())
}

/// Extract the embedded database to the cache directory atomically
///
/// This function writes the embedded database bytes to a temporary file
/// in the cache directory, then atomically renames it to the final location.
/// This ensures the extraction is crash-safe.
///
/// # Errors
/// - Returns `DbError::BundledDbExtraction` if directory creation fails
/// - Returns `DbError::BundledDbExtraction` if temp file write fails
/// - Returns `DbError::BundledDbExtraction` if atomic rename fails
fn extract_database_atomically() -> DbResult<PathBuf> {
  let cache_dir = get_cache_dir()?;
  std::fs::create_dir_all(&cache_dir).map_err(|e| {
    DbError::BundledDbExtraction(format!("Failed to create cache directory: {e}"))
  })?;

  let db_path = cache_dir.join("bundled.db");
  let temp_path = cache_dir.join("bundled.db.tmp");

  // Write to temp file first
  std::fs::write(&temp_path, BUNDLED_DB).map_err(|e| {
    DbError::BundledDbExtraction(format!("Failed to write temp database: {e}"))
  })?;

  // Atomic rename from temp to final
  std::fs::rename(&temp_path, &db_path).map_err(|e| {
    DbError::BundledDbExtraction(format!("Failed to rename temp database: {e}"))
  })?;

  Ok(db_path)
}

/// Get the cache directory for the bundled database
///
/// Returns the platform-appropriate cache directory:
/// - Linux: `$XDG_CACHE_HOME/clarity` or `~/.cache/clarity`
/// - macOS: `~/Library/Caches/clarity`
/// - Windows: `%LOCALAPPDATA%\\clarity\\cache`
fn get_cache_dir() -> DbResult<PathBuf> {
  dirs::cache_dir()
    .map(|p| p.join("clarity"))
    .ok_or_else(|| DbError::BundledDbExtraction(
      "Failed to determine cache directory".to_string()
    ))
}

#[cfg(test)]
mod tests {
  #[allow(clippy::disallowed_methods)]
  #[allow(clippy::unwrap_used)]
  #[allow(clippy::expect_used)]
  #[allow(clippy::panic)]
  use super::*;

  #[test]
  fn test_bundled_db_module_compiles() {
    // This test verifies the module compiles
  }

  #[test]
  fn test_bundled_db_extraction() -> DbResult<()> {
    let path = get_bundled_db_path()?;
    assert!(path.exists(), "Extracted database should exist");
    assert!(path.is_file(), "Path should be a file");
    Ok(())
  }

  #[test]
  fn test_bundled_db_idempotent() -> DbResult<()> {
    let path1 = get_bundled_db_path()?;
    let path2 = get_bundled_db_path()?;
    assert_eq!(path1, path2, "Should return same path on repeated calls");
    Ok(())
  }

  #[test]
  fn test_bundled_db_schema() -> DbResult<()> {
    let path = get_bundled_db_path()?;
    // Verify we can open the database
    let conn = rusqlite::Connection::open(&path).map_err(|e| {
      DbError::BundledDbConnection(format!("Failed to open database: {e}"))
    })?;

    // Verify schema exists (check for sqlite_master)
    let mut stmt = conn
      .prepare("SELECT name FROM sqlite_master WHERE type='table'")
      .map_err(|e| DbError::BundledDbConnection(format!("Query failed: {e}")))?;

    let tables: Vec<String> = stmt
      .query_map([], |row| row.get::<usize, String>(0))
      .map_err(|e| DbError::BundledDbConnection(format!("Query failed: {e}")))?
      .collect::<Result<_, _>>()
      .map_err(|e| DbError::BundledDbConnection(format!("Row parsing failed: {e}")))?;

    assert!(!tables.is_empty(), "Database should have tables");
    Ok(())
  }
}
