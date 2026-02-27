#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

mod domain;
mod generator;
mod render;

pub use domain::{BeadError, BeadTemplate, BeadTemplateStats};
pub use generator::{generate_beads_from_session, generate_profile_beads};
pub use render::{beads_to_enhanced_cue, beads_to_jsonl};
