#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! # Database Module
//!
//! This module provides database access, migrations, and data models for the Clarity application.
//!
//! ## Architecture
//!
//! The database module is organized into several sub-modules:
//!
//! - [`error`]: Database-specific error types and result alias
//! - [`migrate`]: Database migration management
//! - [`models`]: Core data models (User, Bead, Interview, Spec, etc.)
//! - [`pool`]: Connection pooling configuration
//! - [`sqlite_pool`]: SQLite-specific connection pool implementation
//!
//! ## Design Principles
//!
//! - **Zero Panic**: All database operations return `Result` types
//! - **Type Safety**: Strongly typed models with validation
//! - **Connection Pooling**: Efficient connection management via `SqlitePool`
//! - **Migration Safety**: Versioned schema migrations with rollback support
//!
//! ## Thread Safety
//!
//! The connection pool (`SqlitePool`) is thread-safe and can be shared across threads.
//! Individual connections are not thread-safe and should not be shared.
//!
//! ## Performance
//!
//! - Connection pooling reduces connection overhead
//! - Prepared statements are used for all queries
//! - Migrations are transactional for safety
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use clarity_core::db::{SqliteDbConfig, SqlitePool};
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!
//! // Configure the database
//! let config = SqliteDbConfig::default();
//!
//! // Create a connection pool
//! let pool = SqlitePool::new(&config).await?;
//!
//! // Run migrations
//! clarity_core::db::run_migrations(&pool).await?;
//!
//! // Use the pool for queries
//! // ...
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! All database operations return [`DbResult<T>`], which is an alias for
//! `Result<T, DbError>`. See [`DbError`] for the full list of possible errors.
//!
//! ## Models
//!
//! Key data models include:
//! - [`User`]: Application user accounts
//! - [`Bead`]: Work items tracked by the bead system
//! - [`Interview`]: Interview data structures
//! - [`Spec`]: API specification records
//! - [`Email`]: Validated email addresses
//! - [`UserRole`]: User permission levels
//! - [`BeadStatus`]: Bead lifecycle states
//! - [`BeadType`]: Categorization of beads
//! - [`BeadPriority`]: Priority levels for beads

pub mod error;
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
pub use migrate::*;
pub use models::*;
pub use pool::*;
pub use sqlite_pool::*;
// pub use repository::*;

// Re-export commonly used types
pub use models::{BeadPriority, BeadStatus, BeadType, Email, UserRole};
