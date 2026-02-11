#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Planner presentation layer (DDD)
//!
//! This module exposes planner UI components for the desktop application.

pub mod ui {
  pub use crate::planner::components::*;
}

pub use ui::*;
