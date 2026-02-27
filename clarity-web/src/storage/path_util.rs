#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Project database path resolution utilities.
//!
//! Provides XDG-compliant path resolution for project storage.
//! All paths resolve to `~/.local/share/clarity/projects/{id}/data.redb`

use std::path::PathBuf;

use thiserror::Error;

/// Application name for XDG directory resolution
const APP_NAME: &str = "clarity";

/// Projects subdirectory name
const PROJECTS_DIR: &str = "projects";

/// Database filename
const DB_FILENAME: &str = "data.redb";

/// Directory permissions: owner read/write/execute only (0700)
const DIR_PERMISSIONS: u32 = 0o700;

/// Errors that can occur during path resolution operations
#[derive(Debug, Error)]
pub enum StorageError {
  /// XDG data directory could not be determined
  #[error("XDG data directory not found")]
  PathNotFound,

  /// I/O error during directory operations
  #[error("I/O error: {0}")]
  IoError(#[from] std::io::Error),

  /// Project ID is invalid (empty, contains path separators, etc.)
  #[error("invalid project ID: {0}")]
  InvalidProjectId(String),

  /// Database operation error
  #[error("database error: {0}")]
  Database(String),

  /// Serialization error (when converting data to JSON/storage format)
  #[error("serialization error: {0}")]
  Serialization(String),

  /// Deserialization error (when reading data from JSON/storage format)
  #[error("deserialization error: {0}")]
  Deserialization(String),
}

// Implement From for all redb error types to enable ? operator
// Only available on native (non-WASM) targets
#[cfg(not(target_arch = "wasm32"))]
impl From<redb::Error> for StorageError {
  fn from(err: redb::Error) -> Self {
    Self::Database(err.to_string())
  }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<redb::TransactionError> for StorageError {
  fn from(err: redb::TransactionError) -> Self {
    Self::Database(err.to_string())
  }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<redb::TableError> for StorageError {
  fn from(err: redb::TableError) -> Self {
    Self::Database(err.to_string())
  }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<redb::StorageError> for StorageError {
  fn from(err: redb::StorageError) -> Self {
    Self::Database(err.to_string())
  }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<redb::CommitError> for StorageError {
  fn from(err: redb::CommitError) -> Self {
    Self::Database(err.to_string())
  }
}

#[allow(clippy::needless_pass_by_value)]
impl StorageError {
  /// Creates a database error from a redb error (native only)
  #[cfg(not(target_arch = "wasm32"))]
  #[must_use]
  pub fn database(err: redb::Error) -> Self {
    Self::Database(err.to_string())
  }

  /// Creates a serialization error from a `serde_json` error
  #[must_use]
  pub fn serialization(err: serde_json::Error) -> Self {
    Self::Serialization(err.to_string())
  }

  /// Creates a deserialization error from a `serde_json` error
  #[must_use]
  pub fn deserialization(err: serde_json::Error) -> Self {
    Self::Deserialization(err.to_string())
  }
}

/// Validates a project ID.
///
/// # Errors
///
/// Returns `StorageError::InvalidProjectId` if the project ID:
/// - Is empty
/// - Contains path separators (`/` or `\`)
/// - Starts with a dot (`.`)
/// - Contains null bytes
///
/// # Examples
///
/// ```rust
/// use clarity_web::storage::path_util::validate_project_id;
///
/// assert!(validate_project_id("my-project").is_ok());
/// assert!(validate_project_id("my_project-123").is_ok());
/// assert!(validate_project_id("").is_err());
/// assert!(validate_project_id("bad/name").is_err());
/// ```
pub fn validate_project_id(project_id: &str) -> Result<&str, StorageError> {
  match project_id {
    "" => Err(StorageError::InvalidProjectId(
      "project ID cannot be empty".into(),
    )),
    id if id.contains('/') || id.contains('\\') => Err(StorageError::InvalidProjectId(format!(
      "project ID cannot contain path separators: {id}"
    ))),
    id if id.starts_with('.') => Err(StorageError::InvalidProjectId(format!(
      "project ID cannot start with a dot: {id}"
    ))),
    id if id.contains('\0') => Err(StorageError::InvalidProjectId(
      "project ID cannot contain null bytes".into(),
    )),
    id => Ok(id),
  }
}

/// Gets the base application data directory.
///
/// Resolves to `~/.local/share/clarity` on Linux following XDG Base Directory specification.
///
/// # Errors
///
/// Returns `StorageError::PathNotFound` if the XDG data directory cannot be determined.
fn get_app_dir() -> Result<PathBuf, StorageError> {
  dirs::data_local_dir()
    .map(|path| path.join(APP_NAME))
    .ok_or(StorageError::PathNotFound)
}

/// Gets the projects directory.
///
/// Resolves to `~/.local/share/clarity/projects`.
///
/// # Errors
///
/// Returns `StorageError::PathNotFound` if the XDG data directory cannot be determined.
fn get_projects_base_dir() -> Result<PathBuf, StorageError> {
  get_app_dir().map(|path| path.join(PROJECTS_DIR))
}

/// Gets the project directory path for a given project ID.
///
/// Resolves to `~/.local/share/clarity/projects/{project_id}`.
///
/// # Arguments
///
/// * `project_id` - Unique identifier for the project
///
/// # Returns
///
/// Returns the path to the project directory.
///
/// # Errors
///
/// - `StorageError::InvalidProjectId` - if the project ID is invalid
/// - `StorageError::PathNotFound` - if XDG data directory cannot be determined
///
/// # Examples
///
/// ```rust
/// use clarity_web::storage::path_util::get_project_dir;
///
/// let path = get_project_dir("my-project").unwrap();
/// assert!(path.ends_with("clarity/projects/my-project"));
/// ```
pub fn get_project_dir(project_id: &str) -> Result<PathBuf, StorageError> {
  validate_project_id(project_id)?;
  get_projects_base_dir().map(|path| path.join(project_id))
}

/// Gets the database file path for a given project ID.
///
/// Resolves to `~/.local/share/clarity/projects/{project_id}/data.redb`.
///
/// # Arguments
///
/// * `project_id` - Unique identifier for the project
///
/// # Returns
///
/// Returns the path to the project's database file.
///
/// # Errors
///
/// - `StorageError::InvalidProjectId` - if the project ID is invalid
/// - `StorageError::PathNotFound` - if XDG data directory cannot be determined
///
/// # Examples
///
/// ```rust
/// use clarity_web::storage::path_util::get_project_db_path;
///
/// let path = get_project_db_path("my-project").unwrap();
/// assert!(path.ends_with("clarity/projects/my-project/data.redb"));
/// ```
pub fn get_project_db_path(project_id: &str) -> Result<PathBuf, StorageError> {
  get_project_dir(project_id).map(|path| path.join(DB_FILENAME))
}

/// Ensures the project directory exists, creating it if necessary.
///
/// Creates the directory with 0700 permissions (owner read/write/execute only).
/// Parent directories are created as needed.
///
/// # Arguments
///
/// * `project_id` - Unique identifier for the project
///
/// # Errors
///
/// - `StorageError::InvalidProjectId` - if the project ID is invalid
/// - `StorageError::PathNotFound` - if XDG data directory cannot be determined
/// - `StorageError::IoError` - if directory creation fails
///
/// # Examples
///
/// ```rust
/// use clarity_web::storage::path_util::ensure_project_dir_exists;
///
/// // Creates directory if it doesn't exist
/// ensure_project_dir_exists("my-project").unwrap();
/// ```
pub fn ensure_project_dir_exists(project_id: &str) -> Result<(), StorageError> {
  let project_dir = get_project_dir(project_id)?;

  std::fs::create_dir_all(&project_dir)?;

  // Set directory permissions on Unix systems
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    let mut new_perms = std::fs::metadata(&project_dir)?.permissions();
    new_perms.set_mode(DIR_PERMISSIONS);
    std::fs::set_permissions(&project_dir, new_perms)?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  #![allow(clippy::expect_used)]

  use super::*;
  use serial_test::serial;
  use tempfile::TempDir;

  /// Helper to set a custom XDG data directory for testing
  fn set_test_data_dir(temp_dir: &TempDir) {
    std::env::set_var("XDG_DATA_HOME", temp_dir.path());
  }

  /// Helper to restore XDG data directory after testing
  fn restore_data_dir() {
    std::env::remove_var("XDG_DATA_HOME");
  }

  #[test]
  fn test_validate_project_id_valid() {
    // Valid IDs
    assert!(validate_project_id("my-project").is_ok());
    assert!(validate_project_id("my_project-123").is_ok());
    assert!(validate_project_id("Project-ABC_123").is_ok());
    assert!(validate_project_id("a").is_ok());
    assert!(validate_project_id("project-with-many-dashes").is_ok());
  }

  #[test]
  fn test_validate_project_id_empty() {
    let result = validate_project_id("");
    assert!(matches!(result, Err(StorageError::InvalidProjectId(_))));
    if let Err(e) = result {
      assert!(e.to_string().contains("empty"));
    }
  }

  #[test]
  fn test_validate_project_id_with_slash() {
    let result = validate_project_id("bad/name");
    assert!(matches!(result, Err(StorageError::InvalidProjectId(_))));
    if let Err(e) = result {
      assert!(e.to_string().contains("path separators"));
    }
  }

  #[test]
  fn test_validate_project_id_with_backslash() {
    let result = validate_project_id("bad\\name");
    assert!(matches!(result, Err(StorageError::InvalidProjectId(_))));
    if let Err(e) = result {
      assert!(e.to_string().contains("path separators"));
    }
  }

  #[test]
  fn test_validate_project_id_starts_with_dot() {
    let result = validate_project_id(".hidden");
    assert!(matches!(result, Err(StorageError::InvalidProjectId(_))));
    if let Err(e) = result {
      assert!(e.to_string().contains("dot"));
    }
  }

  #[test]
  fn test_validate_project_id_with_null_byte() {
    let result = validate_project_id("bad\0name");
    assert!(matches!(result, Err(StorageError::InvalidProjectId(_))));
    if let Err(e) = result {
      assert!(e.to_string().contains("null"));
    }
  }

  #[test]
  #[serial]
  fn test_get_app_dir() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    set_test_data_dir(&temp_dir);

    let app_dir = get_app_dir();
    assert!(app_dir.is_ok());
    assert!(app_dir.as_ref().is_ok_and(|p| p.ends_with("clarity")));

    restore_data_dir();
  }

  #[test]
  #[serial]
  fn test_get_projects_base_dir() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    set_test_data_dir(&temp_dir);

    let projects_dir = get_projects_base_dir();
    assert!(projects_dir.is_ok());
    assert!(projects_dir
      .as_ref()
      .is_ok_and(|p| p.ends_with("clarity/projects")));

    restore_data_dir();
  }

  #[test]
  #[serial]
  fn test_get_project_dir_valid() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    set_test_data_dir(&temp_dir);

    let result = get_project_dir("test-project");
    assert!(result.is_ok());
    assert!(result
      .as_ref()
      .is_ok_and(|p| p.ends_with("clarity/projects/test-project")));

    restore_data_dir();
  }

  #[test]
  #[serial]
  fn test_get_project_dir_invalid_id() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    set_test_data_dir(&temp_dir);

    let result = get_project_dir("bad/name");
    assert!(matches!(result, Err(StorageError::InvalidProjectId(_))));

    restore_data_dir();
  }

  #[test]
  #[serial]
  fn test_get_project_db_path_valid() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    set_test_data_dir(&temp_dir);

    let result = get_project_db_path("my-project");
    assert!(result.is_ok());
    assert!(result
      .as_ref()
      .is_ok_and(|p| p.ends_with("clarity/projects/my-project/data.redb")));

    restore_data_dir();
  }

  #[test]
  #[serial]
  fn test_get_project_db_path_invalid_id() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    set_test_data_dir(&temp_dir);

    let result = get_project_db_path("/etc/passwd");
    assert!(matches!(result, Err(StorageError::InvalidProjectId(_))));

    restore_data_dir();
  }

  #[test]
  #[serial]
  fn test_ensure_project_dir_exists_creates_directory() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    set_test_data_dir(&temp_dir);

    let project_id = "new-project";
    let project_dir = get_project_dir(project_id).expect("failed to get project dir");

    // Directory should not exist initially
    assert!(!project_dir.exists());

    // Create the directory
    let result = ensure_project_dir_exists(project_id);
    assert!(result.is_ok());
    assert!(project_dir.exists());
    assert!(project_dir.is_dir());

    restore_data_dir();
  }

  #[test]
  #[serial]
  fn test_ensure_project_dir_exists_idempotent() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    set_test_data_dir(&temp_dir);

    let project_id = "existing-project";

    // First creation
    assert!(ensure_project_dir_exists(project_id).is_ok());

    // Second creation should also succeed
    assert!(ensure_project_dir_exists(project_id).is_ok());

    restore_data_dir();
  }

  #[test]
  #[serial]
  fn test_ensure_project_dir_invalid_id() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    set_test_data_dir(&temp_dir);

    let result = ensure_project_dir_exists("");
    assert!(matches!(result, Err(StorageError::InvalidProjectId(_))));

    restore_data_dir();
  }

  #[test]
  #[serial]
  fn test_project_dir_structure() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    set_test_data_dir(&temp_dir);

    let project_id = "structure-test";
    ensure_project_dir_exists(project_id).expect("failed to create project dir");

    let project_dir = get_project_dir(project_id).expect("failed to get project dir");
    let db_path = get_project_db_path(project_id).expect("failed to get db path");

    // Verify project directory exists
    assert!(project_dir.exists());

    // Verify db path is under project directory
    assert!(db_path.starts_with(&project_dir));
    assert!(db_path.file_name().is_some_and(|name| name == DB_FILENAME));

    restore_data_dir();
  }

  #[test]
  #[serial]
  fn test_multiple_projects_separate_directories() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    set_test_data_dir(&temp_dir);

    let project1 = "project-alpha";
    let project2 = "project-beta";

    ensure_project_dir_exists(project1).expect("failed to create project1");
    ensure_project_dir_exists(project2).expect("failed to create project2");

    let dir1 = get_project_dir(project1).expect("failed to get dir1");
    let dir2 = get_project_dir(project2).expect("failed to get dir2");

    // Directories should be different
    assert_ne!(dir1, dir2);

    // Both should exist
    assert!(dir1.exists());
    assert!(dir2.exists());

    restore_data_dir();
  }

  #[test]
  #[serial]
  fn test_db_filename_consistent() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    set_test_data_dir(&temp_dir);

    let path1 = get_project_db_path("test-1").expect("failed to get path1");
    let path2 = get_project_db_path("test-2").expect("failed to get path2");

    // Both should end with the same filename
    assert!(path1.ends_with(DB_FILENAME));
    assert!(path2.ends_with(DB_FILENAME));

    // But should be in different directories
    assert_ne!(path1.parent(), path2.parent());

    restore_data_dir();
  }

  #[test]
  fn test_storage_error_display() {
    let err = StorageError::PathNotFound;
    assert!(err.to_string().contains("XDG"));

    let io_err = StorageError::IoError(std::io::Error::new(
      std::io::ErrorKind::NotFound,
      "test error",
    ));
    assert!(io_err.to_string().contains("I/O"));

    let invalid_err = StorageError::InvalidProjectId("bad-id".into());
    assert!(invalid_err.to_string().contains("invalid"));
  }
}
