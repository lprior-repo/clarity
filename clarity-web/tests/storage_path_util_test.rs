#![allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::unnecessary_debug_formatting,
  clippy::match_same_arms,
  clippy::option_if_let_else,
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro
)]
#![forbid(unsafe_code)]

//! Integration tests for storage `path_util` module

use clarity_web::storage::path_util::{
  ensure_project_dir_exists, get_project_db_path, get_project_dir, StorageError,
};
use tempfile::TempDir;

fn main() {
  println!("Running storage path_util tests...\n");

  // Test 1: Validate project ID - valid cases
  println!("Test 1: Validate project ID (valid)");
  let valid_ids = ["my-project", "my_project-123", "Project-ABC_123", "a"];
  for id in valid_ids {
    let result = clarity_web::storage::path_util::validate_project_id(id);
    assert!(result.is_ok(), "Valid ID {id} should pass validation");
  }
  println!("  ✓ All valid IDs passed\n");

  // Test 2: Validate project ID - invalid cases
  println!("Test 2: Validate project ID (invalid)");
  let invalid_cases = [
    ("", "empty"),
    ("bad/name", "separators"),
    (".hidden", "dot"),
  ];
  for (id, expected_keyword) in invalid_cases {
    let result = clarity_web::storage::path_util::validate_project_id(id);
    match result {
      Err(e) => {
        let msg = e.to_string();
        assert!(
          msg.contains(expected_keyword),
          "Error message should contain '{expected_keyword}': {msg}"
        );
      }
      Ok(_) => panic!("Invalid ID '{id}' should fail validation"),
    }
  }
  println!("  ✓ All invalid IDs rejected properly\n");

  // Test 3: Get project directory
  println!("Test 3: Get project directory");
  let temp_dir = TempDir::new().expect("failed to create temp dir");
  std::env::set_var("XDG_DATA_HOME", temp_dir.path());

  let result = get_project_dir("test-project");
  assert!(result.is_ok(), "Should get project directory");
  let path = result.as_ref().expect("project dir");
  assert!(path.ends_with("clarity/projects/test-project"));

  println!("  ✓ Project directory path correct: {}\n", path.display());

  // Test 4: Get project DB path
  println!("Test 4: Get project DB path");
  let result = get_project_db_path("my-project");
  assert!(result.is_ok(), "Should get DB path");
  let db_path = result.as_ref().expect("db path");
  assert!(db_path.ends_with("clarity/projects/my-project/data.redb"));
  println!("  ✓ DB path correct: {}\n", db_path.display());

  // Test 5: Create project directory
  println!("Test 5: Create project directory");
  let project_id = "new-test-project";
  let project_dir = get_project_dir(project_id).expect("project dir");
  assert!(
    !project_dir.exists(),
    "Directory should not exist initially"
  );

  let result = ensure_project_dir_exists(project_id);
  assert!(result.is_ok(), "Should create directory");
  assert!(
    project_dir.exists(),
    "Directory should exist after creation"
  );
  assert!(project_dir.is_dir(), "Should be a directory");
  println!(
    "  ✓ Directory created successfully: {}\n",
    project_dir.display()
  );

  // Test 6: Idempotent directory creation
  println!("Test 6: Idempotent directory creation");
  let project_id = "idempotent-test";
  assert!(ensure_project_dir_exists(project_id).is_ok());
  assert!(ensure_project_dir_exists(project_id).is_ok());
  println!("  ✓ Multiple calls succeed\n");

  // Test 7: Multiple projects
  println!("Test 7: Multiple projects have separate directories");
  ensure_project_dir_exists("project-alpha").expect("create alpha");
  ensure_project_dir_exists("project-beta").expect("create beta");

  let dir1 = get_project_dir("project-alpha").expect("get alpha");
  let dir2 = get_project_dir("project-beta").expect("get beta");

  assert_ne!(dir1, dir2, "Directories should be different");
  assert!(dir1.exists(), "Alpha directory should exist");
  assert!(dir2.exists(), "Beta directory should exist");
  println!("  ✓ Separate directories created\n");

  // Test 8: Project directory structure
  println!("Test 8: Project directory structure");
  let project_id = "structure-test";
  ensure_project_dir_exists(project_id).expect("create structure test");

  let project_dir = get_project_dir(project_id).expect("get project dir");
  let db_path = get_project_db_path(project_id).expect("get db path");

  assert!(project_dir.exists(), "Project directory should exist");
  assert!(
    db_path.starts_with(&project_dir),
    "DB should be under project dir"
  );
  assert!(db_path.file_name().is_some_and(|n| n == "data.redb"));
  println!("  ✓ Directory structure correct\n");

  // Test 9: Storage error display
  println!("Test 9: Storage error messages");
  let err = StorageError::PathNotFound;
  assert!(err.to_string().contains("XDG"));

  let io_err = StorageError::IoError(std::io::Error::new(
    std::io::ErrorKind::NotFound,
    "test error",
  ));
  assert!(io_err.to_string().contains("I/O"));

  let invalid_err = StorageError::InvalidProjectId("bad-id".into());
  assert!(invalid_err.to_string().contains("invalid"));
  println!("  ✓ Error messages formatted correctly\n");

  std::env::remove_var("XDG_DATA_HOME");

  println!("All tests passed! ✓");
}
