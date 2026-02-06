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
pub mod migrate;
pub mod models;
pub mod pool;

#[cfg(test)]
mod tests;

pub use error::{DbError, DbResult};
pub use migrate::*;
pub use models::*;
pub use pool::*;
