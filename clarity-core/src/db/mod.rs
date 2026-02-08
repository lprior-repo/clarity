#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Database module for Clarity application
//!
//! Provides database access, migrations, and repository pattern for entities.

pub mod error;
// TODO: Re-enable managed_pool when deadpool-sqlx is added to dependencies
// The managed_pool module requires deadpool-sqlx which is not currently
// in the workspace dependencies.
// pub mod managed_pool;
pub mod migrate;
pub mod models;
pub mod pool;
pub mod sqlite_pool;

// TODO: Re-enable repository module when database infrastructure is ready
// The repository module requires SQLX to connect to a database at compile time
// for type checking with `sqlx::query!`. This will be re-enabled after:
// 1. Database infrastructure is set up
// 2. SQLX_OFFLINE mode is configured, or
// 3. Runtime query checking is implemented
// pub mod repository;

#[cfg(test)]
mod tests;

pub use error::{DbError, DbResult};
// pub use managed_pool::*;
pub use migrate::*;
pub use models::*;
pub use pool::*;
pub use sqlite_pool::*;
// pub use repository::*;

// Re-export commonly used types
pub use models::{BeadPriority, BeadStatus, BeadType, Email, UserRole};
