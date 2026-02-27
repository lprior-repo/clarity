//! Security validation module.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]

mod constants;
mod helpers;
mod types;
mod validators;

pub use types::{
  MetacharCategory, PathEncodingType, RegexVulnerability, SecurityError, SecurityResult,
  SessionIdError,
};
pub use validators::{
  is_safe_path, validate_file_path, validate_file_paths, validate_regex_pattern,
  validate_session_id,
};

#[cfg(test)]
mod tests;
