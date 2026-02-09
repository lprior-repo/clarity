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

//! # Clarity Core
//!
//! `clarity-core` provides the foundational functionality for the Clarity application,
//! including database operations, type validation, session management, progress tracking,
//! and output formatting.
//!
//! ## Design Principles
//!
//! This crate follows strict functional programming principles:
//! - **Zero Panic**: All functions return `Result` types instead of panicking
//! - **Type Safety**: Strong typing with validation at boundaries
//! - **Immutability**: Default to immutable data structures
//! - **Composability**: Small, focused functions that compose well
//! - **Error Handling**: Explicit, typed errors with clear context
//!
//! ## Architecture
//!
//! The crate is organized into several key modules:
//!
//! - [`db`]: Database operations and models with connection pooling
//! - [`error`]: Error types and exit code management
//! - [`types`]: Common validated types (URL, HTTP methods, spec names)
//! - [`validation`]: Input validation utilities
//! - [`session`]: Session management and tracking
//! - [`progress`]: Progress calculation and formatting
//! - [`formatter`]: Output formatting with multiple format support
//! - [`interview`]: Interview data structures and builders
//! - [`path_utils`]: Safe file path operations
//! - [`json_formatter`]: JSON formatting utilities
//!
//! ## Getting Started
//!
//! ### Validated Types
//!
//! ```rust
//! use clarity_core::{Url, SpecName};
//!
//! // URLs are validated on creation
//! let url = Url::new("https://example.com".to_string())?;
//! assert_eq!(url.scheme(), "https");
//!
//! // Spec names enforce format rules
//! let name = SpecName::new("my_spec_v1".to_string())?;
//! assert_eq!(name.as_str(), "my_spec_v1");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Validation
//!
//! ```rust
//! use clarity_core::validation::{validate_non_empty, validate_email_format};
//!
//! // Chain validations for robust input handling
//! let email = "user@example.com";
//! let validated = validate_non_empty(email)
//!     .and_then(|_| validate_email_format(email))?;
//! # Ok::<(), clarity_core::validation::ValidationError>(())
//! ```
//!
//! ### Error Handling
//!
//! ```rust
//! use clarity_core::{ExitCode, map_validation_error};
//! use clarity_core::validation::ValidationError;
//!
//! // Convert domain errors to exit codes
//! let error = ValidationError::EmptyInput;
//! let code = map_validation_error(&error)?;
//! assert_eq!(code, ExitCode::VALIDATION_ERROR);
//! # Ok::<(), clarity_core::error::ExitCodeError>(())
//! ```
//!
//! ## Thread Safety
//!
//! Most types in this crate are `Send` and `Sync` when their internal
//! data supports it. Database connections require careful handling -
//! use connection pools for concurrent access.
//!
//! ## Performance
//!
//! - All validation is O(n) where n is input length
//! - Database operations use prepared statements
//! - Path operations avoid allocations where possible
//! - Progress calculations are O(m) where m is the number of items
//!
//! ## Feature Flags
//!
//! Currently no feature flags are exposed. All functionality is available by default.
//!
//! ## Error Handling Philosophy
//!
//! This crate never panics in production code. All errors are explicitly
//! handled through `Result` types. See the [`error`] module for details
//! on error types and conversion utilities.

pub mod db;
pub mod error;
pub mod formatter;
pub mod interview;
pub mod json_formatter;
pub mod path_utils;
pub mod progress;
// pub mod schema_registry;
pub mod session;
pub mod types;
pub mod validation;

pub use error::{map_db_error, map_validation_error, ExitCode, ExitCodeError};
pub use path_utils::PathError;
// pub use schema_registry::{Schema, SchemaId, SchemaRegistry, SchemaRegistryError, SchemaVersion};
pub use types::{HttpMethod, HttpMethodError, SpecName, SpecNameError, Url, UrlError};

/// A simple function to demonstrate core functionality
#[must_use]
pub fn greet(name: &str) -> String {
  format!("Hello, {name}!")
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::expect_used)]

  use super::*;

  #[test]
  fn test_greet() {
    assert_eq!(greet("World"), "Hello, World!");
  }
}
