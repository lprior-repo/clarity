#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro)]
use crate::intent::security::{
  is_safe_path, validate_file_path, validate_file_paths, MetacharCategory, PathEncodingType,
  SecurityError,
};

#[test]
fn test_is_safe_path_valid() {
  assert!(is_safe_path("safe/path/file.txt"));
  assert!(is_safe_path("file.txt"));
  assert!(is_safe_path("path/to/file"));
  assert!(is_safe_path("a"));
}

#[test]
fn test_is_safe_path_traversal() {
  assert!(!is_safe_path("../etc/passwd"));
  assert!(!is_safe_path("path/../../../etc"));
  assert!(!is_safe_path(".."));
  assert!(!is_safe_path("path/.."));
}

#[test]
fn test_is_safe_path_encoded() {
  assert!(!is_safe_path("%2e%2e/etc/passwd"));
  assert!(!is_safe_path("%2E%2E/etc/passwd"));
  assert!(!is_safe_path("%252e%252e/etc/passwd"));
}

#[test]
fn test_is_safe_path_backslash() {
  assert!(!is_safe_path("..\\windows\\system32"));
  assert!(!is_safe_path("path\\to\\file"));
}

#[test]
fn test_is_safe_path_null_byte() {
  assert!(!is_safe_path("file.txt\0.exe"));
  assert!(!is_safe_path("\0"));
}

#[test]
fn test_is_safe_path_empty() {
  assert!(!is_safe_path(""));
}

#[test]
fn test_validate_file_path_valid() {
  let result = validate_file_path("safe/path/file.txt");
  assert_eq!(result, Ok("safe/path/file.txt".to_owned()));
}

#[test]
fn test_validate_file_path_empty() {
  assert!(matches!(
    validate_file_path(""),
    Err(SecurityError::EmptyInput)
  ));
}

#[test]
fn test_validate_file_path_null_byte() {
  assert!(matches!(
    validate_file_path("file\0.txt"),
    Err(SecurityError::NullByteDetected)
  ));
}

#[test]
fn test_validate_file_path_backslash() {
  assert!(matches!(
    validate_file_path("path\\to\\file"),
    Err(SecurityError::BackslashInPath)
  ));
}

#[test]
fn test_validate_file_path_literal_traversal() {
  assert!(matches!(
    validate_file_path("../../etc/passwd"),
    Err(SecurityError::PathTraversal { .. })
  ));
}

#[test]
fn test_validate_file_path_encoded_traversal() {
  assert!(matches!(
    validate_file_path("%2e%2e/etc/passwd"),
    Err(SecurityError::EncodedPathTraversal {
      encoding_type: PathEncodingType::SingleEncoded
    })
  ));
}

#[test]
fn test_validate_file_path_double_encoded() {
  assert!(matches!(
    validate_file_path("%252e%252e/etc/passwd"),
    Err(SecurityError::EncodedPathTraversal {
      encoding_type: PathEncodingType::DoubleEncoded
    })
  ));
}

#[test]
fn test_validate_file_path_shell_metachar_semicolon() {
  assert!(matches!(
    validate_file_path("file;rm -rf /"),
    Err(SecurityError::ShellMetacharacter {
      category: MetacharCategory::CommandSeparator,
      ..
    })
  ));
}

#[test]
fn test_validate_file_path_shell_metachar_pipe() {
  assert!(matches!(
    validate_file_path("file|cat /etc/passwd"),
    Err(SecurityError::ShellMetacharacter {
      category: MetacharCategory::CommandSeparator,
      ..
    })
  ));
}

#[test]
fn test_validate_file_path_shell_metachar_variable() {
  assert!(matches!(
    validate_file_path("file$HOME"),
    Err(SecurityError::ShellMetacharacter {
      category: MetacharCategory::VariableExpansion,
      ..
    })
  ));
}

#[test]
fn test_validate_file_path_control_character() {
  assert!(matches!(
    validate_file_path("file\x01.txt"),
    Err(SecurityError::ShellMetacharacter {
      category: MetacharCategory::ControlCharacter,
      ..
    })
  ));
}

#[test]
fn test_validate_file_paths_all_valid() {
  let paths = vec!["path1.txt", "path2.txt", "path3.txt"];
  assert_eq!(
    validate_file_paths(&paths).map(|validated| validated.len()),
    Ok(3)
  );
}

#[test]
fn test_validate_file_paths_one_invalid() {
  let paths = vec!["path1.txt", "../etc/passwd", "path3.txt"];
  assert!(validate_file_paths(&paths).is_err());
}

#[test]
fn test_validate_file_paths_empty() {
  let paths: Vec<&str> = vec![];
  assert_eq!(validate_file_paths(&paths), Ok(vec![]));
}
