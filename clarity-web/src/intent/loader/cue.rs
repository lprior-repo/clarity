use super::LoaderError;
use std::path::Path;
use std::process::Command;

pub fn validate_file_exists(path: &Path) -> Result<(), LoaderError> {
  if !path.exists() {
    return Err(LoaderError::FileNotFound(
      path.to_string_lossy().to_string(),
    ));
  }
  if !path.is_file() {
    return Err(LoaderError::Io(format!(
      "Path is not a file: {}",
      path.to_string_lossy()
    )));
  }

  path.metadata().map_err(|error| {
    LoaderError::Io(format!(
      "Cannot read file {}: {}",
      path.to_string_lossy(),
      error
    ))
  })?;

  Ok(())
}

pub fn validate_cue_file(path: &Path) -> Result<(), LoaderError> {
  let path_str = path.to_string_lossy();
  check_cue_binary()?;

  let output = Command::new("cue")
    .args(["vet", &path_str])
    .output()
    .map_err(|error| LoaderError::CommandFailed(format!("Failed to execute cue vet: {error}")))?;

  if output.status.success() {
    Ok(())
  } else {
    Err(LoaderError::CommandFailed(format!(
      "cue vet failed for {}: {}",
      path_str,
      String::from_utf8_lossy(&output.stderr)
    )))
  }
}

pub fn export_cue_to_json(path: &Path) -> Result<String, LoaderError> {
  let path_str = path.to_string_lossy();
  check_cue_binary()?;

  let output = Command::new("cue")
    .args(["export", &path_str, "-e", "spec"])
    .output()
    .map_err(|error| {
      LoaderError::CommandFailed(format!("Failed to execute cue export: {error}"))
    })?;

  if !output.status.success() {
    return Err(LoaderError::CommandFailed(format!(
      "cue export failed for {}: {}",
      path_str,
      String::from_utf8_lossy(&output.stderr)
    )));
  }

  String::from_utf8(output.stdout)
    .map_err(|error| LoaderError::InvalidCueOutput(format!("Invalid UTF-8 in cue output: {error}")))
}

fn check_cue_binary() -> Result<(), LoaderError> {
  match Command::new("cue").arg("version").output() {
    Ok(output) if output.status.success() => Ok(()),
    Ok(_) => Err(LoaderError::CueBinaryNotFound(
      "cue command found but returned error. Ensure CUE is properly installed.".into(),
    )),
    Err(_) => Err(LoaderError::CueBinaryNotFound(
      "cue command not found in PATH. Install CUE from https://cuelang.org/docs/install/".into(),
    )),
  }
}
