//! Interview Submodule
//!
//! Core interview session management including:
//! - Session creation and lifecycle
//! - Gap detection (profile-specific required fields)
//! - Conflict detection (CAP theorem, anonymous+audit, perspective conflicts)
//! - JSONL storage and session diffing

pub mod types;

// Engine module will be added in WP12
// pub mod engine;

// Storage module will be added in WP15
// pub mod storage;

// Questions module will be added in WP09 continuation
// pub mod questions;

// Re-export types
pub use types::*;
