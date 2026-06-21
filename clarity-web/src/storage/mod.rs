#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Storage module for clarity-web.
//!
//! Provides XDG-compliant path resolution and database storage utilities.
//!
//! `fjall_event_store` is the canonical event-sourced storage foundation for
//! the Clarity CLI target described in `MASTER_DOC.md`. The redb modules remain
//! legacy UI/transcript persistence until migrated or retired.

pub mod path_util;
pub mod transcript_store;
pub mod types;

#[cfg(not(target_arch = "wasm32"))]
pub mod fjall_event_store;

#[cfg(not(target_arch = "wasm32"))]
pub mod redb_store;

#[cfg(not(target_arch = "wasm32"))]
pub mod redb_transcript_store;

#[cfg(test)]
mod integration_test;

pub use crate::domain::scenario::{HolePunchingResults, ScenarioField};
pub use path_util::{
  ensure_project_dir_exists, get_project_db_path, get_project_dir, validate_project_id,
  StorageError,
};
pub use transcript_store::{
  AntithesisResponse, ExtractedField, InterrogationTranscript, StrawManTrap, StrawManValidation,
  TranscriptResult, TranscriptStore,
};
pub use types::{tables, AnswerRecord, Confidence, ExtractionCache, LatticeCache, ProjectMetadata};

#[cfg(not(target_arch = "wasm32"))]
pub use fjall_event_store::{canonical_sha256, event_key, EventEnvelope, FjallEventStore};

#[cfg(not(target_arch = "wasm32"))]
pub use redb_transcript_store::RedbTranscriptStore;

#[cfg(not(target_arch = "wasm32"))]
pub use redb_store::RedbStore;
