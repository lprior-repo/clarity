#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// Public API exports - used by library consumers
#![allow(unused_imports)]

//! Storage module for clarity-web.
//!
//! Provides XDG-compliant path resolution and database storage utilities.

pub mod path_util;
pub mod transcript_store;
pub mod types;

#[cfg(not(target_arch = "wasm32"))]
pub mod redb_store;

#[cfg(not(target_arch = "wasm32"))]
pub mod redb_transcript_store;

#[cfg(test)]
mod integration_test;

pub use path_util::{
  ensure_project_dir_exists, get_project_db_path, get_project_dir, validate_project_id,
  StorageError,
};
pub use transcript_store::{
  AntithesisResponse, ExtractedField, InterrogationTranscript, StrawManTrap, StrawManValidation,
  TranscriptResult, TranscriptStore,
};
// Re-export types from discover module for backwards compatibility
pub use crate::components::discover::types::{HolePunchingResults, ScenarioField};
pub use types::{tables, AnswerRecord, Confidence, ExtractionCache, LatticeCache, ProjectMetadata};

#[cfg(not(target_arch = "wasm32"))]
pub use redb_transcript_store::RedbTranscriptStore;

#[cfg(not(target_arch = "wasm32"))]
pub use redb_store::RedbStore;
