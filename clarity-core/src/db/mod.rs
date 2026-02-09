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
pub mod models;
pub mod schema;
pub mod sqlite_pool;

// PostgreSQL-specific modules - only compile when postgres feature is enabled
#[cfg(feature = "postgres")]
pub mod migrate;
#[cfg(feature = "postgres")]
pub mod pool;

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
pub use models::*;
pub use sqlite_pool::*;

// Re-export PostgreSQL-specific types when feature is enabled
#[cfg(feature = "postgres")]
pub use migrate::*;
#[cfg(feature = "postgres")]
pub use pool::*;

// pub use repository::*;

// Re-export commonly used types
pub use models::{BeadPriority, BeadStatus, BeadType, Email, UserRole};
