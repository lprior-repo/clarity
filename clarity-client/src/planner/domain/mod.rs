#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Planner domain layer (DDD)
//!
//! This module groups the core planner domain model and domain services.
//! It intentionally re-exports existing planner domain modules so the
//! codebase can migrate incrementally without breaking call sites.

/// Domain model entities and value objects.
pub mod model {
  pub use crate::planner::types::*;
}

/// Domain policies and validation services.
pub mod services {
  pub use crate::planner::validation::*;
}

pub use model::*;
pub use services::*;
