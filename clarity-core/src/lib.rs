#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::needless_return)]
#![warn(clippy::unreadable_literal)]
#![warn(clippy::uninlined_format_args)]
#![warn(clippy::doc_markdown)]
#![warn(clippy::must_use_candidate)]
#![warn(clippy::return_self_not_must_use)]
#![warn(clippy::should_implement_trait)]
#![warn(clippy::new_without_default)]
#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::multiple_crate_versions)]

//! Core functionality for the Clarity application

pub mod db;
pub mod error;
pub mod interview;
pub mod json_formatter;
pub mod path_utils;
pub mod progress;
pub mod schema_registry;
pub mod session;
pub mod types;
pub mod validation;

pub use error::{map_db_error, map_validation_error, ExitCode, ExitCodeError};
pub use path_utils::PathError;
pub use schema_registry::{Schema, SchemaId, SchemaRegistry, SchemaRegistryError, SchemaVersion};
pub use types::{HttpMethod, HttpMethodError, SpecName, SpecNameError, Url, UrlError};

/// A simple function to demonstrate core functionality
#[must_use]
pub fn greet(name: &str) -> String {
  format!("Hello, {name}!")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_greet() {
    assert_eq!(greet("World"), "Hello, World!");
  }
}
