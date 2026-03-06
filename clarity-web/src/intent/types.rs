//! Core Spec Types - Specification data structures for intent system

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

mod ai_hints;
mod anti_pattern;
mod behavior;
mod feature;
mod invariant;
mod spec;
mod type_error;
mod verification;

pub use ai_hints::{AIHints, EntityHint, ImplementationHints, SecurityHints};
pub use anti_pattern::AntiPattern;
pub use behavior::Behavior;
pub use feature::Feature;
pub use invariant::Invariant;
pub use spec::Spec;
pub use type_error::TypeError;
pub use verification::Verification;
