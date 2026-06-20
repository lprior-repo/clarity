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
pub mod newtypes;
pub mod quality;
pub mod scenario;
pub mod straw_man;
pub mod types;

pub use error::ClarityError;
pub use newtypes::{AnswerId, AnswerValue, BeadId, StepId, Timestamp};
pub use scenario::{Hole, HolePunchingResults, HoleType, ScenarioField};
pub use straw_man::{StrawManTrap, StrawManValidation};
pub use types::{Answer, Behavior, Feature, Spec};
