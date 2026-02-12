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
//!
//! This crate provides the core domain logic following Domain-Driven Design principles.
//!
//! ## Domain Structure
//!
//! The `domain` module contains the core business logic organized as:
//! - **Aggregates**: Consistency boundaries (UserAggregate, BeadAggregate, PlanSessionAggregate)
//! - **Entities**: Objects with identity (User, Bead)
//! - **Value Objects**: Immutable objects defined by their values (Email, BeadId, UserId, etc.)
//! - **Domain Events**: Messages capturing domain occurrences
//! - **Repositories**: Persistence abstractions
//! - **Domain Services**: Stateless business logic

pub mod auth;
pub mod db;
pub mod domain;
pub mod error;
pub mod export;
pub mod formatter;
pub mod import;
pub mod interview;
pub mod json_formatter;
pub mod path_utils;
pub mod pme_lattice;
pub mod progress;
pub mod status_colors;
// pub mod schema_registry;
pub mod session;
pub mod session_manager;
#[cfg(test)]
pub mod session_manager_test;
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

/// Example of functional core usage
#[must_use]
pub fn create_example_users() -> Vec<domain::User> {
  use domain::{Email, User, UserRole};

  // Create a user with proper error handling - returns Option
  fn create_user_with_email(email_str: &str, role: UserRole) -> Option<User> {
    Email::new(email_str.to_string())
      .ok()
      .and_then(|email| User::new(email, "hash".to_string(), role).ok())
  }

  // Collection of user configurations
  let user_configs = vec![
    ("alice@example.com", UserRole::Admin),
    ("bob@example.com", UserRole::User),
  ];

  // Create users using functional pipeline, filtering out any failures
  user_configs
    .into_iter()
    .filter_map(|(email_str, role)| create_user_with_email(email_str, role))
    .collect()
}

/// Example of processing beads with functional pipeline
#[must_use]
pub fn process_high_priority_beads(beads: Vec<domain::Bead>) -> Vec<domain::Bead> {
  use domain::bead_operations::BeadOperations;

  BeadOperations::filter_beads(&beads, None, Some(domain::BeadPriority::HIGH), None, None)
    .into_iter()
    .cloned()
    .collect()
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
