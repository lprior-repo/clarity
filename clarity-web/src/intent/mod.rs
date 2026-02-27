//! Intent Module - Ported from intent-cli (Gleam → Rust)
//!
//! This module provides the complete intent-cli functionality ported to idiomatic Rust,
//! enabling single-binary distribution by removing the Gleam/Erlang runtime dependency.
//!
//! ## Submodules
//!
//! - [`interview`] - Interview session management, gap/conflict detection
//! - [`plan`] - Execution planning, dependency resolution
//! - [`beads`] - Bead (work item) generation
//! - [`quality`] - Quality analysis and scoring
//! - [`validation`] - Spec validation and linting
//! - [`batch`] - Multi-spec batch processing
//! - [`documents`] - Document generation (vision, ready, etc.)
//! - [`templates`] - Spec template generation
//! - [`cli`] - CLI support utilities
//! - [`util`] - Shared utilities
//!
//! ## Design Principles
//!
//! - **Zero panics**: All fallible operations return `Result<T, E>`
//! - **Type safety**: Leverage Rust's type system for compile-time guarantees
//! - **JSONL compatibility**: Maintain compatibility with existing intent-cli files

pub mod beads;
pub mod cli;
pub mod documents;
pub mod interview;
pub mod plan;
pub mod quality;
pub mod templates;
pub mod util;
pub mod validation;

// Re-export core types at module level for convenience
pub use interview::types::{
    Answer, Conflict, ConflictResolution, Gap, InterviewSession, InterviewStage, Perspective,
    Profile, Question, QuestionCategory, QuestionPriority,
};

// Error types will be added in WP04
// pub mod errors;
// pub use errors::IntentError;
