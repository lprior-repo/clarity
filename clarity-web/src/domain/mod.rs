//! Domain module (Foundation)
//!
//! This module contains the canonical, shared data structures and traits
//! for the Clarity project. It is strictly side-effect free and
//! isolated from the lattice and intent crates.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub mod error;
pub mod quality;
pub mod types;

pub use error::ClarityError;
pub use types::{Answer, Behavior, Feature, Spec};
