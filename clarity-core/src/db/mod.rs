#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Database module for Clarity application
//!
//! Provides database access, migrations, and repository pattern for entities.

// pub mod adapter;
pub mod error;
pub mod migrate;
pub mod models;
// pub mod repository;
pub mod schema;
pub mod sqlite_pool;

// PostgreSQL support removed - this is a SQLite-only desktop application

#[cfg(test)]
mod tests;

// pub use adapter::*;
pub use error::{DbError, DbResult};
pub use migrate::*;
pub use models::*;
// pub use repository::*;
pub use sqlite_pool::*;

// Re-export commonly used types
pub use models::{BeadPriority, BeadStatus, BeadType, Email, UserRole};
