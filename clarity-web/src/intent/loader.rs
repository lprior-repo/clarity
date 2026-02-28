#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

mod answer_loader;
mod cue;
mod error;

use std::path::{Path, PathBuf};

use super::parser::parse_spec;
use super::security::validate_file_path;
use super::types::Spec;
pub use answer_loader::{load_from_file, AnswerLoaderError, ParseErrorWithDetails};
pub use cue::{export_cue_to_json, validate_cue_file};
pub use error::{format_loader_error, LoaderError};

/// Loads, validates, and parses a CUE spec file.
///
/// # Errors
/// Returns `LoaderError` when path validation fails, the file cannot be read or validated, or JSON/spec parsing fails.
pub fn load_cue_file(path: &Path) -> Result<Spec, LoaderError> {
  let path_str = path.to_string_lossy();
  let validated_path = validate_file_path(&path_str)?;
  let validated_path = PathBuf::from(validated_path);

  cue::validate_file_exists(&validated_path)?;
  validate_cue_file(&validated_path)?;
  let json = export_cue_to_json(&validated_path)?;
  parse_spec(&json).map_err(LoaderError::from)
}
