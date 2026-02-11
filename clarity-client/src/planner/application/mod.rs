#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Planner application layer (DDD)
//!
//! This module exposes planner application state and orchestration logic.

pub mod state {
  pub use crate::planner::state::*;
}

pub use state::*;
