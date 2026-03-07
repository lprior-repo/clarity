#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! CUE loader public boundary.
//!
//! This facade preserves the historical `intent::loader` API while routing the
//! implementation through the typed `loader/error.rs` and `loader/cue.rs`
//! modules. That removes the older stringly duplicate boundary and makes the
//! public module a thin shell over the functional-core design.

#[path = "loader/answer_loader.rs"]
pub mod answer_loader;
#[path = "loader/cue.rs"]
mod cue;
#[path = "loader/error.rs"]
pub mod error;
#[cfg(test)]
#[path = "loader/legacy_tests.rs"]
mod legacy_tests;

use std::path::{Path, PathBuf};

use super::parser::parse_spec;
use super::security::validate_file_path;
use super::types::Spec;

pub use cue::{export_cue_to_json, validate_cue_file};
pub use error::{format_loader_error, LoaderError};

/// Load a CUE file and parse it into a `Spec`.
///
/// # Errors
///
/// Returns `LoaderError` when path validation, file access, CUE validation,
/// export, or spec parsing fails.
pub fn load_cue_file(path: &Path) -> Result<Spec, LoaderError> {
  let path_str = path.to_string_lossy();
  let validated_path = validate_file_path(&path_str)?;
  let path_buf = PathBuf::from(validated_path);

  cue::validate_file_exists(&path_buf)?;
  validate_cue_file(&path_buf)?;
  export_cue_to_json(&path_buf).and_then(|json| parse_spec(&json).map_err(LoaderError::from))
}
