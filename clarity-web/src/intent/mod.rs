//! Intent Module - Ported from intent-cli (Gleam -> Rust)
//!
//! This module provides the complete intent-cli functionality ported to idiomatic Rust,
//! enabling single-binary distribution by removing the Gleam/Erlang runtime dependency.
//!
//! ## Submodules
//!
//! - [`types`] - Core spec types (Spec, Feature, Behavior, etc.)
//! - [`parser`] - JSON parser for Spec parsing (WP17)
//! - [`interview`] - Interview session management, gap/conflict detection
//! - [`plan`] - Execution planning, dependency resolution
//! - [`beads`] - Bead (work item) generation
//! - [`quality`] - Quality analysis and scoring
//! - [`validation`] - Spec validation and linting
//! - [`batch`] - Multi-spec batch processing
//! - [`documents`] - Document generation (vision, ready, etc.)
//! - [`templates`] - Spec template generation
//! - [`formats`] - Format validators (email, UUID, URI, ISO 8601)
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
pub mod formats;
pub mod interview;
pub mod parser;
pub mod plan;
pub mod quality;
pub mod security;
pub mod templates;
pub mod types;
pub mod util;
pub mod validation;

// Re-export core types at module level for convenience
pub use interview::types::{
    Answer, Conflict, ConflictResolution, Gap, InterviewSession, InterviewStage, Perspective,
    Profile, Question, QuestionCategory, QuestionPriority,
};

// Re-export spec types
pub use types::{
    AIHints, AntiPattern, Behavior, EntityHint, Feature, ImplementationHints, Invariant,
    SecurityHints, Spec, TypeError, Verification,
};

// Re-export parser types (WP17)
pub use parser::{parse_spec, parse_spec_from_value, sanitize_string, validate_spec, ParseError};

// Error types (WP04)
pub mod errors;
pub use errors::{
    extract_available_fields, format_error, levenshtein, suggest_field_names, ContextualError,
    FieldFailure, IntentError, Suggestion, ValidationError,
};
