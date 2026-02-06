#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! File path utilities with comprehensive error handling
//!
//! This module provides safe, functional utilities for common file path operations.
//! All functions return `Result` types and follow zero-panic principles.

use std::path::{Path, PathBuf};

/// Errors that can occur during path operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
  /// Path contains invalid characters
  InvalidCharacters(String),
  /// Path is empty
  EmptyPath,
  /// Path does not exist
  NotFound(PathBuf),
  /// Path is not a file
  NotAFile(PathBuf),
  /// Path is not a directory
  NotADirectory(PathBuf),
  /// Path is not absolute
  NotAbsolute(PathBuf),
  /// Path extension is missing
  MissingExtension(PathBuf),
  /// Invalid UTF-8 in path
  InvalidUtf8,
}

impl std::fmt::Display for PathError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::InvalidCharacters(path) => write!(f, "Path contains invalid characters: {path}"),
      Self::EmptyPath => write!(f, "Path cannot be empty"),
      Self::NotFound(path) => write!(f, "Path not found: {}", path.display()),
      Self::NotAFile(path) => write!(f, "Path is not a file: {}", path.display()),
      Self::NotADirectory(path) => write!(f, "Path is not a directory: {}", path.display()),
      Self::NotAbsolute(path) => write!(f, "Path is not absolute: {}", path.display()),
      Self::MissingExtension(path) => write!(f, "Path missing extension: {}", path.display()),
      Self::InvalidUtf8 => write!(f, "Path contains invalid UTF-8"),
    }
  }
}

impl std::error::Error for PathError {}

/// Validate that a path string contains only valid characters
///
/// # Errors
/// - Returns `PathError::EmptyPath` if the path is empty
/// - Returns `PathError::InvalidCharacters` if the path contains null bytes
///
/// # Examples
/// ```
/// use clarity_core::path_utils::validate_path_chars;
///
/// assert!(validate_path_chars("valid/path.txt").is_ok());
/// assert!(validate_path_chars("").is_err());
/// assert!(validate_path_chars("invalid\0path").is_err());
/// ```
pub fn validate_path_chars(path: &str) -> Result<(), PathError> {
  if path.is_empty() {
    return Err(PathError::EmptyPath);
  }

  if path.contains('\0') {
    return Err(PathError::InvalidCharacters(path.to_string()));
  }

  Ok(())
}

/// Get the file extension from a path
///
/// # Errors
/// - Returns `PathError::MissingExtension` if the path has no extension
///
/// # Examples
/// ```
/// use clarity_core::path_utils::get_extension;
///
/// assert_eq!(get_extension("file.txt").unwrap(), "txt");
/// assert_eq!(get_extension("file.tar.gz").unwrap(), "gz");
/// assert!(get_extension("file").is_err());
/// ```
pub fn get_extension(path: &str) -> Result<&str, PathError> {
  Path::new(path)
    .extension()
    .and_then(|ext| ext.to_str())
    .ok_or_else(|| PathError::MissingExtension(path.into()))
}

/// Get the file stem (name without extension) from a path
///
/// # Errors
/// - Returns `PathError::EmptyPath` if the path is empty
///
/// # Examples
/// ```
/// use clarity_core::path_utils::get_file_stem;
///
/// assert_eq!(get_file_stem("file.txt").unwrap(), "file");
/// assert_eq!(get_file_stem("file.tar.gz").unwrap(), "file.tar");
/// assert_eq!(get_file_stem("/path/to/file.txt").unwrap(), "file");
/// ```
pub fn get_file_stem(path: &str) -> Result<&str, PathError> {
  if path.is_empty() {
    return Err(PathError::EmptyPath);
  }

  Path::new(path)
    .file_stem()
    .and_then(|stem| stem.to_str())
    .ok_or(PathError::InvalidUtf8)
}

/// Get the parent directory of a path
///
/// # Errors
/// - Returns `PathError::EmptyPath` if the path has no parent
///
/// # Examples
/// ```
/// use clarity_core::path_utils::get_parent;
///
/// assert_eq!(get_parent("/path/to/file.txt").unwrap().as_str(), "/path/to");
/// assert_eq!(get_parent("file.txt").unwrap().as_str(), "");
/// assert!(get_parent("/").is_ok());
/// ```
pub fn get_parent(path: &str) -> Result<String, PathError> {
  if path.is_empty() {
    return Err(PathError::EmptyPath);
  }

  Ok(Path::new(path)
    .parent()
    .map_or(String::new(), |p| {
      p.to_str().map_or(String::new(), String::from)
    }))
}

/// Join two path components
///
/// # Errors
/// - Returns `PathError::EmptyPath` if both paths are empty
///
/// # Examples
/// ```
/// use clarity_core::path_utils::join_paths;
///
/// assert_eq!(join_paths("/path/to", "file.txt").unwrap(), "/path/to/file.txt");
/// assert_eq!(join_paths("/path/to/", "file.txt").unwrap(), "/path/to/file.txt");
/// ```
pub fn join_paths(base: &str, path: &str) -> Result<String, PathError> {
  if base.is_empty() && path.is_empty() {
    return Err(PathError::EmptyPath);
  }

  let base_path = Path::new(base);
  let result = base_path.join(path);

  result
    .to_str()
    .map(String::from)
    .ok_or(PathError::InvalidUtf8)
}

/// Normalize a path by resolving `.` and `..` components
///
/// # Errors
/// - Returns `PathError::EmptyPath` if the path is empty
///
/// # Examples
/// ```
/// use clarity_core::path_utils::normalize_path;
///
/// assert_eq!(normalize_path("/path/to/./file.txt").unwrap(), "/path/to/file.txt");
/// assert_eq!(normalize_path("/path/to/../file.txt").unwrap(), "/path/file.txt");
/// ```
pub fn normalize_path(path: &str) -> Result<String, PathError> {
  if path.is_empty() {
    return Err(PathError::EmptyPath);
  }

  let normalized = Path::new(path)
    .components()
    .fold(PathBuf::new(), |acc, comp| {
      match comp {
        std::path::Component::ParentDir => {
          acc.parent().map_or_else(|| acc.clone(), Path::to_path_buf)
        }
        std::path::Component::CurDir => acc,
        _ => acc.join(comp),
      }
    });

  normalized
    .to_str()
    .map(String::from)
    .ok_or(PathError::InvalidUtf8)
}

/// Check if a path is absolute
///
/// # Examples
/// ```
/// use clarity_core::path_utils::is_absolute;
///
/// assert!(is_absolute("/path/to/file"));
/// assert!(!is_absolute("path/to/file"));
/// ```
#[must_use]
pub fn is_absolute(path: &str) -> bool {
  Path::new(path).is_absolute()
}

#[cfg(test)]
mod tests {
  use super::*;

  // validate_path_chars tests
  #[test]
  fn test_validate_path_chars_valid() {
    assert!(validate_path_chars("valid/path.txt").is_ok());
    assert!(validate_path_chars("/absolute/path").is_ok());
    assert!(validate_path_chars("relative/path").is_ok());
    assert!(validate_path_chars("path_with_underscores").is_ok());
    assert!(validate_path_chars("path-with-dashes").is_ok());
    assert!(validate_path_chars("path.with.dots").is_ok());
  }

  #[test]
  fn test_validate_path_chars_empty() {
    let result = validate_path_chars("");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), PathError::EmptyPath);
  }

  #[test]
  fn test_validate_path_chars_null_byte() {
    let result = validate_path_chars("invalid\0path");
    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      PathError::InvalidCharacters(_)
    ));
  }

  // get_extension tests
  #[test]
  fn test_get_extension_simple() {
    assert_eq!(get_extension("file.txt").unwrap(), "txt");
  }

  #[test]
  fn test_get_extension_multiple() {
    assert_eq!(get_extension("file.tar.gz").unwrap(), "gz");
  }

  #[test]
  fn test_get_extension_no_extension() {
    let result = get_extension("file");
    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      PathError::MissingExtension(_)
    ));
  }

  #[test]
  fn test_get_extension_with_path() {
    assert_eq!(get_extension("/path/to/file.txt").unwrap(), "txt");
    assert_eq!(get_extension("./relative/file.rs").unwrap(), "rs");
  }

  #[test]
  fn test_get_extension_hidden_file() {
    // Hidden files have no extension in Rust's Path::extension()
    // The entire name is the file stem
    let result = get_extension(".hidden");
    assert!(result.is_err());
  }

  // get_file_stem tests
  #[test]
  fn test_get_file_stem_simple() {
    assert_eq!(get_file_stem("file.txt").unwrap(), "file");
  }

  #[test]
  fn test_get_file_stem_multiple_extensions() {
    assert_eq!(get_file_stem("file.tar.gz").unwrap(), "file.tar");
  }

  #[test]
  fn test_get_file_stem_empty() {
    let result = get_file_stem("");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), PathError::EmptyPath);
  }

  #[test]
  fn test_get_file_stem_with_path() {
    assert_eq!(get_file_stem("/path/to/file.txt").unwrap(), "file");
    assert_eq!(get_file_stem("./relative/file.rs").unwrap(), "file");
  }

  #[test]
  fn test_get_file_stem_no_extension() {
    assert_eq!(get_file_stem("file").unwrap(), "file");
  }

  // get_parent tests
  #[test]
  fn test_get_parent_absolute() {
    assert_eq!(get_parent("/path/to/file.txt").unwrap().as_str(), "/path/to");
    assert_eq!(get_parent("/path/to/").unwrap().as_str(), "/path");
  }

  #[test]
  fn test_get_parent_relative() {
    assert_eq!(get_parent("path/to/file.txt").unwrap().as_str(), "path/to");
    assert_eq!(get_parent("file.txt").unwrap().as_str(), "");
  }

  #[test]
  fn test_get_parent_root() {
    assert_eq!(get_parent("/").unwrap().as_str(), "");
  }

  #[test]
  fn test_get_parent_empty() {
    let result = get_parent("");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), PathError::EmptyPath);
  }

  // join_paths tests
  #[test]
  fn test_join_paths_basic() {
    assert_eq!(
      join_paths("/path/to", "file.txt").unwrap(),
      "/path/to/file.txt"
    );
  }

  #[test]
  fn test_join_paths_with_trailing_slash() {
    assert_eq!(
      join_paths("/path/to/", "file.txt").unwrap(),
      "/path/to/file.txt"
    );
  }

  #[test]
  fn test_join_paths_absolute_second() {
    assert_eq!(
      join_paths("/path/to", "/absolute").unwrap(),
      "/absolute"
    );
  }

  #[test]
  fn test_join_paths_both_empty() {
    let result = join_paths("", "");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), PathError::EmptyPath);
  }

  #[test]
  fn test_join_paths_empty_base() {
    assert_eq!(join_paths("", "file.txt").unwrap(), "file.txt");
  }

  #[test]
  fn test_join_paths_empty_path() {
    let result = join_paths("/path/to", "");
    assert!(result.is_ok());
    // When joining an empty path, we get the base as-is
    assert!(result.unwrap().starts_with("/path/to"));
  }

  // normalize_path tests
  #[test]
  fn test_normalize_path_current_dir() {
    assert_eq!(
      normalize_path("/path/to/./file.txt").unwrap(),
      "/path/to/file.txt"
    );
  }

  #[test]
  fn test_normalize_path_parent_dir() {
    assert_eq!(
      normalize_path("/path/to/../file.txt").unwrap(),
      "/path/file.txt"
    );
  }

  #[test]
  fn test_normalize_path_combined() {
    assert_eq!(
      normalize_path("/path/to/./../file.txt").unwrap(),
      "/path/file.txt"
    );
  }

  #[test]
  fn test_normalize_path_empty() {
    let result = normalize_path("");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), PathError::EmptyPath);
  }

  #[test]
  fn test_normalize_path_already_normalized() {
    assert_eq!(normalize_path("/path/to/file.txt").unwrap(), "/path/to/file.txt");
  }

  #[test]
  fn test_normalize_path_multiple_parent_dirs() {
    assert_eq!(
      normalize_path("/path/to/sub/../other/../file.txt").unwrap(),
      "/path/to/file.txt"
    );
  }

  // is_absolute tests
  #[test]
  fn test_is_absolute_absolute_path() {
    assert!(is_absolute("/path/to/file"));
    assert!(is_absolute("/"));
    assert!(is_absolute("/relative/../path"));
  }

  #[test]
  fn test_is_absolute_relative_path() {
    assert!(!is_absolute("path/to/file"));
    assert!(!is_absolute("./file"));
    assert!(!is_absolute("../file"));
    assert!(!is_absolute("file"));
  }

  #[test]
  fn test_is_absolute_empty() {
    assert!(!is_absolute(""));
  }
}
