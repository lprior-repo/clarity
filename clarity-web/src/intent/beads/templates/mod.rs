pub mod domain;
pub mod generator;
pub mod render;

pub use domain::{BeadError, BeadTemplate, BeadTemplateStats};
pub use generator::{generate_beads_from_session, generate_profile_beads};
pub use render::{beads_to_enhanced_cue, beads_to_jsonl, decode_beads_json};