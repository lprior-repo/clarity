#[allow(
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
  clippy::match_like_matches_macro,
  clippy::needless_pass_by_value
)]
mod tests {
  use super::super::*;
  use crate::intent::loader::error::{
    format_loader_error, CueBinaryError, CueOutputError, LoaderError,
  };
  use crate::intent::parser::ParseError;
  use crate::intent::security::SecurityError;
  use std::fs;
  use std::io::Write;
  use std::path::Path;
  use tempfile::TempDir;

  fn create_temp_cue_file(content: &str) -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().expect("temp dir should exist");

    let file_path = dir.path().join("test.cue");
    let mut file = fs::File::create(&file_path).expect("file should exist");
    file.write_all(content.as_bytes()).expect("Failed to write");
    drop(file);
    (dir, file_path)
  }

  fn cue_available() -> bool {
    std::process::Command::new("cue")
      .arg("version")
      .output()
      .map(|o| o.status.success())
      .is_ok_and(|v| v)
  }

  #[test]
  fn test_legacy_loader_alias_matches_new_formatter() {
    let err = LoaderError::file_not_found("/tmp/spec.cue");
    let formatted = format_loader_error(&err);

    assert!(formatted.contains("File Not Found"));
    assert!(formatted.contains("/tmp/spec.cue"));
  }

  #[test]
  fn test_legacy_parse_error_conversion_preserves_json_reason() {
    let parse_err = ParseError::JsonError("bad json".into());
    let loader_err: LoaderError = parse_err.into();

    assert!(matches!(
      loader_err,
      LoaderError::Json { location, .. } if location == "parse"
    ));
  }

  #[test]
  fn test_legacy_security_error_conversion_preserves_violation() {
    let sec_err = SecurityError::NullByteDetected;
    let loader_err: LoaderError = sec_err.into();

    assert!(matches!(loader_err, LoaderError::Security { .. }));
  }

  #[test]
  fn test_legacy_validate_file_exists_directory() {
    let temp_dir = TempDir::new().expect("temp dir should exist");
    let dir_path = temp_dir.path();

    let result = cue::validate_file_exists(dir_path);

    assert!(matches!(result, Err(LoaderError::Io { .. })));
  }

  #[test]
  fn test_legacy_validate_file_exists_not_found() {
    let result = cue::validate_file_exists(Path::new("/nonexistent/path/file.cue"));

    assert!(matches!(result, Err(LoaderError::FileNotFound { .. })));
  }

  #[test]
  fn test_legacy_check_cue_binary_contract() {
    let validate_result = validate_cue_file(Path::new("/nonexistent/file.cue"));

    if cue_available() {
      assert!(matches!(
        validate_result,
        Err(LoaderError::FileNotFound { .. } | LoaderError::CommandFailed { .. })
      ));
    } else {
      assert!(matches!(
        validate_result,
        Err(LoaderError::CueBinaryNotFound {
          details: CueBinaryError::NotInPath
        })
      ));
    }
  }

  #[test]
  fn test_legacy_export_cue_to_json_no_spec_field() {
    if !cue_available() {
      return;
    }

    let cue_content = r#"
foo: {
  name: "not-a-spec"
}
"#;

    let (_temp_dir, file_path) = create_temp_cue_file(cue_content);
    let result = export_cue_to_json(&file_path);

    assert!(matches!(result, Err(LoaderError::CommandFailed { .. })));
  }

  #[test]
  fn test_legacy_load_cue_file_valid() {
    if !cue_available() {
      return;
    }

    let cue_content = r#"
spec: {
  name: "my-spec"
  version: "1.0.0"
  description: "A valid spec"
  entities: []
  features: []
}
"#;

    let (_temp_dir, file_path) = create_temp_cue_file(cue_content);
    let result = load_cue_file(&file_path);

    assert!(matches!(result, Ok(spec) if spec.name == "my-spec"));
  }

  #[test]
  fn test_legacy_load_cue_file_security_traversal() {
    let result = load_cue_file(Path::new("../../../etc/passwd"));

    assert!(matches!(
      result,
      Err(LoaderError::Security { .. } | LoaderError::FileNotFound { .. })
    ));
  }

  #[test]
  fn test_legacy_error_format_examples() {
    let examples = [
      LoaderError::command_exit_code("cue vet", 1, "bad syntax"),
      LoaderError::InvalidCueOutput {
        reason: CueOutputError::EmptyOutput,
      },
      LoaderError::CueBinaryNotFound {
        details: CueBinaryError::ExecutionError {
          message: "install cue".to_string(),
        },
      },
    ];

    let formatted: Vec<String> = examples.iter().map(format_loader_error).collect();

    assert!(formatted.iter().any(|line| line.contains("Command Failed")));
    assert!(formatted
      .iter()
      .any(|line| line.contains("Invalid CUE Output")));
    assert!(formatted
      .iter()
      .any(|line| line.contains("CUE Binary Not Found")));
  }
}
