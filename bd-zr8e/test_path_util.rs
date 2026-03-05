#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

// Test binary for storage path_util

use std::path::PathBuf;

// Copy the error and functions inline for testing
#[derive(Debug)]
enum StorageError {
    PathNotFound,
    IoError(std::io::Error),
    InvalidProjectId(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::PathNotFound => write!(f, "XDG data directory not found"),
            StorageError::IoError(e) => write!(f, "I/O error: {e}"),
            StorageError::InvalidProjectId(s) => write!(f, "invalid project ID: {s}"),
        }
    }
}

impl From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> Self {
        StorageError::IoError(e)
    }
}

const APP_NAME: &str = "clarity";
const PROJECTS_DIR: &str = "projects";
const DB_FILENAME: &str = "data.redb";
const DIR_PERMISSIONS: u32 = 0o700;

fn validate_project_id(project_id: &str) -> Result<&str, StorageError> {
    match project_id {
        id if id.is_empty() => Err(StorageError::InvalidProjectId(
            "project ID cannot be empty".into(),
        )),
        id if id.contains('/') || id.contains('\\') => Err(StorageError::InvalidProjectId(
            format!("project ID cannot contain path separators: {id}"),
        )),
        id if id.starts_with('.') => Err(StorageError::InvalidProjectId(
            format!("project ID cannot start with a dot: {id}"),
        )),
        id if id.contains('\0') => Err(StorageError::InvalidProjectId(
            "project ID cannot contain null bytes".into(),
        )),
        id => Ok(id),
    }
}

fn get_app_dir() -> Result<PathBuf, StorageError> {
    dirs::data_local_dir()
        .map(|path| path.join(APP_NAME))
        .ok_or(StorageError::PathNotFound)
}

fn get_projects_base_dir() -> Result<PathBuf, StorageError> {
    get_app_dir().map(|path| path.join(PROJECTS_DIR))
}

fn get_project_dir(project_id: &str) -> Result<PathBuf, StorageError> {
    validate_project_id(project_id)?;
    get_projects_base_dir().map(|path| path.join(project_id))
}

fn get_project_db_path(project_id: &str) -> Result<PathBuf, StorageError> {
    get_project_dir(project_id).map(|path| path.join(DB_FILENAME))
}

fn ensure_project_dir_exists(project_id: &str) -> Result<(), StorageError> {
    let project_dir = get_project_dir(project_id)?;

    std::fs::create_dir_all(&project_dir).map_err(StorageError::from)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::metadata(&project_dir)
            .map(|m| m.permissions())
            .map_err(StorageError::from)?;
        let mut new_perms = perms.clone();
        new_perms.set_mode(DIR_PERMISSIONS);
        std::fs::set_permissions(&project_dir, new_perms)
            .map_err(StorageError::from)?;
    }

    Ok(())
}

fn main() {
    println!("Testing path_util functions...\n");

    // Test 1: validate_project_id
    println!("Test 1: validate_project_id");
    assert!(validate_project_id("my-project").is_ok());
    assert!(validate_project_id("my_project-123").is_ok());
    assert!(validate_project_id("").is_err());
    assert!(validate_project_id("bad/name").is_err());
    assert!(validate_project_id(".hidden").is_err());
    println!("  ✓ Pass\n");

    // Test 2: get_app_dir
    println!("Test 2: get_app_dir");
    let temp_dir = tempfile::TempDir::new().expect("temp dir");
    std::env::set_var("XDG_DATA_HOME", temp_dir.path());
    let app_dir = get_app_dir().expect("app dir");
    assert!(app_dir.ends_with("clarity"));
    println!("  ✓ Pass: {}\n", app_dir.display());

    // Test 3: get_project_dir
    println!("Test 3: get_project_dir");
    let proj_dir = get_project_dir("test-project").expect("project dir");
    assert!(proj_dir.ends_with("clarity/projects/test-project"));
    println!("  ✓ Pass: {}\n", proj_dir.display());

    // Test 4: get_project_db_path
    println!("Test 4: get_project_db_path");
    let db_path = get_project_db_path("my-project").expect("db path");
    assert!(db_path.ends_with("clarity/projects/my-project/data.redb"));
    println!("  ✓ Pass: {}\n", db_path.display());

    // Test 5: ensure_project_dir_exists
    println!("Test 5: ensure_project_dir_exists");
    let project_id = "new-test-project";
    let project_dir = get_project_dir(project_id).expect("project dir");
    assert!(!project_dir.exists());

    ensure_project_dir_exists(project_id).expect("create dir");
    assert!(project_dir.exists());
    assert!(project_dir.is_dir());
    println!("  ✓ Pass: {}\n", project_dir.display());

    // Test 6: idempotent
    println!("Test 6: Idempotent directory creation");
    ensure_project_dir_exists("idempotent-test").expect("first");
    ensure_project_dir_exists("idempotent-test").expect("second");
    println!("  ✓ Pass\n");

    // Test 7: multiple projects
    println!("Test 7: Multiple projects");
    ensure_project_dir_exists("project-alpha").expect("alpha");
    ensure_project_dir_exists("project-beta").expect("beta");
    let dir1 = get_project_dir("project-alpha").expect("get alpha");
    let dir2 = get_project_dir("project-beta").expect("get beta");
    assert_ne!(dir1, dir2);
    assert!(dir1.exists());
    assert!(dir2.exists());
    println!("  ✓ Pass\n");

    // Test 8: directory structure
    println!("Test 8: Directory structure");
    ensure_project_dir_exists("structure-test").expect("create");
    let proj_dir = get_project_dir("structure-test").expect("get");
    let db_path = get_project_db_path("structure-test").expect("get db");
    assert!(proj_dir.exists());
    assert!(db_path.starts_with(&proj_dir));
    assert!(db_path.file_name().is_some_and(|n| n == "data.redb"));
    println!("  ✓ Pass\n");

    std::env::remove_var("XDG_DATA_HOME");

    println!("All tests passed! ✓");
}
